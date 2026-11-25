//! Server MCP (Model Context Protocol): permette a un assistente AI (per
//! esempio Claude Code) di leggere, cercare e, se l'utente lo consente,
//! modificare le note.
//!
//! - Ascolta solo su `127.0.0.1`: raggiungibile dallo stesso computer, mai
//!   dalla rete. Senza il codice di accesso (`Authorization: Bearer …`) non
//!   risponde.
//! - Trasporto "Streamable HTTP" dell'MCP nella forma piu' semplice: ogni
//!   richiesta JSON-RPC arriva con un `POST /mcp` e la risposta e' JSON.
//! - Il server non tocca le note: consegna ogni chiamata a uno strumento al
//!   coordinatore (`applicazione::mcp`) tramite un canale, come fa la rete. Le
//!   modifiche passano quindi dagli stessi percorsi di quelle fatte a mano
//!   (salvataggio, cronologia, sincronizzazione).
//! - Niente librerie in piu': JSON con `serde_json`, HTTP con `std::net`.

pub mod protocollo;
pub mod server;

pub use server::{Chiamata, PORTA_PREDEFINITA, ServizioMcp};
