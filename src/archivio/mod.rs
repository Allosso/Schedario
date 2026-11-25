//! Tutto cio' che legge o scrive sul disco.
//!
//! Struttura di una cartella di lavoro scelta dall'utente:
//!
//! ```text
//! Cartella di lavoro/
//!   Note/Nota-<id>.txt        note attive, testo UTF-8 leggibile da qualsiasi editor
//!   Cestino/Nota-<id>.txt     note nel cestino
//!   .schedario/indice.json     metadati e copia di recupero di ogni nota
//!   .schedario/sessione.json   widget aperti, selezione, sezioni della barra
//!   .schedario/rete.json       impostazioni della sincronizzazione
//!   .schedario/storico/<id>.json  versioni precedenti di ogni nota (solo qui)
//!   .schedario.lock            impedisce a due istanze di usare la stessa cartella
//! ```
//!
//! Le preferenze generali (lingua, tema, ultima cartella) stanno invece nella
//! cartella di configurazione del sistema: vedi `piattaforma::percorsi`.

pub mod backup;
pub mod cartella_lavoro;
pub mod file_testo;
pub mod indice;
pub mod preferenze;
pub mod scrittura_atomica;
pub mod sessione;
pub mod storico;

pub use cartella_lavoro::CartellaLavoro;
pub use preferenze::Preferenze;
pub use sessione::{ImpostazioniRete, Sessione, StatoWidget};

/// Errore di archivio con un messaggio gia' leggibile dall'utente.
pub type Risultato<T> = Result<T, String>;

/// Converte un errore di I/O in un messaggio che indica anche il percorso.
pub(crate) fn errore_io(azione: &str, percorso: &std::path::Path, errore: std::io::Error) -> String {
    format!("{azione} {}: {errore}", percorso.display())
}
