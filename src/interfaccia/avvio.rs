//! Schermata mostrata solo quando non si riesce ad aprire una cartella di
//! note: l'ultima usata non esiste piu', oppure Documenti/Schedario non si puo'
//! usare. Negli altri casi la cartella viene aperta senza fare domande.

use egui::{Align, CentralPanel, Frame, Layout, RichText, Ui};

#[cfg(not(target_os = "android"))]
use super::componenti::pulsante_accento;
use super::componenti::{self, pulsante};
use super::tema::Tavolozza;
use crate::applicazione::Applicazione;
use crate::piattaforma::percorsi;

pub fn mostra(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    CentralPanel::default().frame(Frame::new().fill(t.sfondo)).show(ui, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space((ui.available_height() * 0.18).max(20.0));
            componenti::riquadro(t).inner_margin(28).show(ui, |ui| {
                ui.set_width(460.0);
                ui.vertical_centered(|ui| {
                    componenti::logo(ui, 56.0);
                    ui.add_space(8.0);
                    ui.label(RichText::new(crate::NOME_APP).size(24.0).strong());
                    ui.label(RichText::new(app.tr.t("motto")).color(t.attenuato));
                    ui.add_space(18.0);
                    ui.label(RichText::new(app.tr.t("setup_titolo")).size(super::tema::TITOLO_DIALOGO).strong());
                    ui.add_space(4.0);
                    componenti::nota_attenuata(ui, t, app.tr.t("setup_testo"));
                    ui.add_space(14.0);

                    if let Some(errore) = &app.errore_cartella {
                        componenti::errore(ui, t, errore);
                        ui.add_space(8.0);
                    }

                    // Su Android le note stanno sempre nella cartella dell'app.
                    #[cfg(not(target_os = "android"))]
                    if pulsante_accento(ui, t, app.tr.t("setup_scegli")).on_hover_text(app.tr.t("aiuto_setup_scegli")).clicked() {
                        if let Some(cartella) = rfd::FileDialog::new().set_title(app.tr.t("setup_titolo")).pick_folder() {
                            apri(app, &cartella);
                        }
                    }
                    let predefinita = percorsi::cartella_note_predefinita();
                    let testo = app.tr.f("setup_predefinita", &[("percorso", &predefinita.display().to_string())]);
                    if pulsante(ui, t, &testo).on_hover_text(app.tr.t("aiuto_setup_predefinita")).clicked() {
                        apri(app, &predefinita);
                    }
                    ui.add_space(6.0);
                    if pulsante(ui, t, app.tr.t("esci")).on_hover_text(app.tr.t("aiuto_setup_esci")).clicked() {
                        // Uscire da qui chiude il programma senza creare nulla.
                        app.uscita_confermata = true;
                    }
                });
            });
        });
    });
}

fn apri(app: &mut Applicazione, cartella: &std::path::Path) {
    if let Err(errore) = app.apri_cartella(cartella) {
        app.errore_cartella = Some(errore);
    }
}
