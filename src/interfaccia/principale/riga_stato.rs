//! Stato della nota e messaggi brevi.
//!
//! - **Computer**: niente riga di stato. A destra nella riga della data, in
//!   grigio, i contatori ("12 righe · 85 parole · Spunte: 2 di 5"); i messaggi ("Nota spostata nel cestino · Annulla",
//!   l'ordine preso da un altro computer, le note arrivate dalla rete, un
//!   errore di salvataggio) in un riquadro piccolo in basso al centro della
//!   nota, che sparisce da se'. Il salvataggio e' automatico: "Salvato" non
//!   si scrive, si dice solo se non riesce.
//! - **Telefono**: una riga sotto la nota solo quando c'e' qualcosa da dire
//!   (errore di salvataggio, ordine preso da un altro dispositivo); "Annulla"
//!   dopo il Cestino e' un riquadro a parte (`principale::avviso_annulla`).

use egui::{Align2, Rect, RichText, Ui};

use crate::applicazione::{Applicazione, StatoSalvataggio};
use crate::interfaccia::tema::{RAGGIO_CONTENITORI, TESTO_SECONDARIO, Tavolozza};

/// Per quanto resta il messaggio della rete ("3 note ricevute da …").
const DURATA_MESSAGGIO_RETE: std::time::Duration = std::time::Duration::from_secs(8);

/// Telefono: la riga sotto la nota.
pub fn mostra(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    ui.horizontal(|ui| messaggi(ui, app, t));
}

/// Telefono: c'e' qualcosa da dire nella riga sotto la nota?
pub fn ha_qualcosa_telefono(app: &Applicazione) -> bool {
    matches!(app.salvataggio, StatoSalvataggio::Errore(_)) || app.ordine_annullabile()
}

/// Computer: c'e' un messaggio da mostrare nel riquadro?
fn ha_qualcosa_computer(app: &Applicazione) -> bool {
    matches!(app.salvataggio, StatoSalvataggio::Errore(_))
        || app.cancellazione_annullabile().is_some()
        || app.ordine_annullabile()
        || messaggio_rete(app).is_some()
}

/// Il messaggio della rete, finche' e' recente.
fn messaggio_rete(app: &Applicazione) -> Option<&str> {
    let recente = app.rete.messaggio_da.is_some_and(|da| da.elapsed() < DURATA_MESSAGGIO_RETE);
    app.rete.messaggio.as_deref().filter(|_| recente)
}

/// Righe, parole e (se la nota ha una lista) voci spuntate su voci in tutto.
pub struct Conteggi {
    pub righe: usize,
    pub parole: usize,
    pub spunte: Option<(usize, usize)>,
}

pub fn conteggi(nota: &crate::modello::Nota) -> Conteggi {
    // Le righe contano anche quelle vuote, come in un editor.
    let righe = if nota.testo.is_empty() { 0 } else { nota.testo.split('\n').count() };
    let voci = crate::modello::lista::voci(&nota.testo);
    let spunte = (!voci.is_empty()).then(|| (voci.iter().filter(|v| v.fatta).count(), voci.len()));
    Conteggi { righe, parole: nota.conta_parole(), spunte }
}

/// Computer: "Righe: 12 · Parole: 85", e se la nota ha una lista "· Spunte:
/// 2 di 5", in grigio, a destra nella riga della data (sotto Apri widget e
/// Cancella). Se lo spazio non basta si accorcia con "…".
pub fn contatori(ui: &mut Ui, app: &Applicazione, t: &Tavolozza, id: &str) {
    let Some(nota) = app.nota(id) else { return };
    let c = conteggi(nota);
    let mut conteggio = format!(
        "{} · {}",
        app.tr.f("contatore_righe", &[("n", &c.righe.to_string())]),
        app.tr.f("contatore_parole", &[("n", &c.parole.to_string())])
    );
    if let Some((fatte, tutte)) = c.spunte {
        conteggio.push_str(" · ");
        conteggio.push_str(&app.tr.f("voci_fatte", &[("fatte", &fatte.to_string()), ("voci", &tutte.to_string())]));
    }
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.add(egui::Label::new(RichText::new(conteggio).color(t.attenuato).size(TESTO_SECONDARIO)).truncate().selectable(false));
    });
}

/// Computer: il riquadro dei messaggi, in basso al centro di `nota`, con il
/// fondo sopra `sopra` (per non coprire ☐).
pub fn avvisi(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza, nota: Rect, sopra: f32) {
    if !ha_qualcosa_computer(app) {
        return;
    }
    // I messaggi a tempo spariscono da soli: serve un disegno anche senza mouse.
    ctx.request_repaint_after(std::time::Duration::from_millis(500));
    egui::Area::new(egui::Id::unique("avvisi_computer"))
        .order(egui::Order::Foreground)
        .pivot(Align2::CENTER_BOTTOM)
        .fixed_pos(egui::pos2(nota.center().x, sopra))
        .constrain_to(nota)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(t.superficie)
                .stroke(t.tratto())
                .corner_radius(RAGGIO_CONTENITORI)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.set_max_width(nota.width() - 40.0);
                    ui.horizontal(|ui| messaggi(ui, app, t));
                });
        });
}

/// Errore di salvataggio, "Annulla" dopo Cancella o dopo aver preso l'ordine,
/// note arrivate dalla rete.
fn messaggi(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let mut primo = true;
    let mut separa = |ui: &mut Ui| {
        if !std::mem::take(&mut primo) {
            ui.label(RichText::new("·").color(t.attenuato).size(TESTO_SECONDARIO));
        }
    };
    if let StatoSalvataggio::Errore(dettaglio) = &app.salvataggio {
        separa(ui);
        let dettaglio = dettaglio.clone();
        ui.label(RichText::new(app.tr.t("errore_salvataggio")).color(t.pericolo).size(TESTO_SECONDARIO)).on_hover_text(dettaglio);
    }
    let telefono = crate::interfaccia::schermo::telefono(ui.ctx());
    let annulla = |ui: &mut Ui, aiuto: &str| {
        // Come gli altri pulsanti dell'azione principale.
        crate::interfaccia::componenti::pulsante_accento(ui, t, app.tr.t("annulla")).on_hover_text(aiuto).clicked()
    };
    if app.cancellazione_annullabile().is_some() && !telefono {
        // Dopo Cancella, per qualche secondo: "Nota spostata nel cestino · Annulla".
        separa(ui);
        ui.label(RichText::new(app.tr.t("nota_cestinata")).color(t.testo).size(TESTO_SECONDARIO));
        if annulla(ui, app.tr.t("aiuto_annulla_cestino")) {
            app.annulla_cancellazione();
        }
    } else if app.ordine_annullabile() {
        // Dopo aver preso l'ordine delle schede da un altro computer.
        separa(ui);
        ui.label(RichText::new(app.tr.t("ordine_aggiornato")).color(t.testo).size(TESTO_SECONDARIO));
        if annulla(ui, app.tr.t("aiuto_annulla_ordine")) {
            app.annulla_ordine();
        }
    } else if let Some(messaggio) = messaggio_rete(app).map(str::to_owned) {
        separa(ui);
        ui.add(egui::Label::new(RichText::new(messaggio).color(t.testo).size(TESTO_SECONDARIO)).truncate());
    }
}
