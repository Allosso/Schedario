//! Telefono: una barra sottile appena sopra la tastiera, mentre si scrive.
//!
//! ```text
//! │ [↶|↷]                                          [⌄] │
//! ├───────────────────── tastiera ───────────────────┤
//! ```
//!
//! - ↶ annulla l'ultima modifica, ↷ la ripete (sul telefono non c'e' Ctrl+Z);
//! - ⌄ chiude la tastiera.
//!
//! La voce di lista e' il pulsante tondo ☐ sulla nota (`editor`).
//!
//! Mentre si scrive i pulsanti in cima alla nota scorrono via; questi restano
//! sotto il pollice. Toccarli non toglie il fuoco al campo
//! (`schermo::proteggi_fuoco`).

use egui::{Align, Frame, Layout, Margin, Panel, Ui};

use crate::applicazione::Applicazione;
use crate::interfaccia::componenti::{Icona, Simbolo, Strumento, barra_strumenti, pulsante_icona_lato};
use crate::interfaccia::schermo;
use crate::interfaccia::tastiera::{self, AzioneTesto};
use crate::interfaccia::tema::Tavolozza;

const LATO: f32 = crate::interfaccia::componenti::LATO_ICONA;

/// La tastiera del telefono e' aperta?
pub fn tastiera_aperta() -> bool {
    #[cfg(target_os = "android")]
    return crate::piattaforma::android::tastiera_aperta();
    #[cfg(not(target_os = "android"))]
    false
}

/// Mostra la barra se la tastiera e' aperta su un campo di testo.
pub fn mostra(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    if !schermo::TOCCO || !tastiera_aperta() {
        return;
    }
    let Some(campo) = tastiera::campo_precedente(ui.ctx()) else { return };
    let ctx = ui.ctx().clone();
    let risposta = Panel::bottom("barra_tastiera")
        .frame(Frame::new().fill(t.pannello).stroke(t.tratto()).inner_margin(Margin::symmetric(8, 3)))
        .show_separator_line(false)
        .show(ui, |ui| {
            // La zona da trascinare va sotto i pulsanti (registrata prima di
            // loro), cosi' i tocchi su Annulla, Ripeti e chiudi arrivano a loro.
            let trascina = ui.interact(ui.max_rect(), egui::Id::unique("barra_tastiera_trascina"), egui::Sense::drag());
            ui.horizontal(|ui| {
                // Annulla e Ripeti in un solo blocco. (La voce di lista e' il
                // pulsante tondo sulla nota; Fissa e Colore sono sotto il titolo.)
                let storia = [
                    Strumento {
                        simbolo: Simbolo::Icona(Icona::Annulla),
                        testo: None,
                        descrizione: app.tr.t("aiuto_annulla_testo"),
                        attivo: tastiera::si_puo(&ctx, campo.id, &campo.testo, false),
                    },
                    Strumento {
                        simbolo: Simbolo::Icona(Icona::Ripeti),
                        testo: None,
                        descrizione: app.tr.t("aiuto_ripeti_testo"),
                        attivo: tastiera::si_puo(&ctx, campo.id, &campo.testo, true),
                    },
                ];
                let risposte = barra_strumenti(ui, t, egui::Id::unique("barra_tastiera_storia"), &storia, LATO);
                if risposte[0].clicked() {
                    tastiera::richiedi(&ctx, AzioneTesto::Annulla);
                }
                if risposte[1].clicked() {
                    tastiera::richiedi(&ctx, AzioneTesto::Ripeti);
                }
                // A destra chiudi tastiera, come sulla tastiera del telefono.
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if pulsante_icona_lato(ui, t, Icona::Giu, app.tr.t("aiuto_chiudi_tastiera"), true, LATO).clicked() {
                        ctx.memory_mut(|m| m.surrender_focus(campo.id));
                    }
                });
            });
            trascina
        });
    let barra = risposta.response.rect;
    let trascina = risposta.inner;
    // Toccare la barra non toglie il fuoco al campo in cui si scrive.
    schermo::proteggi_fuoco(&ctx, barra);
    // Una lineetta al centro dice che la barra si trascina: tirata verso il
    // basso chiude la tastiera. Il gesto veloce lo riconosce
    // `Programma::smista_gesti`; qui quello lento (dito tenuto e poi mosso).
    schermo::imposta_zona_barra_tastiera(&ctx, Some(barra));
    // La stessa lineetta della maniglia della panoramica.
    crate::interfaccia::componenti::disegna_lineetta(ui.painter(), egui::pos2(barra.center().x, barra.top() + 6.0), t, false);
    let giu = ui.input(|i| match (i.pointer.press_origin(), i.pointer.latest_pos()) {
        (Some(o), Some(p)) => barra.contains(o) && p.y - o.y > 30.0,
        _ => false,
    });
    if trascina.dragged() && giu {
        ctx.memory_mut(|m| m.surrender_focus(campo.id));
    }
}
