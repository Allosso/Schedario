//! Schedario: note desktop con schede, widget fluttuanti e sincronizzazione
//! cifrata tra computer della stessa rete locale.
//!
//! Il codice e' organizzato per macro categorie, una cartella ciascuna:
//!
//! | Cartella        | Responsabilita'                                              |
//! |-----------------|--------------------------------------------------------------|
//! | `modello`       | I dati puri: la nota e il tempo. Nessun disco, rete o grafica |
//! | `archivio`      | Tutto cio' che tocca il disco: file, indice, preferenze      |
//! | `rete`          | Scoperta dei dispositivi, cifratura e sincronizzazione       |
//! | `lingue`        | Testi dell'interfaccia in italiano e inglese                  |
//! | `piattaforma`   | Differenze tra sistemi operativi e argomenti di avvio        |
//! | `applicazione`  | Il coordinatore: stato condiviso, salvataggio, comandi       |
//! | `interfaccia`   | Finestra principale, widget fluttuanti, tema e componenti     |
//!
//! Le dipendenze vanno in una sola direzione: `interfaccia` usa
//! `applicazione`, che usa `archivio` e `rete`, che usano `modello`.

pub mod applicazione;
pub mod archivio;
pub mod interfaccia;
pub mod lingue;
pub mod mcp;
pub mod modello;
pub mod piattaforma;
pub mod rete;

/// Nome mostrato nelle finestre e usato per le cartelle di configurazione.
pub const NOME_APP: &str = "Schedario";

/// Versione letta da Cargo.toml al momento della compilazione.
pub const VERSIONE: &str = env!("CARGO_PKG_VERSION");

/// Punto di ingresso usato da `main.rs`.
pub fn avvia() -> Result<(), String> {
    let argomenti = piattaforma::argomenti::Argomenti::da_riga_di_comando()?;
    if argomenti.aiuto {
        println!("{}", piattaforma::argomenti::TESTO_AIUTO);
        return Ok(());
    }
    interfaccia::avvia_finestre(argomenti)
}
