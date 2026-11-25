//! Widget fluttuanti: una nota sul desktop in una finestra senza cornice.
//!
//! - trascina la barra superiore per spostarlo (doppio clic: riduci/espandi);
//! - il titolo della nota e' nella barra e si modifica con un clic;
//! - pallino colorato: colore della nota, che tinge tutto il widget;
//! - pallino pieno/vuoto: sempre in primo piano attivo/disattivo;
//! - trattino: riduce il widget alla sola barra;
//! - croce: chiude il widget, la nota resta;
//! - angolo in basso a destra: ridimensiona;
//! - "Note": apre la finestra principale su questa nota.
//!
//! Il testo e' lo stesso oggetto mostrato nella finestra principale:
//! scrivere da una parte aggiorna subito l'altra.

use egui::{
    Align, CentralPanel, Frame, Id, Layout, Margin, ResizeDirection, RichText, ScrollArea, Sense, TextEdit, Ui,
    ViewportBuilder, ViewportCommand, ViewportId, WindowLevel, vec2,
};

use super::componenti::{self, Icona, pulsante, pulsante_icona};
use super::icona;
use super::tema::{self, Tavolozza};
use super::testo_nota;
use crate::applicazione::{
    ALTEZZA_MINIMA_WIDGET, ALTEZZA_WIDGET_RIDOTTO, Applicazione, LARGHEZZA_MINIMA_WIDGET, StatoSalvataggio,
};

pub fn mostra_tutti(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza) {
    let aperti: Vec<String> = app.widget.iter().map(|w| w.stato.id_nota.clone()).collect();
    for id in aperti {
        let Some(widget) = app.widget.iter().find(|w| w.stato.id_nota == id) else { continue };
        let Some(nota) = app.nota(&id) else {
            app.chiudi_widget(&id);
            continue;
        };
        let titolo = format!("{} · {}", crate::NOME_APP, nota.titolo_visibile(app.tr.t("senza_titolo")));
        let [x, y, larghezza, altezza] = widget.geometria_iniziale;
        let livello = if widget.stato.in_primo_piano { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal };
        let costruttore = ViewportBuilder::default()
            .with_title(titolo)
            .with_app_id("schedario")
            .with_icon(icona::icona())
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(true)
            .with_taskbar(false)
            .with_position([x, y])
            .with_inner_size([larghezza, altezza])
            .with_min_inner_size([LARGHEZZA_MINIMA_WIDGET, ALTEZZA_WIDGET_RIDOTTO])
            // Niente `with_active` legato allo stato: se cambiasse da un
            // fotogramma all'altro, egui toglierebbe il fuoco alla finestra.
            .with_window_level(livello);

        let id_finestra = ViewportId::from_hash_of(("widget", id.as_str()));
        ctx.show_viewport_immediate(id_finestra, costruttore, |ui, _classe| {
            // Le descrizioni vanno a capo entro la larghezza del widget, che
            // spesso e' piu' stretto della finestra principale: altrimenti
            // uscirebbero dal bordo e verrebbero tagliate.
            let larghezza = (ui.max_rect().width() - 24.0).clamp(160.0, tema::LARGHEZZA_DESCRIZIONI);
            ui.ctx().all_styles_mut(|s| s.spacing.tooltip_width = larghezza);
            contenuto(ui, app, t, &id);
            ui.ctx().all_styles_mut(|s| s.spacing.tooltip_width = tema::LARGHEZZA_DESCRIZIONI);
        });
    }
}

fn contenuto(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza, id: &str) {
    let ctx = ui.ctx().clone();
    if ctx.input(|i| i.viewport().close_requested()) {
        app.chiudi_widget(id);
        return;
    }
    ricorda_geometria(&ctx, app, id);
    // Ctrl + rotella e Ctrl +/- funzionano anche dentro i widget.
    super::programma::gestisci_dimensione_testo(&ctx, app);
    let Some(indice) = app.widget.iter().position(|w| w.stato.id_nota == id) else { return };
    let ridotto = app.widget[indice].stato.ridotto;
    let in_primo_piano = app.widget[indice].stato.in_primo_piano;

    let mut chiudi = false;
    let mut alterna_riduzione = false;
    let mut alterna_primo_piano = false;
    let mut apri_note = false;
    let mut cambiato = false;
    // Il colore della nota tinge tutto il widget, per riconoscerlo sul desktop.
    let mut colore = app.nota(id).map(|n| n.colore.clone()).unwrap_or_default();
    let tinta = tema::tinta_nota(&colore, t.scuro);
    let (fondo, contorno) = match tinta {
        Some(tinta) => (tinta.fondo, egui::Stroke::new(1.5, tinta.segno)),
        None => (t.pannello, t.tratto()),
    };

    CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
        Frame::new()
            .fill(fondo)
            .stroke(contorno)
            .corner_radius(tema::RAGGIO_CONTENITORI)
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());

                // Barra superiore: area di trascinamento sotto, pulsanti sopra.
                let (barra, _) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::hover());
                let trascina = ui.interact(barra, Id::unique(("trascina", id)), Sense::click_and_drag());
                // Titolo e pulsanti stanno sopra l'area di trascinamento: la sua
                // descrizione compare solo fuori da loro, altrimenti coprirebbe
                // quella del pulsante sotto il mouse.
                let mut controlli: Vec<egui::Rect> = Vec::new();
                if trascina.drag_started() {
                    ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                }
                if trascina.double_clicked() {
                    alterna_riduzione = true;
                }
                // Il titolo della nota sta nella barra: si modifica con un clic.
                // Occupa solo la larghezza del testo, cosi' il resto della barra
                // (logo compreso) resta libero per trascinare il widget. Un
                // titolo lungo si accorcia con "…" finche' non lo si modifica.
                ui.scope_builder(egui::UiBuilder::new().max_rect(barra), |ui| {
                    ui.horizontal_centered(|ui| {
                        componenti::logo(ui, 20.0);
                        // Lo stesso carattere dei titoli nelle schede della panoramica.
                        let carattere = tema::carattere_titolo_piccolo();
                        if ridotto {
                            let titolo = app
                                .nota(id)
                                .map(|n| n.titolo_visibile(app.tr.t("senza_titolo")).to_owned())
                                .unwrap_or_default();
                            ui.add(
                                egui::Label::new(RichText::new(titolo).font(carattere).strong())
                                    .truncate()
                                    .selectable(false)
                                    .sense(Sense::hover()),
                            );
                        } else {
                            let suggerimento = app.tr.t("titolo_suggerimento").to_owned();
                            // Spazio per i quattro pulsanti a destra piu' un margine da trascinare.
                            let massimo = (ui.available_width() - (4.0 * 26.0 + 3.0 * 4.0 + 24.0)).max(40.0);
                            let id_campo = Id::unique(("widget_titolo", id));
                            let in_modifica = ui.memory(|m| m.has_focus(id_campo));
                            let titolo = app.nota(id).map(|n| n.titolo.clone()).unwrap_or_default();
                            if !in_modifica && !titolo.trim().is_empty() {
                                // Fuori dalla modifica: titolo intero o con "…", un clic per cambiarlo.
                                let etichetta = ui
                                    .allocate_ui(vec2(massimo, 26.0), |ui| {
                                        ui.add(
                                            egui::Label::new(RichText::new(titolo.trim()).font(carattere.clone()).strong())
                                                .truncate()
                                                .selectable(false)
                                                .sense(Sense::click()),
                                        )
                                    })
                                    .inner
                                    .on_hover_cursor(egui::CursorIcon::Text)
                                    .on_hover_text(app.tr.t("aiuto_titolo_widget"));
                                controlli.push(etichetta.rect);
                                if etichetta.clicked() {
                                    ui.memory_mut(|m| m.request_focus(id_campo));
                                }
                            } else {
                                let mut titolo = titolo;
                                let misura = if titolo.is_empty() { &suggerimento } else { &titolo };
                                let larghezza = ui
                                    .painter()
                                    .layout_no_wrap(misura.clone(), carattere.clone(), t.testo)
                                    .size()
                                    .x;
                                let prima = testo_nota::Prima::leggi(ui, id_campo);
                                let uscita = TextEdit::singleline(&mut titolo)
                                    .id(id_campo)
                                    .font(carattere)
                                    .hint_text(RichText::new(suggerimento).strong())
                                    .frame(Frame::NONE)
                                    .clip_text(true)
                                    .desired_width((larghezza + 8.0).clamp(40.0, massimo))
                                    .show(ui);
                                controlli.push(uscita.response.rect);
                                // Solo finche' il campo non e' in uso: mentre si scrive darebbe fastidio.
                                if !uscita.response.has_focus() {
                                    uscita.response.response.clone().on_hover_text(app.tr.t("aiuto_titolo_widget"));
                                }
                                let dal_menu = testo_nota::menu_campo(ui, &app.tr, id_campo, &uscita, &prima, &mut titolo);
                                if uscita.response.changed() || dal_menu {
                                    if let Some(nota) = app.nota_mut(id) {
                                        nota.titolo = titolo;
                                        cambiato = true;
                                    }
                                }
                            }
                        }
                        let pulsanti = ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            if pulsante_icona(ui, t, Icona::Chiudi, app.tr.t("chiudi_widget"), true).clicked() {
                                chiudi = true;
                            }
                            if pulsante_icona(ui, t, Icona::Riduci, app.tr.t("riduci"), true).clicked() {
                                alterna_riduzione = true;
                            }
                            let (icona, testo) = if in_primo_piano {
                                (Icona::PrimoPianoAttivo, app.tr.t("primo_piano_on"))
                            } else {
                                (Icona::PrimoPianoSpento, app.tr.t("primo_piano_off"))
                            };
                            if pulsante_icona(ui, t, icona, testo, true).clicked() {
                                alterna_primo_piano = true;
                            }
                            if componenti::scelta_colore(ui, t, &mut colore, &app.tr, None, 26.0) {
                                if let Some(nota) = app.nota_mut(id) {
                                    nota.colore = colore.clone();
                                    cambiato = true;
                                }
                            }
                            // Da destra a sinistra: l'area usata e' quella dei pulsanti.
                            ui.min_rect()
                        });
                        controlli.push(pulsanti.inner);
                    });
                });
                let su_controllo = ui
                    .input(|i| i.pointer.hover_pos())
                    .is_some_and(|p| controlli.iter().any(|r| r.contains(p)));
                if !su_controllo {
                    trascina.on_hover_text(app.tr.t("sposta"));
                }

                if ridotto {
                    return;
                }
                let suggerimento_testo = app.tr.t("testo_suggerimento").to_owned();
                let etichetta_note = app.tr.t("apri_note").to_owned();
                let aiuto_note = app.tr.t("aiuto_apri_note").to_owned();
                let testo_ridimensiona = app.tr.t("ridimensiona").to_owned();
                // Come nella finestra principale: il salvataggio e' automatico,
                // "Salvato" non si scrive; si dice solo, in rosso, se non riesce.
                let errore = matches!(app.salvataggio, StatoSalvataggio::Errore(_)).then(|| app.tr.t("errore_salvataggio").to_owned());

                ui.separator();

                // Spazio per il testo: tutto tranne la riga inferiore.
                let altezza_testo = (ui.available_height() - 36.0).max(40.0);
                let id_testo = Id::unique(("widget_testo", id));
                if app.widget[indice].fuoco_testo > 0 {
                    // Nota rapida: la finestra viene in primo piano, pronta per
                    // scrivere. Si insiste finche' il sistema non le da' il fuoco.
                    // Nei primi fotogrammi egui puo' credere la finestra nuova gia'
                    // attiva anche se il sistema non l'ha ancora portata davanti:
                    // la richiesta parte comunque.
                    ui.memory_mut(|m| m.request_focus(id_testo));
                    let restanti = app.widget[indice].fuoco_testo - 1;
                    let attiva = ctx.input(|i| i.viewport().focused == Some(true));
                    app.widget[indice].fuoco_testo = if attiva && restanti < 25 { 0 } else { restanti };
                    if app.widget[indice].fuoco_testo > 0 {
                        ctx.send_viewport_cmd(ViewportCommand::Focus);
                        ctx.request_repaint_after(std::time::Duration::from_millis(50));
                    }
                }
                let mut testo = app.nota(id).map(|n| n.testo.clone()).unwrap_or_default();
                let carattere = tema::carattere_nota(app.preferenze.dimensione_testo);
                ScrollArea::vertical()
                    .id_salt(("widget_scorrimento", id))
                    .max_height(altezza_testo)
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        let modificato = testo_nota::testo(
                            ui,
                            t,
                            &app.tr,
                            &mut testo,
                            id_testo,
                            carattere,
                            &suggerimento_testo,
                            true,
                            altezza_testo,
                            testo_nota::Evidenzia::default(),
                        );
                        if modificato {
                            if let Some(nota) = app.nota_mut(id) {
                                nota.testo = testo;
                                cambiato = true;
                            }
                        }
                    });

                ui.horizontal(|ui| {
                    if pulsante(ui, t, &etichetta_note).on_hover_text(&aiuto_note).clicked() {
                        apri_note = true;
                    }
                    if let Some(errore) = &errore {
                        ui.label(RichText::new(errore).color(t.pericolo).size(tema::TESTO_SECONDARIO));
                    }
                    ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                        let maniglia = pulsante_icona(ui, t, Icona::Ridimensiona, &testo_ridimensiona, true)
                            .interact(Sense::drag())
                            .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
                        if maniglia.drag_started() {
                            ctx.send_viewport_cmd(ViewportCommand::BeginResize(ResizeDirection::SouthEast));
                        }
                    });
                });
            });
    });

    if cambiato {
        app.registra_modifica(id);
    }
    if apri_note {
        app.mostra_nota(id);
    }
    if alterna_primo_piano {
        let w = &mut app.widget[indice];
        w.stato.in_primo_piano = !w.stato.in_primo_piano;
        let livello = if w.stato.in_primo_piano { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal };
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(livello));
    }
    if alterna_riduzione {
        let w = &mut app.widget[indice];
        let larghezza = w.stato.larghezza;
        if w.stato.ridotto {
            ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(LARGHEZZA_MINIMA_WIDGET, ALTEZZA_MINIMA_WIDGET)));
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(larghezza, w.altezza_espansa)));
            ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
        } else {
            w.altezza_espansa = w.stato.altezza;
            ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(LARGHEZZA_MINIMA_WIDGET, ALTEZZA_WIDGET_RIDOTTO)));
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(larghezza, ALTEZZA_WIDGET_RIDOTTO)));
        }
        w.stato.ridotto = !w.stato.ridotto;
    }
    if chiudi {
        app.chiudi_widget_e_scarta_se_vuota(id);
    }
}

/// Memorizza posizione e dimensione correnti, per ritrovarle al prossimo avvio.
fn ricorda_geometria(ctx: &egui::Context, app: &mut Applicazione, id: &str) {
    let (esterno, interno) = ctx.input(|i| (i.viewport().outer_rect, i.viewport().inner_rect));
    let Some(w) = app.widget.iter_mut().find(|w| w.stato.id_nota == id) else { return };
    if let Some(esterno) = esterno {
        w.stato.x = esterno.min.x;
        w.stato.y = esterno.min.y;
    }
    if let Some(interno) = interno {
        w.stato.larghezza = interno.width();
        if !w.stato.ridotto && interno.height() > ALTEZZA_WIDGET_RIDOTTO + 20.0 {
            w.stato.altezza = interno.height();
            w.altezza_espansa = interno.height();
        }
    }
}
