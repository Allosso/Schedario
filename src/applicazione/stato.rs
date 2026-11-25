//! Lo stato completo del programma e l'apertura delle cartelle di lavoro.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::rete::StatoRete;
use crate::archivio::cartella_lavoro::ERRORE_IN_USO;
use crate::archivio::{CartellaLavoro, Preferenze, StatoWidget};
use crate::lingue::Traduttore;
use crate::modello::Nota;
use crate::piattaforma::argomenti::Argomenti;

pub struct Applicazione {
    pub preferenze: Preferenze,
    pub tr: Traduttore,
    /// `None` finche' l'utente non ha scelto una cartella.
    pub cartella: Option<CartellaLavoro>,
    /// Messaggio mostrato nella schermata di scelta della cartella.
    pub errore_cartella: Option<String>,
    pub interfaccia: StatoInterfaccia,
    pub widget: Vec<WidgetAperto>,
    pub finestra_visibile: bool,
    pub salvataggio: StatoSalvataggio,
    pub rete: StatoRete,
    /// Server MCP per gli assistenti AI.
    pub mcp: super::mcp::StatoMcp,
    /// Finestra di messaggio in primo piano, se presente.
    pub avviso: Option<Avviso>,
    /// Impostato quando il programma deve chiudersi.
    pub uscita_confermata: bool,

    /// Avvio automatico con il sistema attivo? Letto una volta all'avvio.
    pub avvio_automatico: bool,
    /// Perche' la scorciatoia globale non funziona (per esempio su Wayland).
    pub errore_scorciatoia: Option<String>,
    /// Quante volte le note sono state salvate da quando il programma e'
    /// aperto: il backup automatico del telefono si rifa' quando cambia.
    pub modifiche_salvate: u64,
    /// Telefono: perche' l'ultimo backup automatico non e' riuscito.
    pub errore_backup_automatico: Option<String>,

    pub(super) in_sospeso: BTreeSet<String>,
    pub(super) scadenza_salvataggio: Option<Instant>,
    /// Ci sono modifiche fatte qui da comunicare agli altri computer?
    pub(super) modifiche_da_condividere: bool,
    /// Ultima modifica locale di ciascuna nota: mentre l'utente scrive, le
    /// versioni in arrivo dalla rete vengono rinviate.
    pub(super) ultima_digitazione: HashMap<String, Instant>,
    /// Note arrivate dalla rete in attesa che l'utente smetta di scrivere.
    pub(super) note_rinviate: Vec<(Nota, String)>,
    /// Ultima nota spostata nel cestino, finche' si puo' ancora annullare.
    pub(super) cestinata: Option<(String, Instant)>,
    /// Note del cestino da eliminare definitivamente, e da quando: finche'
    /// l'avviso "Annulla" e' valido restano nel cestino (nascoste), poi si
    /// eliminano davvero (`aggiorna`). Vedi `elimina_con_annulla`.
    pub(super) eliminazione: Option<(Vec<String>, Instant)>,
    /// Preferenze cambiate da salvare (la dimensione del testo cambia a ogni
    /// scatto della rotella: si salva una volta sola, dopo una pausa).
    pub(super) scadenza_preferenze: Option<Instant>,
    /// Ordine delle schede prima di averlo preso da un altro computer,
    /// finche' si puo' ancora annullare.
    pub(super) ordine_precedente: Option<(Vec<String>, Instant)>,
}

/// Telefono: che cosa occupa lo schermo sotto le schede.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pagina {
    /// La nota aperta (o la panoramica, se `panoramica`).
    #[default]
    Nota,
    Impostazioni,
    Cestino,
}

/// Stato dell'interfaccia che sopravvive tra un fotogramma e l'altro.
#[derive(Default)]
pub struct StatoInterfaccia {
    pub selezionata: Option<String>,
    /// Ricerca tra le note (riga delle schede, panoramica).
    pub ricerca: String,
    /// Ricerca tra le note del cestino: ha il suo campo nel Cestino (pagina
    /// sul telefono, sezione del menu sul computer). Vedi `ricerca_per`.
    pub ricerca_cestino: String,
    /// Sezioni della barra laterale: `true` = aperta.
    pub sezioni: BTreeMap<String, bool>,
    /// Scheda in corso di rinomina: (id, testo provvisorio).
    pub rinomina: Option<(String, String)>,
    /// Indice della prima scheda visibile quando non entrano tutte.
    pub prima_scheda: usize,
    /// Telefono: la pagina a schermo intero mostrata al posto della nota
    /// (Impostazioni, Cestino). Non si ricorda: all'avvio si vede la nota.
    pub pagina: Pagina,
    /// Telefono: la nota aperta quando si e' entrati nel Cestino. Uscendo dal
    /// Cestino, se nel frattempo si guardava una nota cestinata, si torna a lei.
    pub prima_del_cestino: Option<String>,
    /// Ricerca: quale parola trovata nella nota aperta e' quella attuale
    /// (frecce ‹ ›), per quale ricerca e quale nota, e se portarla in vista.
    pub ricerca_attuale: usize,
    pub ricerca_attuale_di: Option<(String, String)>,
    pub ricerca_mostra_attuale: bool,
    /// Telefono, tastiera aperta e poco spazio: si mostra solo il testo della nota.
    pub scrittura_compatta: bool,
    /// Telefono: la nota aperta nel fotogramma precedente. Quando se ne apre
    /// un'altra, se quella era rimasta vuota viene eliminata.
    pub aperta_prima: Option<String>,
    /// Panoramica delle schede (telefono) aperta.
    pub panoramica: bool,
    /// Telefono: nel fotogramma prima si trascinava una scheda (il Cestino si colora).
    pub trascinava_nota: bool,
    /// Telefono: quando e' stato premuto Indietro sulla nota. Un secondo
    /// Indietro entro poco esce dall'app.
    pub uscita_chiesta: Option<std::time::Instant>,
    /// Tema del sistema (`Some(true)` scuro), se il sistema lo comunica (Android).
    pub tema_sistema: Option<bool>,
    pub fuoco_ricerca: bool,
    pub fuoco_titolo: bool,
    /// In attesa che l'utente prema la nuova combinazione della nota rapida.
    pub cattura_scorciatoia: bool,
    /// Messaggio mostrato durante la scelta (per esempio "serve Ctrl o Alt").
    pub avviso_cattura: Option<String>,
    /// Ordine delle schede scelto trascinandole (solo su questo computer).
    pub ordine_schede: Vec<String>,
}

pub struct WidgetAperto {
    pub stato: StatoWidget,
    /// Altezza da ripristinare quando il widget ridotto viene espanso.
    pub altezza_espansa: f32,
    /// Posizione e dimensione con cui la finestra e' stata creata.
    pub geometria_iniziale: [f32; 4],
    /// Nota rapida: per quanti fotogrammi ancora chiedere al sistema di
    /// portare il widget in primo piano con il cursore nel testo (0 = no).
    /// Si riprova perche' la finestra appena creata non sempre lo ottiene subito.
    pub fuoco_testo: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatoSalvataggio {
    Salvato,
    InCorso,
    Errore(String),
}

pub enum Avviso {
    Info(String),
    Errore(String),
    ConfermaImportazione(PathBuf),
    /// Conferma dell'eliminazione definitiva di una nota del cestino.
    ConfermaEliminazione(String),
    ConfermaSvuotaCestino,
    Informazioni,
    /// Testi delle licenze (da Informazioni): Schedario e cio' che include.
    Licenze,
    /// Telefono, alla prima apertura: le note stanno solo nell'app, conviene
    /// attivare il backup automatico o la sincronizzazione.
    InvitoBackup,
    /// Versioni precedenti di una nota (dalla piu' recente) e quella scelta.
    Cronologia { id: String, versioni: Vec<crate::archivio::storico::Versione>, scelta: usize },
    /// "Prendi l'ordine delle schede da…": il computer scelto, se c'e'.
    PrendiOrdine { scelta: Option<String> },
}

impl Applicazione {
    pub fn new(argomenti: &Argomenti) -> Self {
        let preferenze = Preferenze::carica(argomenti.cartella_dati.as_deref());
        let tr = Traduttore::new(&preferenze.lingua);
        let mut app = Self {
            preferenze,
            tr,
            cartella: None,
            errore_cartella: None,
            interfaccia: StatoInterfaccia::default(),
            widget: Vec::new(),
            finestra_visibile: true,
            salvataggio: StatoSalvataggio::Salvato,
            rete: StatoRete::default(),
            mcp: Default::default(),
            avviso: None,
            uscita_confermata: false,
            avvio_automatico: crate::piattaforma::avvio_automatico::attivo(),
            errore_scorciatoia: None,
            modifiche_salvate: 0,
            errore_backup_automatico: None,
            in_sospeso: BTreeSet::new(),
            scadenza_salvataggio: None,
            modifiche_da_condividere: false,
            ultima_digitazione: HashMap::new(),
            note_rinviate: Vec::new(),
            cestinata: None,
            eliminazione: None,
            scadenza_preferenze: None,
            ordine_precedente: None,
        };

        // Una cartella indicata esplicitamente vince sull'ultima usata. Al primo
        // avvio si usa Documenti/Schedario senza fare domande: la cartella si
        // cambia poi da "Cartella e backup". Se l'ultima cartella non esiste
        // piu' (per esempio un disco scollegato) non se ne crea una nuova di
        // nascosto: la schermata di scelta spiega cosa e' successo.
        let candidata = match (&argomenti.cartella_note, &app.preferenze.ultima_cartella) {
            (Some(esplicita), _) => Some(esplicita.clone()),
            (None, Some(ultima)) if ultima.is_dir() => Some(ultima.clone()),
            (None, Some(ultima)) => {
                app.errore_cartella =
                    Some(app.tr.f("cartella_mancante", &[("percorso", &ultima.display().to_string())]));
                None
            }
            (None, None) => Some(crate::piattaforma::percorsi::cartella_note_predefinita()),
        };
        if let Some(cartella) = candidata {
            if let Err(errore) = app.apri_cartella(&cartella) {
                app.errore_cartella = Some(errore);
            }
        }
        if argomenti.mostra_finestra || app.widget.is_empty() {
            app.finestra_visibile = true;
        }
        app
    }

    /// Chiude la cartella corrente (salvando tutto) e apre quella indicata.
    /// La cartella precedente resta intatta: le note non vengono copiate.
    pub fn apri_cartella(&mut self, percorso: &Path) -> Result<(), String> {
        if let Some(attuale) = &self.cartella {
            if attuale.radice() == std::path::absolute(percorso).unwrap_or_default() {
                return Ok(());
            }
        }
        if self.cartella.is_some() {
            if !self.salva_ora() {
                return Err(self.tr.t("uscita_bloccata").to_owned());
            }
            self.salva_sessione();
        }

        let nuova = CartellaLavoro::apri(percorso, &self.preferenze.id_dispositivo).map_err(|errore| {
            if errore == ERRORE_IN_USO {
                self.tr.t("cartella_in_uso").to_owned()
            } else {
                self.tr.f("errore_cartella", &[("errore", &errore)])
            }
        })?;

        // Da qui in poi la vecchia cartella viene chiusa (e il suo blocco rilasciato).
        self.rete.ferma();
        self.widget.clear();
        self.in_sospeso.clear();
        self.ultima_digitazione.clear();
        self.note_rinviate.clear();
        self.cestinata = None;
        // Un'eliminazione non ancora confermata non si fa: le note restano nel
        // cestino della cartella di prima.
        self.eliminazione = None;
        self.ordine_precedente = None;
        self.scadenza_salvataggio = None;
        self.salvataggio = StatoSalvataggio::Salvato;
        self.interfaccia = StatoInterfaccia::default();
        self.cartella = Some(nuova);
        self.errore_cartella = None;

        self.ripristina_sessione();
        self.carica_impostazioni_rete();

        self.preferenze.ultima_cartella = self.cartella.as_ref().map(|c| c.radice().to_path_buf());
        self.salva_preferenze();
        Ok(())
    }

    pub fn salva_preferenze(&mut self) {
        if let Err(errore) = self.preferenze.salva() {
            self.avviso = Some(Avviso::Errore(errore));
        }
    }

    pub fn imposta_lingua(&mut self, codice: &str) {
        if self.tr.codice() == codice {
            return;
        }
        self.tr = Traduttore::new(codice);
        self.preferenze.lingua = self.tr.codice().to_owned();
        self.salva_preferenze();
    }

    /// "Come il sistema" (sul telefono "Come il telefono", `automatico`),
    /// oppure chiaro o scuro.
    pub fn imposta_tema(&mut self, automatico: bool, scuro: bool) {
        self.preferenze.tema_automatico = automatico;
        if !automatico {
            self.preferenze.tema_scuro = scuro;
        }
        self.salva_preferenze();
    }

    /// Il tema da usare adesso: quello scelto, oppure quello del sistema se
    /// e' attivo "Come il telefono" e il sistema l'ha detto.
    pub fn tema_scuro_attuale(&self) -> bool {
        match self.interfaccia.tema_sistema {
            Some(scuro) if self.preferenze.tema_automatico => scuro,
            _ => self.preferenze.tema_scuro,
        }
    }

    /// Prepara l'uscita: salva note, sessione e preferenze.
    /// Restituisce `false` (e mostra un avviso) se le note non si possono salvare.
    pub fn prepara_uscita(&mut self) -> bool {
        if self.cartella.is_some() {
            self.conferma_rinomina();
            // Un nome o un codice di rete ancora nel campo non va perso.
            self.conferma_bozze_rete();
            if !self.salva_ora() {
                self.avviso = Some(Avviso::Errore(self.tr.t("uscita_bloccata").to_owned()));
                return false;
            }
            self.salva_sessione();
        }
        self.rete.ferma();
        let _ = self.preferenze.salva();
        self.uscita_confermata = true;
        true
    }

    /// Da chiamare a ogni fotogramma: salvataggi differiti ed eventi di rete.
    pub fn aggiorna(&mut self) {
        if self.scadenza_salvataggio.is_some_and(|t| Instant::now() >= t) {
            self.salva_ora();
        }
        if self.scadenza_preferenze.is_some_and(|t| Instant::now() >= t) {
            self.scadenza_preferenze = None;
            self.salva_preferenze();
        }
        if self.eliminazione.as_ref().is_some_and(|(_, quando)| quando.elapsed() >= super::note::DURATA_ANNULLA) {
            self.conferma_eliminazione();
        }
        self.gestisci_eventi_rete();
        self.gestisci_richieste_mcp();
        self.riprova_note_rinviate();
    }

    /// Tempo che manca al prossimo lavoro programmato (salvataggi, note
    /// rinviate, fine dell'avviso "Annulla"), per sapere quando ridisegnare.
    pub fn prossima_scadenza(&self) -> Option<std::time::Duration> {
        let ora = Instant::now();
        let salvataggio = self.scadenza_salvataggio.map(|t| t.saturating_duration_since(ora));
        let preferenze = self.scadenza_preferenze.map(|t| t.saturating_duration_since(ora));
        let rinvio = (!self.note_rinviate.is_empty()).then(|| std::time::Duration::from_millis(500));
        let annulla = self
            .cestinata
            .as_ref()
            .map(|(_, quando)| (*quando + super::note::DURATA_ANNULLA).saturating_duration_since(ora));
        let annulla_ordine = self
            .ordine_precedente
            .as_ref()
            .map(|(_, quando)| (*quando + super::note::DURATA_ANNULLA).saturating_duration_since(ora));
        let elimina = self
            .eliminazione
            .as_ref()
            .map(|(_, quando)| (*quando + super::note::DURATA_ANNULLA).saturating_duration_since(ora));
        [salvataggio, preferenze, rinvio, annulla, annulla_ordine, elimina].into_iter().flatten().min()
    }

    /// Salva le preferenze tra poco: per i cambi che arrivano a raffica.
    pub fn salva_preferenze_dopo(&mut self) {
        self.scadenza_preferenze = Some(Instant::now() + std::time::Duration::from_millis(800));
    }

    pub fn imposta_avvio_automatico(&mut self, attivo: bool) {
        match crate::piattaforma::avvio_automatico::imposta(attivo) {
            Ok(()) => self.avvio_automatico = attivo,
            Err(errore) => self.avviso = Some(Avviso::Errore(errore)),
        }
    }
}
