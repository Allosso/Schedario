//! L'interfaccia grafica (egui/eframe).
//!
//! - `programma`: il ciclo di ogni fotogramma, scorciatoie e chiusura;
//! - `avvio`: la schermata di scelta della cartella al primo avvio;
//! - `principale`: la finestra principale (intestazione, schede, editor,
//!   ricerca, barra laterale, finestre di messaggio);
//! - `widget_fluttuante`: le note sul desktop, senza cornice di sistema;
//! - `area_notifica`: icona vicino all'orologio con Mostra/Nuova nota/Esci;
//! - `scorciatoia_globale`: Ctrl+Alt+N per una nota rapida, da qualsiasi programma;
//! - `testo_nota`: il campo di testo delle note (liste di controllo, menu del
//!   tasto destro), uguale nell'editor e nei widget;
//! - `tastiera`: collega i campi di testo alla tastiera del telefono (Android);
//! - `schermo`: disposizione da telefono o da computer, misure per il dito;
//! - `tema`, `componenti`, `icona`: aspetto condiviso.
//!
//! Su Android non ci sono widget sul desktop, icona nell'area di notifica,
//! scorciatoia globale e assistenti AI: quelle parti si compilano lo stesso
//! (dove possibile) ma non vengono usate.

pub mod area_notifica;
pub mod avvio;
pub mod componenti;
pub mod icona;
pub mod icona_forme;
pub mod principale;
mod programma;
pub mod schermo;
#[cfg(not(target_os = "android"))]
pub mod scorciatoia_globale;
pub mod tastiera;
pub mod tema;
pub mod testo_nota;
pub mod widget_fluttuante;

use crate::applicazione::Applicazione;
use crate::piattaforma::argomenti::Argomenti;

/// Crea la finestra principale e avvia il ciclo degli eventi.
pub fn avvia_finestre(argomenti: Argomenti) -> Result<(), String> {
    let app = Applicazione::new(&argomenti);
    let [x, y, larghezza, altezza] = app.preferenze.finestra.unwrap_or([120.0, 80.0, 1100.0, 760.0]);

    let mut finestra = egui::ViewportBuilder::default()
        .with_title(crate::NOME_APP)
        .with_app_id("schedario")
        .with_icon(icona::icona())
        .with_inner_size([larghezza.max(840.0), altezza.max(560.0)])
        .with_min_inner_size([840.0, 560.0])
        // Serve perche' i widget (senza cornice) abbiano angoli trasparenti.
        .with_transparent(true);
    if app.preferenze.finestra.is_some() {
        finestra = finestra.with_position([x.max(0.0), y.max(0.0)]);
    }

    let opzioni = eframe::NativeOptions {
        viewport: finestra,
        renderer: eframe::Renderer::Glow,
        centered: app.preferenze.finestra.is_none(),
        ..Default::default()
    };
    eframe::run_native(
        crate::NOME_APP,
        opzioni,
        Box::new(move |cc| Ok(Box::new(programma::Programma::new(cc, app)))),
    )
    .map_err(|e| e.to_string())
}

/// Android: una sola finestra a schermo intero.
#[cfg(target_os = "android")]
pub fn avvia_android(argomenti: Argomenti, android: crate::piattaforma::android::AndroidApp) -> Result<(), String> {
    let app = Applicazione::new(&argomenti);
    let opzioni = eframe::NativeOptions {
        android_app: Some(android),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        crate::NOME_APP,
        opzioni,
        Box::new(move |cc| Ok(Box::new(programma::Programma::new(cc, app)))),
    )
    .map_err(|e| e.to_string())
}
