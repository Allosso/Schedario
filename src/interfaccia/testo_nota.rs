//! Campi di testo delle note, uguali nell'editor e nei widget.
//!
//! - **Liste di controllo**: le righe `[ ]` / `[x]` mostrano una casella da
//!   cliccare (le regole sono in `modello::lista`). Il segno nel testo resta,
//!   ma e' trasparente: sopra viene disegnata la casella. Le voci fatte sono
//!   barrate e in grigio. Invio su una voce ne crea un'altra; Ctrl+L
//!   trasforma la riga in voce (o la riporta a testo normale).
//! - **Menu del tasto destro**: Taglia, Copia, Incolla, Seleziona tutto e,
//!   nel testo, Voce di lista.
//!
//! Il menu non usa librerie in piu': chiede al sistema le stesse azioni delle
//! scorciatoie (Ctrl+X, Ctrl+C, Ctrl+V).

use egui::text::{CCursor, CCursorRange, LayoutJob, TextFormat};
use egui::text_edit::{TextEditOutput, TextEditState};
use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Frame, Id, Key, Rect, Sense, Stroke, TextEdit, Ui, Vec2, pos2, vec2,
};
#[cfg(not(target_os = "android"))]
use egui::ViewportCommand;

use super::schermo::{TOCCO, misura};
use super::tema::Tavolozza;
use crate::lingue::Traduttore;
use crate::modello::lista;

/// Parole cercate da evidenziare nel testo.
#[derive(Clone, Copy, Default)]
pub struct Evidenzia<'a> {
    /// Il testo della ricerca (vuoto: nessuna evidenza).
    pub parole: &'a str,
    /// Quale delle parole trovate e' quella attuale (le frecce ‹ ›): ha un
    /// colore piu' forte.
    pub attuale: Option<usize>,
    /// Porta in vista la parola attuale (appena scelta con le frecce).
    pub mostra_attuale: bool,
}

/// Il testo di una nota, con liste di controllo e menu del tasto destro.
/// Restituisce `true` se il testo e' cambiato.
#[allow(clippy::too_many_arguments)]
pub fn testo(
    ui: &mut Ui,
    t: &Tavolozza,
    tr: &Traduttore,
    testo: &mut String,
    id: Id,
    carattere: FontId,
    suggerimento: &str,
    abilitato: bool,
    altezza_minima: f32,
    evidenzia: Evidenzia,
) -> bool {
    let mut cambiato = false;
    let ctx = ui.ctx().clone();

    // Ctrl+L sulla riga del cursore, prima che il campo veda il tasto.
    let ha_fuoco = ui.memory(|m| m.has_focus(id));
    if abilitato && ha_fuoco && ui.input_mut(|i| super::programma::consuma_comando(i, Key::L)) {
        cambiato |= alterna_riga(&ctx, id, testo);
    }
    // Backspace sul segno di una voce ("[ ] ", "[x] "): via tutto il segno,
    // non una parentesi alla volta.
    if abilitato && ha_fuoco {
        cambiato |= cancella_segno_voce(ui, id, testo);
    }

    let prima = Prima::leggi(ui, id);
    let colori = Colori {
        testo: t.testo,
        attenuato: t.attenuato,
        evidenzia: t.evidenzia,
        attuale: t.accento,
        testo_attuale: t.testo_accento,
    };
    let carattere_disposizione = carattere.clone();
    let ricerca = evidenzia.parole.to_owned();
    let attuale = evidenzia.attuale;
    let mut disposizione = move |ui: &Ui, buffer: &dyn egui::TextBuffer, larghezza: f32| {
        // Quanto la "x" di una voce fatta e' piu' larga dello spazio di una da fare.
        let piu_larga = ui.fonts_mut(|f| {
            let spazio = f.glyph_width(&carattere_disposizione, ' ');
            [f.glyph_width(&carattere_disposizione, 'x') - spazio, f.glyph_width(&carattere_disposizione, 'X') - spazio]
        });
        let lavoro = componi(buffer.as_str(), &carattere_disposizione, colori, (&ricerca, attuale), larghezza, piu_larga);
        ui.fonts_mut(|f| f.layout_job(lavoro))
    };
    let uscita = ui
        .add_enabled_ui(abilitato, |ui| {
            TextEdit::multiline(testo)
                .id(id)
                .font(carattere.clone())
                .hint_text(suggerimento)
                .frame(Frame::NONE)
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .min_size(vec2(0.0, altezza_minima))
                .layouter(&mut disposizione)
                .show(ui)
        })
        .inner;
    cambiato |= uscita.response.changed();

    // La parola trovata scelta con le frecce: la nota scorre fino a lei.
    if evidenzia.mostra_attuale {
        if let Some(parte) = evidenzia.attuale.and_then(|i| crate::modello::nota::Nota::occorrenze(testo, evidenzia.parole).get(i).cloned()) {
            let carattere = testo[..parte.start].chars().count();
            let posto = uscita.galley.pos_from_cursor(CCursor::new(carattere)).translate(uscita.galley_pos.to_vec2());
            ui.scroll_to_rect(posto.expand(24.0), Some(egui::Align::Center));
        }
    }

    // Invio su una voce di lista: la riga nuova e' un'altra voce.
    // (Lo stato per la tastiera del telefono si legge dopo, a testo sistemato.)
    let invio = ui.input(|i| i.key_pressed(Key::Enter) && !i.modifiers.shift);
    if invio && uscita.response.changed() {
        if let Some(cursore) = uscita.cursor_range.map(|r| r.primary.index.0) {
            if let Some(nuovo) = lista::continua_dopo_invio(testo, cursore) {
                sposta_cursore(&ctx, id, nuovo);
            }
        }
    }

    cambiato |= caselle(ui, t, &uscita, testo, &carattere, abilitato);
    if TOCCO && abilitato {
        // Dove si vede il testo: li' il dito trascinando fa scorrere.
        super::schermo::imposta_zona_testo(&ctx, Some(uscita.response.rect.intersect(ui.clip_rect())));
    }
    if abilitato {
        cambiato |= menu(ui, tr, id, &uscita, &prima, testo, true);
    }
    if TOCCO && abilitato {
        maniglie(ui, t, id, &uscita);
    }
    super::tastiera::campo(ui, &uscita, testo, true, false);
    cambiato
}

/// Menu del tasto destro per un campo gia' mostrato (per esempio un titolo).
/// Da usare con `Prima::leggi` chiamato prima di mostrare il campo.
/// Restituisce `true` se il testo e' cambiato (Taglia, Incolla).
pub fn menu_campo(ui: &mut Ui, tr: &Traduttore, id: Id, uscita: &TextEditOutput, prima: &Prima, testo: &mut String) -> bool {
    menu(ui, tr, id, uscita, prima, testo, false)
}

/// Stato del campo prima di disegnarlo: serve a non perdere la selezione con
/// il tasto destro (egui sposterebbe il cursore dove si clicca).
pub struct Prima {
    selezione: Option<CCursorRange>,
    destro: bool,
}

impl Prima {
    pub fn leggi(ui: &Ui, id: Id) -> Self {
        let selezione = TextEditState::load(ui.ctx(), id).and_then(|s| s.cursor.char_range());
        // Sul telefono il menu si apre tenendo premuto, ma il cursore si sposta
        // gia' quando il dito tocca: la selezione di quel momento si mette da parte.
        if ui.input(|i| i.pointer.primary_pressed() && i.any_touches()) {
            ui.ctx().data_mut(|d| d.insert_temp(chiave_tocco(id), selezione));
        }
        Self { selezione, destro: ui.input(|i| i.pointer.secondary_pressed()) }
    }
}

fn chiave_tocco(id: Id) -> Id {
    id.with("selezione_al_tocco")
}

/// Pressione lunga: la selezione su cui lavora il menu. Se si tiene premuto
/// dentro il testo gia' selezionato resta quello; altrimenti si seleziona la
/// parola sotto il dito (in "La città è bella" tenendo premuto su "città" si
/// seleziona "città"). Nessuna selezione se sotto il dito c'e' uno spazio.
fn selezione_tocco(ctx: &egui::Context, id: Id, uscita: &TextEditOutput, testo: &str) -> Option<CCursorRange> {
    // Il carattere sotto il dito (il campo puo' non avere ancora il cursore).
    let punto = ctx.input(|i| i.pointer.interact_pos())?;
    let cursore = uscita.galley.cursor_from_pos(punto - uscita.galley_pos).index.0;
    let prima: Option<CCursorRange> = ctx.data(|d| d.get_temp::<Option<CCursorRange>>(chiave_tocco(id))).flatten();
    if let Some(s) = prima.filter(|s| !s.is_empty()) {
        let [x, y] = [s.primary.index.0, s.secondary.index.0];
        if (x.min(y)..=x.max(y)).contains(&cursore) {
            return Some(s);
        }
    }
    let (da, a) = parola_attorno(testo, cursore)?;
    Some(CCursorRange::two(CCursor::new(da), CCursor::new(a)))
}

/// La parola che contiene la posizione `cursore` (in caratteri), o che finisce li'.
fn parola_attorno(testo: &str, cursore: usize) -> Option<(usize, usize)> {
    let caratteri: Vec<char> = testo.chars().collect();
    let di_parola = |c: char| c.is_alphanumeric() || c == '_' || c == '\'';
    let cursore = cursore.min(caratteri.len());
    let mut da = cursore;
    while da > 0 && di_parola(caratteri[da - 1]) {
        da -= 1;
    }
    let mut a = cursore;
    while a < caratteri.len() && di_parola(caratteri[a]) {
        a += 1;
    }
    (da < a).then_some((da, a))
}

#[derive(Clone, Copy)]
enum Azione {
    Taglia,
    Copia,
    Incolla,
    SelezionaTutto,
    VoceLista,
    Annulla,
    Ripeti,
}

fn menu(ui: &mut Ui, tr: &Traduttore, id: Id, uscita: &TextEditOutput, prima: &Prima, testo: &mut String, lista: bool) -> bool {
    let ctx = ui.ctx().clone();
    // La selezione al momento del clic destro: il menu lavora su quella.
    // (Il clic su una voce del menu arriva anche al campo sotto, che
    // altrimenti sposterebbe il cursore e perderebbe la selezione.)
    let chiave = id.with("selezione_menu");
    // Sul telefono il menu si apre tenendo premuto.
    let tocco = uscita.response.long_touched();
    let apre = (prima.destro && uscita.response.hovered()) || tocco;
    if apre {
        let selezione = if tocco {
            selezione_tocco(&ctx, id, uscita, testo)
        } else {
            prima.selezione.filter(|s| !s.is_empty())
        };
        ctx.data_mut(|d| d.insert_temp(chiave, selezione));
        // Dove si e' aperto il menu: sul telefono lo si sposta poi nello spazio libero.
        let punto = ctx.input(|i| i.pointer.interact_pos());
        ctx.data_mut(|d| d.insert_temp(id.with("punto_menu"), punto));
        if let Some(selezione) = selezione {
            // Il clic destro dentro una selezione la conserva, come in ogni editor.
            let mut stato = uscita.state.clone();
            stato.cursor.set_char_range(Some(selezione));
            stato.store(&ctx, id);
            if tocco {
                // Cosi' la parola scelta si vede evidenziata.
                ui.memory_mut(|m| m.request_focus(id));
            }
        }
    }
    let salvata: Option<CCursorRange> = ctx.data(|d| d.get_temp::<Option<CCursorRange>>(chiave)).flatten();
    let selezione = salvata.is_some();
    let mut azione = None;
    // Mentre il dito trascina una maniglia della selezione il menu non c'e':
    // torna sopra la selezione nuova quando il dito si alza (`maniglie`).
    if TOCCO && ctx.data(|d| d.get_temp::<usize>(chiave_presa(id))).is_some() {
        return false;
    }
    let mut finestra = egui::Popup::context_menu(&uscita.response);
    let id_altro = id.with("menu_altre_voci");
    if apre {
        ctx.data_mut(|d| d.insert_temp(id_altro, false));
    }
    if TOCCO {
        // Telefono: una striscia piccola sopra la parola scelta, cosi' resta
        // visibile; sotto, se sopra non c'e' posto. Sempre nello spazio libero
        // sopra la tastiera (egui la terrebbe solo dentro lo schermo intero).
        // Le voci si scelgono con un tocco: la striscia si chiude da se'.
        finestra = finestra.close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        let punto: Option<egui::Pos2> = ctx.data(|d| d.get_temp::<Option<egui::Pos2>>(id.with("punto_menu"))).flatten();
        if let Some(p) = punto {
            let dimensione = finestra.get_expected_size().unwrap_or(vec2(280.0, 56.0));
            let libera = super::schermo::area_libera(&ctx);
            let sopra = p.y - 46.0 - dimensione.y;
            let y = if sopra >= libera.top() + 4.0 { sopra } else { (p.y + 40.0).min(libera.bottom() - 4.0 - dimensione.y) };
            let x = (p.x - dimensione.x / 2.0).clamp(libera.left() + 4.0, (libera.right() - 4.0 - dimensione.x).max(libera.left() + 4.0));
            finestra = finestra.at_position(egui::pos2(x, y)).align(egui::RectAlign::BOTTOM_START).align_alternatives(&[]);
        }
    }
    let mostrata = finestra.show(|ui| {
        let tasto = if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" };
        let si_annulla = super::tastiera::si_puo(&ctx, id, testo, false);
        let si_ripete = super::tastiera::si_puo(&ctx, id, testo, true);
        if TOCCO {
            // Taglia · Copia · Incolla · … ; "…" mostra sotto le altre voci.
            let altre: bool = ctx.data(|d| d.get_temp(id_altro)).unwrap_or(false);
            let mut voce = |ui: &mut Ui, testo_voce: &str, attiva: bool, a: Azione| {
                let pulsante = egui::Button::new(testo_voce).min_size(vec2(0.0, super::componenti::ALTEZZA_PULSANTE));
                if ui.add_enabled(attiva, pulsante).clicked() {
                    azione = Some(a);
                    ui.close();
                }
            };
            ui.horizontal(|ui| {
                voce(ui, tr.t("menu_taglia"), selezione, Azione::Taglia);
                voce(ui, tr.t("menu_copia"), selezione, Azione::Copia);
                voce(ui, tr.t("menu_incolla"), true, Azione::Incolla);
                let tre_punti = egui::Button::selectable(altre, "…").min_size(vec2(44.0, super::componenti::ALTEZZA_PULSANTE));
                if ui.add(tre_punti).on_hover_text(tr.t("aiuto_altre_voci")).clicked() {
                    ctx.data_mut(|d| d.insert_temp(id_altro, !altre));
                }
            });
            if altre {
                ui.horizontal_wrapped(|ui| {
                    voce(ui, tr.t("menu_seleziona_tutto"), !testo.is_empty(), Azione::SelezionaTutto);
                    voce(ui, tr.t("menu_annulla"), si_annulla, Azione::Annulla);
                    voce(ui, tr.t("menu_ripeti"), si_ripete, Azione::Ripeti);
                    if lista {
                        voce(ui, tr.t("menu_voce_lista_breve"), true, Azione::VoceLista);
                    }
                });
            }
            return;
        }
        ui.set_min_width(170.0);
        let mut voce = |ui: &mut Ui, chiave: &str, scorciatoia: &str, attiva: bool, a: Azione| {
            let pulsante = egui::Button::new(tr.t(chiave)).shortcut_text(scorciatoia);
            if ui.add_enabled(attiva, pulsante).clicked() {
                azione = Some(a);
            }
        };
        voce(ui, "menu_annulla", &format!("{tasto}+Z"), si_annulla, Azione::Annulla);
        voce(ui, "menu_ripeti", &format!("{tasto}+Y"), si_ripete, Azione::Ripeti);
        ui.separator();
        voce(ui, "menu_taglia", &format!("{tasto}+X"), selezione, Azione::Taglia);
        voce(ui, "menu_copia", &format!("{tasto}+C"), selezione, Azione::Copia);
        voce(ui, "menu_incolla", &format!("{tasto}+V"), true, Azione::Incolla);
        voce(ui, "menu_seleziona_tutto", &format!("{tasto}+A"), !testo.is_empty(), Azione::SelezionaTutto);
        if lista {
            ui.separator();
            voce(ui, "menu_voce_lista", &format!("{tasto}+L"), true, Azione::VoceLista);
        }
    });
    // Toccare il menu (anche "…") non toglie il fuoco al testo: la tastiera resta.
    if let Some(menu) = mostrata {
        super::schermo::proteggi_fuoco(&ctx, menu.response.rect);
    }
    let Some(azione) = azione else { return false };
    // Cliccare nel menu toglie il fuoco al campo: lo restituiamo.
    ui.memory_mut(|m| m.request_focus(id));
    ctx.request_repaint();
    ctx.data_mut(|d| d.remove::<Option<CCursorRange>>(chiave));
    // Taglia e Copia lavorano direttamente sulla selezione salvata; Incolla
    // chiede al sistema il contenuto degli appunti (arriva al campo con il fuoco).
    match azione {
        Azione::Taglia | Azione::Copia => {
            let Some(selezione) = salvata else { return false };
            let [da, a] = [selezione.primary.index.0, selezione.secondary.index.0];
            let (da, a) = (da.min(a), da.max(a).min(testo.chars().count()));
            let pezzo: String = testo.chars().skip(da).take(a.saturating_sub(da)).collect();
            ctx.copy_text(pezzo);
            let mut stato = TextEditState::load(&ctx, id).unwrap_or_default();
            if matches!(azione, Azione::Taglia) {
                let byte = |c: usize| testo.char_indices().nth(c).map(|(b, _)| b).unwrap_or(testo.len());
                let (inizio, fine) = (byte(da), byte(a));
                testo.replace_range(inizio..fine, "");
                stato.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(da))));
                stato.store(&ctx, id);
                return true;
            }
            // Dopo Copia il testo resta selezionato.
            stato.cursor.set_char_range(Some(selezione));
            stato.store(&ctx, id);
        }
        #[cfg(not(target_os = "android"))]
        Azione::Incolla => ctx.send_viewport_cmd(ViewportCommand::RequestPaste),
        // Su Android gli appunti si leggono dalla parte Java e il testo si
        // inserisce qui, al posto della selezione.
        #[cfg(target_os = "android")]
        Azione::Incolla => {
            let Some(mut appunti) = crate::piattaforma::android::leggi_appunti() else { return false };
            if !lista {
                appunti = appunti.replace(['\r', '\n'], " ");
            }
            let selezione = salvata.or_else(|| TextEditState::load(&ctx, id).and_then(|s| s.cursor.char_range()));
            let fine_testo = testo.chars().count();
            let (da, a) = selezione
                .map(|s| {
                    let [x, y] = [s.primary.index.0, s.secondary.index.0];
                    (x.min(y).min(fine_testo), x.max(y).min(fine_testo))
                })
                .unwrap_or((fine_testo, fine_testo));
            let byte = |c: usize| testo.char_indices().nth(c).map(|(b, _)| b).unwrap_or(testo.len());
            let (inizio, fine) = (byte(da), byte(a));
            testo.replace_range(inizio..fine, &appunti);
            sposta_cursore(&ctx, id, da + appunti.chars().count());
            return true;
        }
        Azione::SelezionaTutto => {
            let mut stato = TextEditState::load(&ctx, id).unwrap_or_default();
            stato.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(testo.chars().count()))));
            stato.store(&ctx, id);
        }
        Azione::VoceLista => return alterna_riga(&ctx, id, testo),
        // Arrivano al campo al prossimo fotogramma, come Ctrl+Z e Ctrl+Y.
        Azione::Annulla => super::tastiera::richiedi(&ctx, super::tastiera::AzioneTesto::Annulla),
        Azione::Ripeti => super::tastiera::richiedi(&ctx, super::tastiera::AzioneTesto::Ripeti),
    }
    false
}

/// Raggio del tondo delle maniglie della selezione.
const RAGGIO_MANIGLIA: f32 = 9.0;

/// Mentre il dito trascina una maniglia: l'altro capo della selezione, che
/// resta fermo (in caratteri).
fn chiave_presa(id: Id) -> Id {
    id.with("maniglia_presa")
}

/// Telefono: le due maniglie della selezione, gocce nel colore principale
/// sotto l'inizio e la fine del testo selezionato, come nelle altre app di
/// Android. Trascinandole si allarga o si restringe la selezione, anche su
/// piu' righe; vicino al bordo dell'area il testo scorre. Per esempio in "La
/// città è bella", tenendo premuto su "città" e trascinando la maniglia di
/// destra fino a "bella", si seleziona "città è bella". Mentre si trascina il
/// menu Taglia/Copia/Incolla si nasconde; sollevando il dito ricompare sopra
/// la selezione nuova.
fn maniglie(ui: &mut Ui, t: &Tavolozza, id: Id, uscita: &TextEditOutput) {
    let ctx = ui.ctx().clone();
    if !ui.memory(|m| m.has_focus(id)) {
        ctx.data_mut(|d| d.remove::<usize>(chiave_presa(id)));
        return;
    }
    let Some(selezione) = TextEditState::load(&ctx, id).and_then(|s| s.cursor.char_range()) else { return };
    let presa: Option<usize> = ctx.data(|d| d.get_temp(chiave_presa(id)));
    if selezione.is_empty() && presa.is_none() {
        return;
    }
    let [x, y] = [selezione.primary.index.0, selezione.secondary.index.0];
    let (da, a) = (x.min(y), x.max(y));
    let origine = uscita.galley_pos;
    let posto = |c: usize| uscita.galley.pos_from_cursor(CCursor::new(c)).translate(origine.to_vec2());
    let r = RAGGIO_MANIGLIA;
    let clip = ui.clip_rect();
    // Di lato le gocce possono uscire dall'area del testo (una parola
    // all'inizio della riga): li' non si tagliano, sopra e sotto si'.
    let pittore = ctx.layer_painter(ui.layer_id()).with_clip_rect(clip.expand2(vec2(3.0 * r, 0.0)));
    let mut zone = super::schermo::maniglie_selezione(&ctx);
    let mut nuova = None;
    let mut finita = false;
    // L'inizio ha la goccia a sinistra, la fine a destra.
    for (n, (c, verso)) in [(da, -1.0_f32), (a, 1.0)].into_iter().enumerate() {
        let cursore = posto(c);
        let punta = pos2(cursore.left(), cursore.bottom());
        // Contro il bordo dello schermo la goccia si gira dall'altra parte.
        let schermo = super::schermo::area_libera(&ctx);
        let verso = if punta.x + verso * 2.0 * r < schermo.left() || punta.x + verso * 2.0 * r > schermo.right() {
            -verso
        } else {
            verso
        };
        let centro = punta + vec2(verso * r, r);
        let zona = Rect::from_center_size(centro, Vec2::splat(44.0));
        let risposta = ui.interact(zona, id.with(("maniglia_selezione", n)), Sense::drag());
        if clip.intersects(zona) {
            zone.push(zona.intersect(clip));
            super::schermo::proteggi_fuoco(&ctx, zona);
        }
        pittore.circle_filled(centro, r, t.accento);
        // L'angolo verso il testo e' dritto: la goccia punta al carattere.
        pittore.rect_filled(Rect::from_two_pos(punta, punta + vec2(verso * r, r)), 0.0, t.accento);
        if risposta.drag_started() {
            let fisso = if n == 0 { a } else { da };
            ctx.data_mut(|d| d.insert_temp(chiave_presa(id), fisso));
        }
        if risposta.dragged() {
            if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
                // Il dito tiene il tondo sotto la riga: conta il punto della riga sopra.
                let bersaglio = p - vec2(verso * r, r + cursore.height() / 2.0);
                nuova = Some(uscita.galley.cursor_from_pos(bersaglio - origine).index.0);
                if p.y > clip.bottom() - 24.0 {
                    ui.scroll_with_delta(vec2(0.0, -8.0));
                } else if p.y < clip.top() + 24.0 {
                    ui.scroll_with_delta(vec2(0.0, 8.0));
                }
            }
        }
        finita |= risposta.drag_stopped();
    }
    super::schermo::imposta_maniglie_selezione(&ctx, zone);
    let fisso: Option<usize> = ctx.data(|d| d.get_temp(chiave_presa(id)));
    if let (Some(c), Some(fisso)) = (nuova, fisso) {
        // Almeno un carattere: le due maniglie non si sovrappongono.
        if c != fisso {
            let mut stato = TextEditState::load(&ctx, id).unwrap_or_default();
            stato.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(fisso), CCursor::new(c))));
            stato.store(&ctx, id);
        }
        ctx.request_repaint();
    }
    if finita {
        ctx.data_mut(|d| d.remove::<usize>(chiave_presa(id)));
        // Il menu torna, sopra l'inizio della selezione nuova.
        let nuova = TextEditState::load(&ctx, id).and_then(|s| s.cursor.char_range()).filter(|s| !s.is_empty());
        if let Some(s) = nuova {
            let inizio = posto(s.primary.index.0.min(s.secondary.index.0));
            ctx.data_mut(|d| {
                d.insert_temp(id.with("selezione_menu"), Some(s));
                d.insert_temp(id.with("punto_menu"), Some(inizio.center()));
            });
            egui::Popup::open_id(&ctx, egui::Popup::default_response_id(&uscita.response));
        }
    }
}

/// Pulsante "voce di lista" fuori dal campo: agisce sulla riga dove era il
/// cursore e rimette il fuoco nel testo.
pub fn voce_lista(ctx: &egui::Context, id: Id, testo: &mut String) -> bool {
    ctx.memory_mut(|m| m.request_focus(id));
    alterna_riga(ctx, id, testo)
}

/// Telefono, pulsante ☐ mentre non si scrive: una voce vuota in fondo alla
/// nota (se l'ultima riga non e' gia' una voce vuota), con il cursore li' e il
/// fuoco nel testo, cosi' si apre la tastiera. `true` se il testo e' cambiato.
pub fn aggiungi_voce_in_fondo(ctx: &egui::Context, id: Id, testo: &mut String) -> bool {
    let ultima = testo.rsplit('\n').next().unwrap_or("");
    let gia_vuota = matches!(ultima.trim_end(), "[ ]");
    if !gia_vuota {
        if !testo.is_empty() && !testo.ends_with('\n') {
            testo.push('\n');
        }
        testo.push_str("[ ] ");
    }
    ctx.memory_mut(|m| m.request_focus(id));
    sposta_cursore(ctx, id, testo.chars().count());
    // La riga nuova deve vedersi anche nelle note lunghe.
    super::tastiera::mostra_cursore(ctx);
    !gia_vuota
}

/// Backspace che cancellerebbe un carattere del segno di una voce: toglie
/// tutto il segno (vedi `lista::cancella_segno`) e il tasto non arriva al
/// campo. Sul computer Backspace cancella il carattere prima del cursore; dal
/// telefono arriva con quel carattere gia' selezionato (vedi il ponte della
/// tastiera di Android). `true` se il testo e' cambiato.
fn cancella_segno_voce(ui: &mut Ui, id: Id, testo: &mut String) -> bool {
    let premuto = ui.input(|i| {
        i.events.iter().any(|e| matches!(e, egui::Event::Key { key: Key::Backspace, pressed: true, modifiers, .. } if modifiers.is_none()))
    });
    if !premuto {
        return false;
    }
    let ctx = ui.ctx().clone();
    let Some(selezione) = TextEditState::load(&ctx, id).and_then(|s| s.cursor.char_range()) else { return false };
    let [x, y] = [selezione.primary.index.0, selezione.secondary.index.0];
    let (da, a) = (x.min(y), x.max(y));
    // Il carattere che sparirebbe: quello prima del cursore, o l'unico selezionato.
    let cancellato = match a - da {
        0 if a > 0 => a - 1,
        1 => da,
        _ => return false,
    };
    let Some(cursore) = lista::cancella_segno(testo, cancellato) else { return false };
    ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Backspace));
    sposta_cursore(&ctx, id, cursore);
    true
}

/// Trasforma la riga del cursore in voce di lista (o viceversa).
fn alterna_riga(ctx: &egui::Context, id: Id, testo: &mut String) -> bool {
    let cursore = TextEditState::load(ctx, id)
        .and_then(|s| s.cursor.char_range())
        .map(|r| r.primary.index.0)
        .unwrap_or(testo.chars().count());
    let nuovo = lista::alterna_riga(testo, cursore);
    sposta_cursore(ctx, id, nuovo);
    true
}

fn sposta_cursore(ctx: &egui::Context, id: Id, posizione: usize) {
    let mut stato = TextEditState::load(ctx, id).unwrap_or_default();
    stato.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(posizione))));
    stato.store(ctx, id);
}

/// Spazio in piu' tra i tre caratteri del segno `[ ]`, in frazione della
/// dimensione del carattere: fa posto alla casella disegnata sopra.
const ALLARGA_SEGNO: f32 = misura(0.14, 0.2);

#[derive(Clone, Copy)]
struct Colori {
    testo: Color32,
    attenuato: Color32,
    evidenzia: Color32,
    /// Fondo e testo della parola trovata attuale.
    attuale: Color32,
    testo_attuale: Color32,
}

/// Disposizione del testo: il segno delle voci e' trasparente, il testo
/// delle voci fatte barrato e attenuato, le parole cercate evidenziate.
///
/// `piu_larga`: di quanto "x" e "X" sono piu' larghe dello spazio. Il segno
/// `[x]` di una voce fatta si stringe di altrettanto, cosi' occupa lo stesso
/// posto di `[ ]`: spuntando una voce la casella e il testo non si spostano.
fn componi(
    testo: &str,
    carattere: &FontId,
    colori: Colori,
    (ricerca, attuale): (&str, Option<usize>),
    larghezza: f32,
    piu_larga: [f32; 2],
) -> LayoutJob {
    // Righe un po' piu' distanziate del normale: il testo lungo si legge meglio.
    let normale = TextFormat {
        line_height: Some((carattere.size * super::tema::INTERLINEA).round()),
        ..TextFormat::simple(carattere.clone(), colori.testo)
    };
    let segno = TextFormat {
        color: Color32::TRANSPARENT,
        extra_letter_spacing: carattere.size * ALLARGA_SEGNO,
        ..normale.clone()
    };
    let fatta = TextFormat {
        color: colori.attenuato,
        strikethrough: Stroke::new(1.0, colori.attenuato),
        ..normale.clone()
    };
    // Pezzi del testo (in byte) con il loro formato.
    let byte: Vec<usize> = testo.char_indices().map(|(b, _)| b).chain([testo.len()]).collect();
    let mut pezzi: Vec<(usize, usize, TextFormat)> = Vec::new();
    let mut corrente = 0;
    for voce in lista::voci(testo) {
        let fine_segno = voce.segno + lista::LUNGHEZZA_SEGNO;
        pezzi.push((byte[corrente], byte[voce.segno], normale.clone()));
        // "[", il carattere in mezzo e "]": quello in mezzo si allarga meno se e' una x.
        let mezzo = match testo[byte[voce.segno + 1]..].chars().next() {
            Some('x') => piu_larga[0],
            Some('X') => piu_larga[1],
            _ => 0.0,
        };
        pezzi.push((byte[voce.segno], byte[voce.segno + 1], segno.clone()));
        let stretto = TextFormat { extra_letter_spacing: segno.extra_letter_spacing - mezzo, ..segno.clone() };
        pezzi.push((byte[voce.segno + 1], byte[voce.segno + 2], stretto));
        pezzi.push((byte[voce.segno + 2], byte[fine_segno], segno.clone()));
        // La riga che barra le voci fatte parte dalla parola, non dallo spazio dopo il segno.
        let mut inizio_testo = fine_segno;
        if voce.fatta && inizio_testo < voce.fine_riga && testo[byte[inizio_testo]..].starts_with(' ') {
            pezzi.push((byte[inizio_testo], byte[inizio_testo + 1], normale.clone()));
            inizio_testo += 1;
        }
        let formato = if voce.fatta { fatta.clone() } else { normale.clone() };
        pezzi.push((byte[inizio_testo], byte[voce.fine_riga], formato));
        corrente = voce.fine_riga;
    }
    let vuoto = normale.clone();
    pezzi.push((byte[corrente], testo.len(), normale));

    // Le parole trovate tagliano i pezzi e ne cambiano il fondo.
    let trovate = crate::modello::nota::Nota::occorrenze(testo, ricerca);
    let mut lavoro = LayoutJob::default();
    lavoro.wrap.max_width = larghezza;
    for (da, a, formato) in pezzi {
        let mut inizio = da;
        while inizio < a {
            let posizione = trovate.iter().position(|r| r.start <= inizio && inizio < r.end);
            let dentro = posizione.map(|p| &trovate[p]);
            let fine = match dentro {
                Some(r) => r.end.min(a),
                None => trovate.iter().map(|r| r.start).filter(|&s| s > inizio).min().unwrap_or(a).min(a),
            };
            let mut f = formato.clone();
            if dentro.is_some() && f.color != Color32::TRANSPARENT {
                if posizione.is_some() && posizione == attuale {
                    f.background = colori.attuale;
                    f.color = colori.testo_attuale;
                } else {
                    f.background = colori.evidenzia;
                }
            }
            lavoro.append(&testo[inizio..fine], 0.0, f);
            inizio = fine;
        }
    }
    if lavoro.sections.is_empty() {
        // Testo vuoto: serve comunque un formato per l'altezza del cursore.
        lavoro.append("", 0.0, vuoto);
    }
    lavoro
}

/// Disegna le caselle sopra i segni e gestisce il clic. `true` se una voce
/// e' stata spuntata o tolta.
fn caselle(ui: &mut Ui, t: &Tavolozza, uscita: &TextEditOutput, testo: &mut String, carattere: &FontId, abilitato: bool) -> bool {
    let voci = lista::voci(testo);
    if voci.is_empty() {
        return false;
    }
    let pittore = ui.painter_at(uscita.text_clip_rect);
    let origine = uscita.galley_pos.to_vec2();
    let mut da_alternare = None;
    for voce in voci {
        let inizio = uscita.galley.pos_from_cursor(CCursor::new(voce.segno)).translate(origine);
        let fine = uscita.galley.pos_from_cursor(CCursor::new(voce.segno + lista::LUNGHEZZA_SEGNO)).translate(origine);
        if (inizio.center().y - fine.center().y).abs() > 1.0 {
            continue; // segno spezzato su due righe: campo troppo stretto
        }
        // Con il dito la casella e' un po' piu' grande e si prende anche
        // toccando poco accanto (fino a circa 44 punti di lato).
        let lato = (carattere.size * misura(0.78, 0.9)).min(inizio.height() * misura(0.8, 0.9));
        let centro = pos2((inizio.min.x + fine.min.x) / 2.0, inizio.center().y);
        let casella = Rect::from_center_size(centro, Vec2::splat(lato));
        let presa = misura(3.0, ((44.0 - lato) / 2.0).max(6.0));
        // La casella e' un controllo a se', sopra il testo: un clic la spunta
        // senza spostare il cursore dentro il segno "[ ]" e, se il campo non
        // era in uso, senza aprire la tastiera del telefono. Se lo era, il
        // campo resta in uso (vedi `schermo::proteggi_fuoco`).
        let zona = casella.expand(presa).intersect(uscita.text_clip_rect);
        let mut sopra = false;
        if abilitato && zona.is_positive() {
            // Anche "trascinamento": con il campo in uso egui da' il trascinamento
            // al testo, che sposterebbe il cursore dentro il segno "[ ]" (e quello
            // che si scrive dopo romperebbe la voce: "[ ] uno" -> "[Xx] uno").
            let risposta = ui.interact(zona, uscita.response.id.with(("casella", voce.segno)), Sense::click_and_drag());
            super::schermo::proteggi_fuoco(ui.ctx(), zona);
            sopra = risposta.hovered();
            if risposta.clicked() {
                da_alternare = Some(voce.segno);
            }
        }
        if voce.fatta {
            pittore.rect_filled(casella, CornerRadius::same(3), t.accento);
            let segno = [
                pos2(casella.left() + lato * 0.22, casella.center().y),
                pos2(casella.left() + lato * 0.42, casella.bottom() - lato * 0.25),
                pos2(casella.right() - lato * 0.2, casella.top() + lato * 0.25),
            ];
            pittore.line_segment([segno[0], segno[1]], Stroke::new(1.8, t.testo_accento));
            pittore.line_segment([segno[1], segno[2]], Stroke::new(1.8, t.testo_accento));
        } else {
            let bordo = if sopra { t.accento } else { t.attenuato };
            pittore.rect_stroke(casella, CornerRadius::same(3), Stroke::new(1.5, bordo), egui::StrokeKind::Inside);
        }
        if sopra {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
    }
    da_alternare.is_some_and(|segno| lista::alterna(testo, segno))
}

#[cfg(test)]
mod prove {
    use super::parola_attorno;

    #[test]
    fn parola_sotto_il_dito() {
        let testo = "La città è bella";
        // Dentro "città" (caratteri 3..8).
        assert_eq!(parola_attorno(testo, 5), Some((3, 8)));
        // Subito dopo la parola.
        assert_eq!(parola_attorno(testo, 8), Some((3, 8)));
        // In fondo.
        assert_eq!(parola_attorno(testo, 16), Some((11, 16)));
        // Apostrofo dentro la parola.
        assert_eq!(parola_attorno("l'acqua", 3), Some((0, 7)));
        // Solo spazi.
        assert_eq!(parola_attorno("a   b", 2), None);
    }
}
