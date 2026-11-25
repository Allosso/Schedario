//! Dati puri dell'applicazione: nessun accesso a disco, rete o grafica.
//! Tutto cio' che sta qui e' facile da testare e da riusare.

pub mod lista;
pub mod nota;
pub mod ordine;
pub mod tempo;

pub use nota::Nota;
