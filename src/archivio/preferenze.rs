//! Preferenze generali del programma, indipendenti dalla cartella di note.
//! Non contengono mai il testo delle note.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{Risultato, scrittura_atomica};
use crate::modello::nota::nuovo_id;
use crate::piattaforma::percorsi;

const NOME_FILE: &str = "preferenze.json";

/// Dimensione del testo delle note: predefinita e limiti, in punti.
pub const DIMENSIONE_TESTO_PREDEFINITA: f32 = 15.5;
pub const DIMENSIONE_TESTO_MINIMA: f32 = 11.0;
pub const DIMENSIONE_TESTO_MASSIMA: f32 = 28.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferenze {
    pub ultima_cartella: Option<PathBuf>,
    /// Codice lingua: "it" oppure "en".
    pub lingua: String,
    pub tema_scuro: bool,
    /// Telefono: il tema segue quello del sistema (chiaro o scuro); `tema_scuro`
    /// vale solo se questa scelta e' spenta. Sul computer resta spenta.
    pub tema_automatico: bool,
    /// Identificativo di questo computer nella rete locale.
    pub id_dispositivo: String,
    /// Nome mostrato agli altri computer.
    pub nome_dispositivo: String,
    /// Dimensioni e posizione della finestra principale.
    pub finestra: Option<[f32; 4]>,
    /// Dimensione del testo delle note, in punti (Ctrl + rotella).
    pub dimensione_testo: f32,
    /// Scorciatoia da tastiera globale per una nota rapida.
    pub scorciatoia_globale: bool,
    /// Tasti della scorciatoia globale (predefinita Ctrl+Alt+N).
    pub combinazione: Combinazione,
    /// Menu di destra ridotto a una striscia sottile.
    pub menu_chiuso: bool,
    /// Server MCP per gli assistenti AI (vedi `crate::mcp`): acceso?
    pub mcp_attivo: bool,
    /// Gli assistenti possono anche modificare le note (non solo leggerle)?
    pub mcp_modifiche: bool,
    /// Codice di accesso richiesto dal server MCP (`Authorization: Bearer …`).
    pub mcp_codice: String,
    pub mcp_porta: u16,

    #[serde(skip)]
    percorso: PathBuf,
}

/// Una combinazione di tasti: modificatori piu' un tasto. `tasto` e' il nome
/// del tasto come lo scrive egui ("N", "5", "F7", "Space").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Combinazione {
    pub ctrl: bool,
    pub alt: bool,
    pub maiuscolo: bool,
    /// Tasto Cmd (solo macOS).
    pub comando: bool,
    pub tasto: String,
}

impl Default for Combinazione {
    fn default() -> Self {
        Self { ctrl: true, alt: true, maiuscolo: false, comando: false, tasto: "N".to_owned() }
    }
}

impl Combinazione {
    /// Testo da mostrare, per esempio `Ctrl+Alt+N` (`Ctrl+Option+N` su macOS).
    pub fn descrizione(&self) -> String {
        let alt = if cfg!(target_os = "macos") { "Option" } else { "Alt" };
        let tasto = self.tasto.as_str();
        [(self.ctrl, "Ctrl"), (self.alt, alt), (self.maiuscolo, "Shift"), (self.comando, "Cmd"), (true, tasto)]
            .iter()
            .filter(|(attivo, _)| *attivo)
            .map(|(_, nome)| *nome)
            .collect::<Vec<_>>()
            .join("+")
    }

    /// Serve almeno un modificatore "vero" (Ctrl, Alt o Cmd): da solo, o con
    /// il solo Shift, il tasto servirebbe a scrivere in tutti i programmi.
    pub fn valida(&self) -> bool {
        (self.ctrl || self.alt || self.comando) && !self.tasto.is_empty()
    }
}

impl Default for Preferenze {
    fn default() -> Self {
        Self {
            ultima_cartella: None,
            lingua: "it".to_owned(),
            tema_scuro: false,
            tema_automatico: cfg!(target_os = "android"),
            id_dispositivo: nuovo_id(),
            nome_dispositivo: percorsi::nome_computer(),
            finestra: None,
            dimensione_testo: DIMENSIONE_TESTO_PREDEFINITA,
            scorciatoia_globale: true,
            combinazione: Combinazione::default(),
            menu_chiuso: false,
            mcp_attivo: false,
            mcp_modifiche: false,
            mcp_codice: String::new(),
            mcp_porta: crate::mcp::PORTA_PREDEFINITA,
            percorso: PathBuf::new(),
        }
    }
}

impl Preferenze {
    /// Carica le preferenze dalla cartella indicata, oppure da quella di sistema.
    /// Un file mancante o danneggiato non blocca l'avvio: si riparte dai valori iniziali.
    pub fn carica(cartella: Option<&Path>) -> Self {
        let cartella = cartella
            .map(Path::to_path_buf)
            .unwrap_or_else(percorsi::cartella_configurazione);
        let percorso = cartella.join(NOME_FILE);
        let mut preferenze: Preferenze = scrittura_atomica::leggi_json(&percorso)
            .ok()
            .flatten()
            .unwrap_or_default();
        preferenze.percorso = percorso;
        if !crate::modello::nota::id_valido(&preferenze.id_dispositivo) {
            preferenze.id_dispositivo = nuovo_id();
        }
        if preferenze.nome_dispositivo.trim().is_empty() {
            preferenze.nome_dispositivo = percorsi::nome_computer();
        }
        if preferenze.lingua != "en" {
            preferenze.lingua = "it".to_owned();
        }
        if preferenze.mcp_porta == 0 {
            preferenze.mcp_porta = crate::mcp::PORTA_PREDEFINITA;
        }
        if !preferenze.combinazione.valida() {
            preferenze.combinazione = Combinazione::default();
        }
        preferenze.dimensione_testo = if preferenze.dimensione_testo.is_finite() {
            preferenze.dimensione_testo.clamp(DIMENSIONE_TESTO_MINIMA, DIMENSIONE_TESTO_MASSIMA)
        } else {
            DIMENSIONE_TESTO_PREDEFINITA
        };
        preferenze
    }

    pub fn salva(&self) -> Risultato<()> {
        scrittura_atomica::scrivi_json(&self.percorso, self)
    }

    pub fn percorso(&self) -> &Path {
        &self.percorso
    }
}
