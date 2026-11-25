//! Il servizio di rete: tre thread in background e un canale verso l'interfaccia.
//!
//! 1. **server**: accetta le sincronizzazioni in arrivo (TCP);
//! 2. **scoperta**: annuncia questo computer e ascolta gli altri (UDP);
//! 3. **pianificatore**: si collega agli altri computer quando serve
//!    (modifica locale, nuovo dispositivo, oppure ogni 20 secondi).
//!
//! L'interfaccia non tocca mai i socket: consegna al servizio una copia delle
//! note (`aggiorna_note`) e riceve gli `Evento` con le note arrivate.

use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use super::cifratura::Chiave;
use super::protocollo::{PORTA_SCOPERTA, PORTA_SINCRONIZZAZIONE};
use super::{scoperta, sincronizzazione};
use crate::modello::{Nota, tempo};

const INTERVALLO_PERIODICO: Duration = Duration::from_secs(20);
/// Un dispositivo scoperto che non si annuncia da questo tempo e' considerato assente.
const ASSENZA: Duration = Duration::from_secs(45);

pub struct ConfigurazioneRete {
    pub id_dispositivo: String,
    pub nome: String,
    pub codice: String,
    pub indirizzi_manuali: Vec<String>,
    pub porta_sincronizzazione: u16,
    pub porta_scoperta: u16,
    /// Ascolta solo su 127.0.0.1 e non annuncia nulla in rete (usato dai test).
    pub solo_locale: bool,
}

impl ConfigurazioneRete {
    pub fn new(id_dispositivo: &str, nome: &str, codice: &str) -> Self {
        Self {
            id_dispositivo: id_dispositivo.to_owned(),
            nome: nome.to_owned(),
            codice: codice.to_owned(),
            indirizzi_manuali: Vec::new(),
            porta_sincronizzazione: PORTA_SINCRONIZZAZIONE,
            porta_scoperta: PORTA_SCOPERTA,
            solo_locale: false,
        }
    }
}

#[derive(Debug)]
pub enum Evento {
    NoteRicevute { note: Vec<Nota>, da: String },
    Errore(String),
    /// Risposta a `chiedi_ordine`: l'ordine delle schede dell'altro computer.
    OrdineRicevuto { ordine: Vec<String>, da: String },
    /// `chiedi_ordine` non e' riuscito.
    OrdineNonRicevuto { errore: sincronizzazione::ErroreOrdine, da: String },
}

#[derive(Clone, Debug)]
pub struct Dispositivo {
    pub id: String,
    pub nome: String,
    pub indirizzo: SocketAddr,
    pub visto: Instant,
    /// Istante (ms) dell'ultima sincronizzazione riuscita.
    pub ultima_sincronizzazione: Option<i64>,
    /// Esito dell'ultimo tentativo di collegamento.
    pub raggiungibile: Option<bool>,
    pub manuale: bool,
}

/// Stato condiviso tra l'interfaccia e i thread di rete.
pub(crate) struct Condiviso {
    /// Calcolata in background (Argon2 richiede qualche decimo di secondo).
    chiave: OnceLock<Chiave>,
    pub id_dispositivo: String,
    pub nome: String,
    pub porta: u16,
    pub porta_scoperta: u16,
    pub istantanea: Mutex<Vec<Nota>>,
    /// Ordine delle schede di questo computer, da dare a chi lo chiede.
    pub ordine: Mutex<Vec<String>>,
    pub dispositivi: Mutex<HashMap<String, Dispositivo>>,
    pub manuali: Vec<SocketAddr>,
    pub ferma: AtomicBool,
    sveglia: (Mutex<bool>, Condvar),
    eventi: mpsc::Sender<Evento>,
    ridisegna: Box<dyn Fn() + Send + Sync>,
}

impl Condiviso {
    /// La chiave di gruppo; attende se il calcolo non e' ancora finito.
    pub fn chiave(&self) -> &Chiave {
        self.chiave.wait()
    }

    pub fn invia_evento(&self, evento: Evento) {
        let _ = self.eventi.send(evento);
        (self.ridisegna)();
    }

    pub fn richiedi_sincronizzazione(&self) {
        let (richiesta, segnale) = &self.sveglia;
        *richiesta.lock().unwrap() = true;
        segnale.notify_all();
    }

    pub fn fermato(&self) -> bool {
        self.ferma.load(Ordering::Relaxed)
    }

    /// Registra o aggiorna un dispositivo. Restituisce `true` se e' nuovo
    /// o ha cambiato indirizzo (vale la pena sincronizzarsi subito).
    pub fn registra_dispositivo(&self, id: &str, nome: &str, indirizzo: SocketAddr) -> bool {
        let mut dispositivi = self.dispositivi.lock().unwrap();
        // Un indirizzo manuale ora risponde con un'identita': togliamo il segnaposto.
        dispositivi.retain(|chiave, d| !(d.manuale && d.indirizzo == indirizzo && chiave != id));
        let manuale = self.manuali.contains(&indirizzo);
        match dispositivi.get_mut(id) {
            Some(d) => {
                let cambiato = d.indirizzo != indirizzo;
                d.indirizzo = indirizzo;
                d.nome = nome.to_owned();
                d.visto = Instant::now();
                d.manuale |= manuale;
                cambiato
            }
            None => {
                dispositivi.insert(
                    id.to_owned(),
                    Dispositivo {
                        id: id.to_owned(),
                        nome: nome.to_owned(),
                        indirizzo,
                        visto: Instant::now(),
                        ultima_sincronizzazione: None,
                        raggiungibile: None,
                        manuale,
                    },
                );
                true
            }
        }
    }

    pub fn segna_esito(&self, id: &str, riuscito: bool) {
        if let Some(d) = self.dispositivi.lock().unwrap().get_mut(id) {
            d.raggiungibile = Some(riuscito);
            if riuscito {
                d.ultima_sincronizzazione = Some(tempo::adesso_ms());
                d.visto = Instant::now();
            }
        }
        (self.ridisegna)();
    }
}

pub struct ServizioRete {
    condiviso: Arc<Condiviso>,
    eventi: mpsc::Receiver<Evento>,
}

impl ServizioRete {
    pub fn avvia(
        configurazione: ConfigurazioneRete,
        note: Vec<Nota>,
        ridisegna: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, String> {
        // Porta preferita; se occupata (es. seconda istanza) una libera qualsiasi.
        let interfaccia = if configurazione.solo_locale { "127.0.0.1" } else { "0.0.0.0" };
        let ascolto = TcpListener::bind((interfaccia, configurazione.porta_sincronizzazione))
            .or_else(|_| TcpListener::bind((interfaccia, 0)))
            .map_err(|e| format!("impossibile aprire la porta TCP: {e}"))?;
        ascolto.set_nonblocking(true).map_err(|e| e.to_string())?;
        let porta = ascolto.local_addr().map_err(|e| e.to_string())?.port();

        let manuali: Vec<SocketAddr> = configurazione
            .indirizzi_manuali
            .iter()
            .filter_map(|testo| risolvi_indirizzo(testo))
            .collect();

        let (mittente, eventi) = mpsc::channel();
        let codice = configurazione.codice;
        let condiviso = Arc::new(Condiviso {
            chiave: OnceLock::new(),
            id_dispositivo: configurazione.id_dispositivo,
            nome: configurazione.nome,
            porta,
            porta_scoperta: configurazione.porta_scoperta,
            istantanea: Mutex::new(note),
            ordine: Mutex::new(Vec::new()),
            dispositivi: Mutex::new(HashMap::new()),
            manuali,
            ferma: AtomicBool::new(false),
            sveglia: (Mutex::new(true), Condvar::new()),
            eventi: mittente,
            ridisegna: Box::new(ridisegna),
        });

        let c = condiviso.clone();
        thread::Builder::new()
            .name("rete-chiave".into())
            .spawn(move || {
                let _ = c.chiave.set(Chiave::da_codice(&codice));
            })
            .map_err(|e| e.to_string())?;
        let c = condiviso.clone();
        thread::Builder::new()
            .name("rete-server".into())
            .spawn(move || sincronizzazione::ciclo_server(ascolto, c))
            .map_err(|e| e.to_string())?;
        if !configurazione.solo_locale {
            let c = condiviso.clone();
            thread::Builder::new()
                .name("rete-scoperta".into())
                .spawn(move || scoperta::ciclo(c))
                .map_err(|e| e.to_string())?;
        }
        let c = condiviso.clone();
        thread::Builder::new()
            .name("rete-pianificatore".into())
            .spawn(move || ciclo_pianificatore(c))
            .map_err(|e| e.to_string())?;

        Ok(Self { condiviso, eventi })
    }

    /// Consegna al servizio la versione corrente di tutte le note.
    pub fn aggiorna_note(&self, note: Vec<Nota>) {
        *self.condiviso.istantanea.lock().unwrap() = note;
    }

    pub fn sincronizza_ora(&self) {
        self.condiviso.richiedi_sincronizzazione();
    }

    /// Consegna al servizio l'ordine delle schede di questo computer.
    pub fn aggiorna_ordine(&self, ordine: Vec<String>) {
        *self.condiviso.ordine.lock().unwrap() = ordine;
    }

    /// Chiede in background l'ordine delle schede al dispositivo indicato; la
    /// risposta arriva come `Evento::OrdineRicevuto` o `OrdineNonRicevuto`.
    pub fn chiedi_ordine(&self, id_dispositivo: &str) {
        let dispositivo = self.condiviso.dispositivi.lock().unwrap().get(id_dispositivo).cloned();
        let c = self.condiviso.clone();
        let _ = thread::Builder::new().name("rete-ordine".into()).spawn(move || {
            let Some(d) = dispositivo else {
                let errore = sincronizzazione::ErroreOrdine::NonRaggiungibile;
                c.invia_evento(Evento::OrdineNonRicevuto { errore, da: String::new() });
                return;
            };
            let evento = match sincronizzazione::chiedi_ordine(d.indirizzo, &c) {
                Ok(ordine) => Evento::OrdineRicevuto { ordine, da: d.nome },
                Err(errore) => Evento::OrdineNonRicevuto { errore, da: d.nome },
            };
            c.invia_evento(evento);
        });
    }

    /// Eventi arrivati dall'ultima chiamata.
    pub fn eventi(&self) -> Vec<Evento> {
        self.eventi.try_iter().collect()
    }

    /// Dispositivi presenti o aggiunti a mano, ordinati per nome.
    pub fn dispositivi(&self) -> Vec<Dispositivo> {
        let mut elenco: Vec<_> = self
            .condiviso
            .dispositivi
            .lock()
            .unwrap()
            .values()
            .filter(|d| d.manuale || d.visto.elapsed() < ASSENZA)
            .cloned()
            .collect();
        elenco.sort_by(|a, b| a.nome.to_lowercase().cmp(&b.nome.to_lowercase()));
        elenco
    }

    pub fn porta(&self) -> u16 {
        self.condiviso.porta
    }
}

impl Drop for ServizioRete {
    fn drop(&mut self) {
        // I thread controllano il segnale ogni frazione di secondo e terminano da soli.
        self.condiviso.ferma.store(true, Ordering::Relaxed);
        self.condiviso.richiedi_sincronizzazione();
    }
}

/// Interpreta `192.168.1.20`, `192.168.1.20:47811` o `nome-pc:47811`.
pub fn risolvi_indirizzo(testo: &str) -> Option<SocketAddr> {
    let testo = testo.trim();
    if testo.is_empty() {
        return None;
    }
    let con_porta = if testo.parse::<SocketAddr>().is_ok() || testo.rsplit_once(':').is_some_and(|(_, p)| p.parse::<u16>().is_ok()) {
        testo.to_owned()
    } else {
        format!("{testo}:{PORTA_SINCRONIZZAZIONE}")
    };
    con_porta.to_socket_addrs().ok()?.find(SocketAddr::is_ipv4)
}

fn ciclo_pianificatore(c: Arc<Condiviso>) {
    loop {
        {
            let (richiesta, segnale) = &c.sveglia;
            let mut richiesta = richiesta.lock().unwrap();
            let inizio = Instant::now();
            while !*richiesta && !c.fermato() {
                let attesa = INTERVALLO_PERIODICO.saturating_sub(inizio.elapsed());
                if attesa.is_zero() {
                    break;
                }
                richiesta = segnale.wait_timeout(richiesta, attesa).unwrap().0;
            }
            *richiesta = false;
        }
        if c.fermato() {
            return;
        }
        // Raggruppa le richieste ravvicinate (es. digitazione veloce).
        thread::sleep(Duration::from_millis(300));

        for (id, indirizzo) in bersagli(&c) {
            if c.fermato() {
                return;
            }
            match sincronizzazione::sincronizza_con(indirizzo, &c) {
                Ok(esito) => {
                    c.registra_dispositivo(&esito.id_dispositivo, &esito.nome, indirizzo);
                    c.segna_esito(&esito.id_dispositivo, true);
                    if !esito.ricevute.is_empty() {
                        c.invia_evento(Evento::NoteRicevute { note: esito.ricevute, da: esito.nome });
                    }
                }
                Err(_) => {
                    let chiave = id.unwrap_or_else(|| segnaposto_manuale(&c, indirizzo));
                    c.segna_esito(&chiave, false);
                }
            }
        }
    }
}

/// Dispositivi da contattare: quelli presenti piu' gli indirizzi manuali.
fn bersagli(c: &Condiviso) -> Vec<(Option<String>, SocketAddr)> {
    let dispositivi = c.dispositivi.lock().unwrap();
    let mut elenco: Vec<(Option<String>, SocketAddr)> = dispositivi
        .values()
        .filter(|d| !d.id.starts_with("manuale:") && (d.manuale || d.visto.elapsed() < ASSENZA))
        .map(|d| (Some(d.id.clone()), d.indirizzo))
        .collect();
    let coperti: HashSet<SocketAddr> = elenco.iter().map(|(_, a)| *a).collect();
    elenco.extend(c.manuali.iter().filter(|a| !coperti.contains(a)).map(|a| (None, *a)));
    elenco
}

/// Voce provvisoria per un indirizzo manuale che non ha ancora risposto.
fn segnaposto_manuale(c: &Condiviso, indirizzo: SocketAddr) -> String {
    let id = format!("manuale:{indirizzo}");
    c.dispositivi.lock().unwrap().entry(id.clone()).or_insert_with(|| Dispositivo {
        id: id.clone(),
        nome: indirizzo.to_string(),
        indirizzo,
        visto: Instant::now(),
        ultima_sincronizzazione: None,
        raggiungibile: None,
        manuale: true,
    });
    id
}
