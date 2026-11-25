//! Il server HTTP dell'MCP: un thread in ascolto su 127.0.0.1 e un thread per
//! ogni collegamento. Le chiamate agli strumenti arrivano al coordinatore come
//! `Chiamata` (con un canale per la risposta) e il server attende la risposta.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use serde_json::Value;

use super::protocollo;
use crate::modello::tempo;

/// Porta predefinita (scelta vicino a quelle della rete locale, 47810-47811).
pub const PORTA_PREDEFINITA: u16 = 47820;
/// Quanto il server aspetta che il programma esegua uno strumento.
const ATTESA_RISPOSTA: Duration = Duration::from_secs(15);
const DIMENSIONE_MASSIMA: usize = 4 * 1024 * 1024;

/// Una chiamata a uno strumento, da eseguire dal coordinatore.
pub struct Chiamata {
    pub nome: String,
    pub argomenti: Value,
    risposta: mpsc::Sender<Result<String, String>>,
}

impl Chiamata {
    pub fn rispondi(self, esito: Result<String, String>) {
        let _ = self.risposta.send(esito);
    }
}

struct Condiviso {
    codice: String,
    ferma: AtomicBool,
    chiamate: Mutex<mpsc::Sender<Chiamata>>,
    ridisegna: Box<dyn Fn() + Send + Sync>,
    richieste: AtomicU32,
    ultimo_accesso: AtomicI64,
}

pub struct ServizioMcp {
    condiviso: Arc<Condiviso>,
    chiamate: mpsc::Receiver<Chiamata>,
    porta: u16,
}

impl ServizioMcp {
    /// Avvia il server su `127.0.0.1:porta` (0 = una porta libera, per i test).
    pub fn avvia(porta: u16, codice: &str, ridisegna: impl Fn() + Send + Sync + 'static) -> Result<Self, String> {
        let ascolto = TcpListener::bind((Ipv4Addr::LOCALHOST, porta)).map_err(|e| e.to_string())?;
        ascolto.set_nonblocking(true).map_err(|e| e.to_string())?;
        let porta = ascolto.local_addr().map_err(|e| e.to_string())?.port();
        let (mittente, chiamate) = mpsc::channel();
        let condiviso = Arc::new(Condiviso {
            codice: codice.to_owned(),
            ferma: AtomicBool::new(false),
            chiamate: Mutex::new(mittente),
            ridisegna: Box::new(ridisegna),
            richieste: AtomicU32::new(0),
            ultimo_accesso: AtomicI64::new(0),
        });
        let c = condiviso.clone();
        thread::Builder::new().name("mcp-server".into()).spawn(move || ciclo(ascolto, c)).map_err(|e| e.to_string())?;
        Ok(Self { condiviso, chiamate, porta })
    }

    pub fn porta(&self) -> u16 {
        self.porta
    }

    /// Chiamate arrivate e non ancora eseguite.
    pub fn chiamate(&self) -> Vec<Chiamata> {
        self.chiamate.try_iter().collect()
    }

    /// Richieste accettate (con il codice giusto) da quando il server e' acceso.
    pub fn richieste(&self) -> u32 {
        self.condiviso.richieste.load(Ordering::Relaxed)
    }

    /// Istante (ms) dell'ultima richiesta accettata.
    pub fn ultimo_accesso(&self) -> Option<i64> {
        Some(self.condiviso.ultimo_accesso.load(Ordering::Relaxed)).filter(|t| *t > 0)
    }
}

impl Drop for ServizioMcp {
    fn drop(&mut self) {
        self.condiviso.ferma.store(true, Ordering::Relaxed);
    }
}

fn ciclo(ascolto: TcpListener, c: Arc<Condiviso>) {
    while !c.ferma.load(Ordering::Relaxed) {
        match ascolto.accept() {
            Ok((flusso, _)) => {
                let c = c.clone();
                let _ = thread::Builder::new().name("mcp-richiesta".into()).spawn(move || {
                    let _ = servi(flusso, &c);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(100)),
            Err(_) => thread::sleep(Duration::from_millis(500)),
        }
    }
}

struct RichiestaHttp {
    metodo: String,
    percorso: String,
    intestazioni: Vec<(String, String)>,
    corpo: Vec<u8>,
}

impl RichiestaHttp {
    fn intestazione(&self, nome: &str) -> Option<&str> {
        self.intestazioni.iter().find(|(n, _)| n.eq_ignore_ascii_case(nome)).map(|(_, v)| v.as_str())
    }
}

fn leggi_richiesta(flusso: &TcpStream) -> Result<RichiestaHttp, String> {
    let mut lettore = BufReader::new(flusso);
    let mut riga = String::new();
    lettore.read_line(&mut riga).map_err(|e| e.to_string())?;
    let mut parti = riga.split_whitespace();
    let metodo = parti.next().unwrap_or_default().to_owned();
    let percorso = parti.next().unwrap_or_default().to_owned();
    let mut intestazioni = Vec::new();
    let mut letti = riga.len();
    loop {
        let mut riga = String::new();
        let n = lettore.read_line(&mut riga).map_err(|e| e.to_string())?;
        letti += n;
        if n == 0 || riga.trim_end().is_empty() {
            break;
        }
        if letti > 64 * 1024 {
            return Err("intestazioni troppo lunghe".into());
        }
        if let Some((nome, valore)) = riga.split_once(':') {
            intestazioni.push((nome.trim().to_owned(), valore.trim().to_owned()));
        }
    }
    let lunghezza: usize = intestazioni
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);
    if lunghezza > DIMENSIONE_MASSIMA {
        return Err("richiesta troppo grande".into());
    }
    let mut corpo = vec![0; lunghezza];
    lettore.read_exact(&mut corpo).map_err(|e| e.to_string())?;
    Ok(RichiestaHttp { metodo, percorso, intestazioni, corpo })
}

fn scrivi_risposta(flusso: &mut TcpStream, stato: &str, tipo: Option<&str>, corpo: &[u8]) -> std::io::Result<()> {
    let mut testa = format!("HTTP/1.1 {stato}\r\nContent-Length: {}\r\nConnection: close\r\n", corpo.len());
    if let Some(tipo) = tipo {
        testa.push_str(&format!("Content-Type: {tipo}\r\n"));
    }
    testa.push_str("\r\n");
    flusso.write_all(testa.as_bytes())?;
    flusso.write_all(corpo)?;
    flusso.flush()
}

/// L'intestazione Origin, se c'e', deve essere locale: una pagina web aperta
/// nel browser non deve poter parlare con Schedario (protezione dal "DNS rebinding").
fn origine_locale(origine: Option<&str>) -> bool {
    let Some(origine) = origine else { return true };
    let ospite = origine.split("://").nth(1).unwrap_or(origine);
    let ospite = ospite.split(['/', ':']).next().unwrap_or_default();
    matches!(ospite, "localhost" | "127.0.0.1" | "[::1]")
}

fn codice_giusto(intestazione: Option<&str>, codice: &str) -> bool {
    let Some(valore) = intestazione else { return false };
    let Some(dato) = valore.strip_prefix("Bearer ").map(str::trim) else { return false };
    // Confronto a tempo costante: la durata non rivela quanti caratteri sono giusti.
    dato.len() == codice.len() && dato.bytes().zip(codice.bytes()).fold(0u8, |d, (a, b)| d | (a ^ b)) == 0
}

fn servi(mut flusso: TcpStream, c: &Condiviso) -> Result<(), String> {
    flusso.set_nonblocking(false).map_err(|e| e.to_string())?;
    flusso.set_read_timeout(Some(Duration::from_secs(10))).map_err(|e| e.to_string())?;
    let richiesta = leggi_richiesta(&flusso)?;
    let rispondi = |flusso: &mut TcpStream, stato: &str, tipo: Option<&str>, corpo: &[u8]| {
        scrivi_risposta(flusso, stato, tipo, corpo).map_err(|e| e.to_string())
    };
    if !origine_locale(richiesta.intestazione("Origin")) {
        return rispondi(&mut flusso, "403 Forbidden", None, b"");
    }
    if richiesta.percorso.split('?').next() != Some("/mcp") {
        return rispondi(&mut flusso, "404 Not Found", None, b"");
    }
    if !codice_giusto(richiesta.intestazione("Authorization"), &c.codice) {
        return rispondi(&mut flusso, "401 Unauthorized", None, b"");
    }
    if richiesta.metodo != "POST" {
        // Nessun flusso di eventi dal server: tutto passa dalle risposte ai POST.
        return rispondi(&mut flusso, "405 Method Not Allowed", None, b"");
    }
    c.richieste.fetch_add(1, Ordering::Relaxed);
    c.ultimo_accesso.store(tempo::adesso_ms(), Ordering::Relaxed);
    (c.ridisegna)();

    let Ok(messaggio) = serde_json::from_slice::<Value>(&richiesta.corpo) else {
        let errore = serde_json::json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "JSON non valido" } });
        return rispondi(&mut flusso, "400 Bad Request", Some("application/json; charset=utf-8"), errore.to_string().as_bytes());
    };
    let mut esegui = |nome: &str, argomenti: &Value| esegui_nel_programma(c, nome, argomenti);
    let risposta = match &messaggio {
        Value::Array(messaggi) => {
            let risposte: Vec<Value> = messaggi.iter().filter_map(|m| protocollo::rispondi(m, &mut esegui)).collect();
            (!risposte.is_empty()).then(|| Value::Array(risposte))
        }
        singolo => protocollo::rispondi(singolo, &mut esegui),
    };
    match risposta {
        Some(risposta) => rispondi(&mut flusso, "200 OK", Some("application/json; charset=utf-8"), risposta.to_string().as_bytes()),
        // Solo notifiche: accettate, nessuna risposta.
        None => rispondi(&mut flusso, "202 Accepted", None, b""),
    }
}

/// Consegna la chiamata al programma e ne aspetta l'esito.
fn esegui_nel_programma(c: &Condiviso, nome: &str, argomenti: &Value) -> Result<String, String> {
    let (risposta, esito) = mpsc::channel();
    let chiamata = Chiamata { nome: nome.to_owned(), argomenti: argomenti.clone(), risposta };
    c.chiamate.lock().unwrap().send(chiamata).map_err(|_| "Schedario si sta chiudendo".to_owned())?;
    (c.ridisegna)();
    esito.recv_timeout(ATTESA_RISPOSTA).unwrap_or_else(|_| Err("Schedario non ha risposto in tempo".to_owned()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn solo_origini_locali_e_codice_giusto() {
        assert!(origine_locale(None));
        assert!(origine_locale(Some("http://localhost:3000")));
        assert!(origine_locale(Some("http://127.0.0.1")));
        assert!(!origine_locale(Some("https://sito-qualsiasi.example")));
        assert!(!origine_locale(Some("http://localhost.example.com")));
        assert!(codice_giusto(Some("Bearer abc-123"), "abc-123"));
        assert!(!codice_giusto(Some("Bearer abc-124"), "abc-123"));
        assert!(!codice_giusto(Some("abc-123"), "abc-123"));
        assert!(!codice_giusto(None, "abc-123"));
    }
}
