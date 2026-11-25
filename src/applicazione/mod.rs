//! Il coordinatore: tiene lo stato del programma e offre i comandi che
//! l'interfaccia invoca (nuova nota, cestino, salvataggio, rete...).
//!
//! Non disegna nulla: l'interfaccia legge questo stato e chiama i metodi.
//! Cosi' la logica resta separata dalla grafica e si puo' testare.
//!
//! - `stato`: la struttura `Applicazione` e l'apertura delle cartelle;
//! - `note`: comandi sulle note e salvataggio differito;
//! - `rete`: avvio del servizio di rete e note in arrivo;
//! - `file`: backup, importazione ed esportazione;
//! - `sessione`: widget aperti e stato dell'interfaccia da ricordare;
//! - `mcp`: gli strumenti offerti agli assistenti AI dal server MCP.

mod file;
mod mcp;
mod note;
mod rete;
mod sessione;
mod stato;

pub use mcp::StatoMcp;
pub use rete::StatoRete;
pub use sessione::{ALTEZZA_MINIMA_WIDGET, ALTEZZA_WIDGET_RIDOTTO, LARGHEZZA_MINIMA_WIDGET};
pub use stato::{Applicazione, Avviso, Pagina, StatoInterfaccia, StatoSalvataggio, WidgetAperto};
