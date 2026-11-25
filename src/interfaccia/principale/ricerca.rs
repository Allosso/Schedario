//! Ricerca in cima al menu di destra, alla stessa altezza delle schede che
//! filtra. Le parole trovate sono evidenziate nel testo (vedi `testo_nota`).
//! Cerca solo tra le note: il cestino ha la sua ricerca (`campo_cestino`).
//!
//! Sul telefono la ricerca e' in cima alla panoramica: filtra le schede.
//!
//! Qui c'e' anche la striscia del menu chiuso: « per riaprirlo e la lente.

use egui::{Id, TextEdit, Ui};

use crate::applicazione::Applicazione;
use crate::interfaccia::componenti::{Icona, pulsante_icona};
use crate::interfaccia::schermo;
use crate::interfaccia::tema::Tavolozza;

/// Altezza della riga, uguale a quella delle schede.
const ALTEZZA: f32 = schermo::misura(34.0, 44.0);
const PULSANTE: f32 = schermo::misura(26.0, 38.0);

/// Computer: spazio tra il campo e il menu sotto. Il campo comincia
/// all'altezza delle schede ed e' piu' basso di tanto: cosi' il menu comincia
/// comunque all'altezza della nota, ma non tocca il campo.
/// Come lo spazio tra una scheda e l'altra.
pub const FESSURA: f32 = 4.0;

pub fn mostra(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let telefono = schermo::telefono(ui.ctx());
    let altezza = if telefono { ALTEZZA } else { ALTEZZA - FESSURA };
    ui.horizontal(|ui| {
        ui.set_height(altezza);
        // Sul telefono il menu si chiude con Indietro o toccando fuori: niente ›.
        let pulsanti = if telefono { 1.0 } else { 2.0 };
        let larghezza_campo = (ui.available_width() - pulsanti * (PULSANTE + ui.spacing().item_spacing.x)).max(80.0);
        let suggerimento = app.tr.t("cerca_suggerimento").to_owned();
        let uscita = TextEdit::singleline(&mut app.interfaccia.ricerca)
            .id(Id::unique("ricerca"))
            .hint_text(suggerimento)
            .margin(egui::vec2(10.0, (altezza - 18.0) / 2.0))
            .min_size(egui::vec2(0.0, altezza))
            .desired_width(larghezza_campo)
            .show(ui);
        crate::interfaccia::tastiera::campo(ui, &uscita, &app.interfaccia.ricerca, false, false);
        let campo = uscita.response.response.on_hover_text(app.tr.t("aiuto_ricerca"));
        if std::mem::take(&mut app.interfaccia.fuoco_ricerca) {
            campo.request_focus();
        }
        // Esc nel campo svuota la ricerca (egui toglie il fuoco nello stesso fotogramma).
        if (campo.has_focus() || campo.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            app.interfaccia.ricerca.clear();
        }
        let ha_testo = !app.interfaccia.ricerca.is_empty();
        // Il filtro si applica mentre scrivi: basta la croce per svuotarlo.
        if pulsante_icona(ui, t, Icona::Chiudi, app.tr.t("cancella_ricerca"), ha_testo).clicked() {
            app.interfaccia.ricerca.clear();
            campo.request_focus();
        }
        if !telefono && pulsante_icona(ui, t, Icona::Successivo, app.tr.t("aiuto_chiudi_menu"), true).clicked() {
            app.preferenze.menu_chiuso = true;
            app.salva_preferenze();
        }
    });
}

/// La ricerca del Cestino, con il suo campo (pagina Cestino sul telefono,
/// sezione Cestino del menu sul computer): cerca solo tra le note del
/// cestino, come quella in cima cerca solo tra le note. Le parole trovate si
/// evidenziano anche nella nota del cestino aperta.
pub fn campo_cestino(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let telefono = schermo::telefono(ui.ctx());
    let altezza = if telefono { ALTEZZA } else { ALTEZZA - FESSURA };
    ui.horizontal(|ui| {
        ui.set_height(altezza);
        let larghezza_campo = (ui.available_width() - PULSANTE - ui.spacing().item_spacing.x).max(60.0);
        let suggerimento = app.tr.t("cerca_cestino_suggerimento").to_owned();
        let uscita = TextEdit::singleline(&mut app.interfaccia.ricerca_cestino)
            .id(Id::unique("ricerca_cestino"))
            .hint_text(suggerimento)
            .margin(egui::vec2(10.0, (altezza - 18.0) / 2.0))
            .min_size(egui::vec2(0.0, altezza))
            .desired_width(larghezza_campo)
            .show(ui);
        crate::interfaccia::tastiera::campo(ui, &uscita, &app.interfaccia.ricerca_cestino, false, false);
        let campo = uscita.response.response.on_hover_text(app.tr.t("aiuto_ricerca_cestino"));
        if (campo.has_focus() || campo.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            app.interfaccia.ricerca_cestino.clear();
        }
        let ha_testo = !app.interfaccia.ricerca_cestino.is_empty();
        if pulsante_icona(ui, t, Icona::Chiudi, app.tr.t("cancella_ricerca"), ha_testo).clicked() {
            app.interfaccia.ricerca_cestino.clear();
            campo.request_focus();
        }
    });
}

/// Il menu chiuso: una striscia con « (riapre) e la lente (riapre e cerca).
pub fn menu_chiuso(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    ui.vertical(|ui| {
        ui.add_space((ALTEZZA - PULSANTE) / 2.0);
        if pulsante_icona(ui, t, Icona::Precedente, app.tr.t("aiuto_apri_menu"), true).clicked() {
            app.preferenze.menu_chiuso = false;
            app.salva_preferenze();
        }
        let lente = pulsante_icona(ui, t, Icona::Lente, app.tr.t("aiuto_ricerca"), true);
        // Un puntino ricorda che le schede sono filtrate da una ricerca.
        if !app.interfaccia.ricerca.trim().is_empty() {
            ui.painter().circle_filled(lente.rect.right_top() + egui::vec2(-3.0, 3.0), 3.5, t.accento);
        }
        if lente.clicked() {
            app.preferenze.menu_chiuso = false;
            app.salva_preferenze();
            app.interfaccia.fuoco_ricerca = true;
        }
    });
}
