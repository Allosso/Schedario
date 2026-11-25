//! Sincronizzazione delle note tra computer della stessa rete locale.
//!
//! - `scoperta`: ogni istanza annuncia la propria presenza via UDP broadcast
//!   e ascolta gli annunci delle altre con lo stesso codice di gruppo.
//! - `sincronizzazione`: due istanze si collegano via TCP e si scambiano
//!   solo le note che all'altra mancano o che sono piu' recenti.
//! - `fusione`: le regole pure che decidono quale versione vince.
//! - `cifratura`: ogni messaggio e' cifrato e autenticato (ChaCha20-Poly1305)
//!   con una chiave derivata dal codice di gruppo tramite Argon2id.
//! - `servizio`: i thread in background e il canale verso l'interfaccia.
//!
//! Dettagli del protocollo in `docs/RETE.md`.

pub mod cifratura;
pub mod fusione;
pub mod protocollo;
pub mod scoperta;
pub mod servizio;
pub mod sincronizzazione;

pub use servizio::{ConfigurazioneRete, Dispositivo, Evento, ServizioRete};

/// Lunghezza minima del codice di gruppo.
pub const LUNGHEZZA_MINIMA_CODICE: usize = 8;
