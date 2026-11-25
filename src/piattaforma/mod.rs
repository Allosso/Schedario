//! Tutto cio' che cambia da un sistema operativo all'altro, isolato qui:
//! percorsi delle cartelle di configurazione, argomenti di avvio, avvio
//! automatico all'accesso e fuso orario per mostrare le date. Su Android,
//! anche il collegamento con la parte Java e con la tastiera (`android`).

#[cfg(target_os = "android")]
pub mod android;
pub mod argomenti;
pub mod avvio_automatico;
pub mod ora_locale;
pub mod percorsi;
