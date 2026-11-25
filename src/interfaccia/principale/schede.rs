//! Le schede delle note attive: fissate prima, poi nell'ordine scelto
//! trascinandole (vedi `modello::ordine`), poi per creazione.
//!
//! Le schede si restringono da 195 a 80 punti; se non entrano tutte
//! compaiono due frecce per scorrere un gruppo alla volta. Doppio clic (o F2)
//! su una scheda per rinominare la nota direttamente nella barra.
//!
//! Trascinamento: tenendo premuto su una scheda e spostando il mouse, la
//! scheda segue il puntatore e le altre si spostano per farle posto; al
//! rilascio resta li'. Vicino alle frecce le schede scorrono. Le fissate
//! restano un gruppo a se'. Durante una ricerca il trascinamento e' spento.

use egui::{
    Align2, CornerRadius, FontId, Id, Key, Order, Pos2, Rect, Sense, StrokeKind, TextEdit, Ui, pos2, vec2,
};

use crate::applicazione::Applicazione;
use crate::interfaccia::componenti::{Icona, LATO_ICONA, disegna_icona, pulsante_icona};
use crate::interfaccia::schermo::{self, TOCCO, misura};
use crate::interfaccia::tema::{PERICOLO_LEGGERO, PERICOLO_SOPRA, RAGGIO, TESTO_VUOTO, TITOLO_PAGINA, TRATTO_ATTIVO, Tavolozza, TintaNota, tinta_nota};
use crate::modello::ordine;
use crate::piattaforma::ora_locale;

/// Larghezza della striscia colorata a sinistra delle schede con un colore.
const STRISCIA: f32 = 5.0;

struct Scheda {
    id: String,
    titolo: String,
    conflitto: bool,
    tinta: Option<TintaNota>,
    fissata: bool,
}

/// Comandi del menu del tasto destro di una scheda.
enum ComandoScheda {
    Rinomina,
    ApriWidget,
    Fissa,
    Cronologia,
    Esporta,
    SpostaSinistra,
    SpostaDestra,
    Cancella,
}

/// Scheda trascinata: quale, e a che distanza dal suo bordo sinistro e'
/// stata presa (cosi' resta "attaccata" al puntatore nello stesso punto).
#[derive(Clone)]
struct Trascinamento {
    id: String,
    presa: f32,
}

const LARGHEZZA_MINIMA: f32 = misura(80.0, 96.0);
const LARGHEZZA_MASSIMA: f32 = 195.0;
const ALTEZZA: f32 = misura(34.0, 44.0);
use crate::interfaccia::tema::SPAZIO;
/// Distanza tra due schede della riga in alto.
const SPAZIO_SCHEDE: f32 = 4.0;
/// Larghezza del "+" dopo l'ultima scheda (computer).
const LARGHEZZA_PIU: f32 = misura(30.0, 44.0);
const FRECCIA: f32 = LATO_ICONA;
/// Ogni quanto scorrono le schede tenendo la scheda trascinata vicino a una freccia.
const PAUSA_SCORRIMENTO: f64 = 0.35;

pub fn mostra(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    if app.cartella.is_none() {
        return;
    }
    let segnaposto = app.tr.t("senza_titolo");
    let ricerca = app.interfaccia.ricerca.clone();
    let tutte = app.note_ordinate();
    let fissate_totali = ordine::fissate_in_testa(&tutte);
    let schede: Vec<Scheda> = tutte
        .into_iter()
        .filter(|n| n.corrisponde(&ricerca))
        .map(|n| Scheda {
            id: n.id.clone(),
            titolo: n.titolo_visibile(segnaposto).to_owned(),
            conflitto: n.conflitto_di.is_some(),
            tinta: tinta_nota(&n.colore, t.scuro),
            fissata: n.fissata,
        })
        .collect();

    if schede.is_empty() {
        let vuoto = if ricerca.trim().is_empty() { "nessuna_nota" } else { "nessun_risultato" };
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPAZIO * 2.0;
            nuova_nota(ui, app, t);
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width().max(0.0), ALTEZZA), Sense::hover());
            ui.painter().text(r.left_center(), Align2::LEFT_CENTER, app.tr.t(vuoto), FontId::proportional(TESTO_VUOTO), t.attenuato);
        });
        return;
    }

    // Quante schede entrano e quanto sono larghe (il "+" ha sempre il suo posto).
    // Sul telefono, quando le schede scorrono, il "+" e' in fondo alla fascia
    // che scorre (si vede arrivando all'ultima scheda): niente posto fisso.
    let riserva_piu = LARGHEZZA_PIU + SPAZIO_SCHEDE;
    let n = schede.len();
    let entrano_tutte = n as f32 * (LARGHEZZA_MINIMA + SPAZIO_SCHEDE) - SPAZIO_SCHEDE <= ui.available_width() - riserva_piu;
    let menu = if TOCCO && !entrano_tutte { 0.0 } else { riserva_piu };
    let disponibile = ui.available_width() - menu;
    let aiuto_nuova = app.tr.t("aiuto_nuova_nota").to_owned();
    let mut piu_premuto = false;
    // Con il dito le schede si scorrono trascinandole: niente frecce, che
    // sullo schermo stretto lascerebbero posto a una scheda sola.
    let frecce = !entrano_tutte && !TOCCO;
    // Con il dito, se non entrano tutte, le schede stanno in una fascia che
    // scorre seguendo il dito (e continua per un attimo dopo un gesto veloce).
    let scorrevoli = !entrano_tutte && TOCCO;
    let spazio_schede = if frecce { disponibile - 2.0 * (FRECCIA + SPAZIO_SCHEDE) } else { disponibile };
    let visibili = if entrano_tutte || scorrevoli {
        n
    } else {
        (((spazio_schede + SPAZIO_SCHEDE) / (LARGHEZZA_MINIMA + SPAZIO_SCHEDE)).floor() as usize).max(1)
    };
    let larghezza = ((spazio_schede + SPAZIO_SCHEDE) / visibili as f32 - SPAZIO_SCHEDE).clamp(LARGHEZZA_MINIMA, LARGHEZZA_MASSIMA);
    let passo = larghezza + SPAZIO_SCHEDE;

    // Una nota appena selezionata viene portata nella parte visibile.
    let selezionata = app.interfaccia.selezionata.clone();
    let posizione_selezionata = selezionata.as_ref().and_then(|s| schede.iter().position(|scheda| &scheda.id == s));
    let interfaccia = &mut app.interfaccia;
    let memoria = ui.scope_id().with("ultima_selezione_vista");
    // Si ricorda la selezione dell'ultima volta (anche "nessuna"): solo quando
    // cambia si spostano le schede, cosi' le frecce e il dito possono
    // nascondere la scheda selezionata senza che torni subito in vista.
    // Anche il posto della scheda conta: fissando la nota la sua scheda si
    // sposta, e deve restare in vista.
    let attuale = (selezionata.clone(), posizione_selezionata);
    let ultima: Option<(Option<String>, Option<usize>)> = ui.data(|d| d.get_temp(memoria));
    let selezione_cambiata = ultima.as_ref() != Some(&attuale);
    if selezione_cambiata {
        if let Some(p) = posizione_selezionata {
            if p < interfaccia.prima_scheda {
                interfaccia.prima_scheda = p;
            } else if p >= interfaccia.prima_scheda + visibili {
                interfaccia.prima_scheda = p + 1 - visibili;
            }
        }
        ui.data_mut(|d| d.insert_temp(memoria, attuale.clone()));
    }
    interfaccia.prima_scheda = interfaccia.prima_scheda.min(n.saturating_sub(visibili));

    // Trascinamento in corso (solo senza ricerca: con schede nascoste la
    // posizione sarebbe ambigua) e posizione in cui la scheda finirebbe.
    // Con il dito trascinare scorre le schede: per spostarne una c'e' il
    // menu che si apre tenendola premuta (Sposta a sinistra/destra).
    let si_trascina = ricerca.trim().is_empty() && app.interfaccia.rinomina.is_none() && !TOCCO;
    let si_sposta = ricerca.trim().is_empty() && app.interfaccia.rinomina.is_none();
    let id_trascinamento = Id::unique("scheda_trascinata");
    let mut trascinata: Option<Trascinamento> =
        if si_trascina { ui.data(|d| d.get_temp(id_trascinamento)) } else { None };
    if trascinata.as_ref().is_some_and(|tr| !schede.iter().any(|s| s.id == tr.id)) {
        trascinata = None;
    }
    let puntatore: Option<Pos2> = ui.input(|i| i.pointer.interact_pos());
    // Con il dito: la scheda tenuta premuta e poi mossa (vedi `SchedaPresa`).
    let id_presa = Id::unique("scheda_presa_dito");
    let mut presa: Option<SchedaPresa> = if TOCCO && si_sposta { ui.data(|d| d.get_temp(id_presa)) } else { None };
    if presa.as_ref().is_some_and(|p| !schede.iter().any(|s| s.id == p.id)) {
        presa = None;
    }
    let mut nuova_presa: Option<SchedaPresa> = None;
    let mut posti: Vec<(usize, Rect)> = Vec::new();

    let mut da_selezionare = None;
    let mut inizia_rinomina = None;
    let mut fine_rinomina: Option<bool> = None; // Some(true) conferma, Some(false) annulla
    let mut comando: Option<(String, ComandoScheda)> = None;
    let mut inizio_trascinamento: Option<Trascinamento> = None;
    let mut rilascio: Option<(String, usize)> = None;
    let mut disegno_trascinata: Option<(Rect, usize)> = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPAZIO_SCHEDE;
        let prima = app.interfaccia.prima_scheda;
        if frecce && pulsante_icona(ui, t, Icona::Precedente, app.tr.t("schede_precedenti"), prima > 0).clicked() {
            app.interfaccia.prima_scheda = prima.saturating_sub(visibili);
        }
        let inizio_schede = ui.cursor().min.x;
        let fine_schede = inizio_schede + visibili as f32 * passo - SPAZIO_SCHEDE;

        // Dove finirebbe la scheda trascinata: lo slot sotto il puntatore,
        // dentro il suo gruppo (fissate o no).
        let mut ordine_mostrato: Vec<usize> = (0..n).collect();
        let mut destinazione = None;
        if let (Some(tr), Some(p)) = (&trascinata, puntatore) {
            if let Some(da) = schede.iter().position(|s| s.id == tr.id) {
                let bordo = p.x - tr.presa + larghezza / 2.0;
                let slot = ((bordo - inizio_schede) / passo).floor().clamp(0.0, (visibili - 1) as f32) as usize;
                let a = ordine::limita_al_gruppo(da, (prima + slot).min(n - 1), fissate_totali, n);
                let elemento = ordine_mostrato.remove(da);
                ordine_mostrato.insert(a, elemento);
                destinazione = Some(a);
                // Vicino ai bordi le schede scorrono, una alla volta.
                if frecce {
                    let adesso = ui.input(|i| i.time);
                    let chiave = Id::unique("scheda_ultimo_scorrimento");
                    let ultimo: f64 = ui.data(|d| d.get_temp(chiave)).unwrap_or(0.0);
                    if adesso - ultimo > PAUSA_SCORRIMENTO {
                        if p.x < inizio_schede + 12.0 && prima > 0 {
                            app.interfaccia.prima_scheda -= 1;
                            ui.data_mut(|d| d.insert_temp(chiave, adesso));
                        } else if p.x > fine_schede - 12.0 && prima + visibili < n {
                            app.interfaccia.prima_scheda += 1;
                            ui.data_mut(|d| d.insert_temp(chiave, adesso));
                        }
                    }
                }
                ui.ctx().request_repaint();
            }
        }

        let mut disegna_schede = |ui: &mut Ui| {
        for &indice in ordine_mostrato.iter().skip(prima).take(visibili) {
            let scheda = &schede[indice];
            let Scheda { id, fissata, .. } = scheda;
            // Nella fascia che scorre ogni scheda e' larga quanto il suo titolo.
            let larghezza = if scorrevoli { larghezza_naturale(ui, scheda) } else { larghezza };
            // Identificativo legato alla nota: il menu del tasto destro e il
            // trascinamento continuano anche se le schede cambiano posto.
            let (r, _) = ui.allocate_exact_size(vec2(larghezza, ALTEZZA), Sense::hover());
            let attiva = selezionata.as_deref() == Some(id.as_str());
            // Aprendo una nota la sua scheda si vede intera, non tagliata dal bordo
            // ne' coperta dalla piccola freccia che indica altre schede.
            if scorrevoli && attiva && selezione_cambiata {
                ui.scroll_to_rect(r.expand2(vec2(12.0, 0.0)), None);
            }
            let senso = if si_trascina { Sense::click_and_drag() } else { Sense::click() };
            let risposta = ui.interact(r, Id::unique(("scheda", id.as_str())), senso);

            if let Some((id_rinomina, testo)) = app.interfaccia.rinomina.as_mut().filter(|(r, _)| r == id) {
                let id_campo = Id::unique(("rinomina", id_rinomina.as_str()));
                let uscita = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(r), |ui| {
                        TextEdit::singleline(testo)
                            .id(id_campo)
                            .margin(vec2(8.0, 7.0))
                            .desired_width(larghezza)
                            .hint_text(app.tr.t("senza_titolo"))
                            .show(ui)
                    })
                    .inner;
                crate::interfaccia::tastiera::campo(ui, &uscita, testo, false, false);
                let campo = uscita.response.response;
                if !campo.has_focus() && !campo.lost_focus() {
                    campo.request_focus();
                }
                if ui.input(|i| i.key_pressed(Key::Escape)) {
                    fine_rinomina = Some(false);
                } else if campo.lost_focus() {
                    fine_rinomina = Some(true);
                }
                continue;
            }

            if TOCCO {
                posti.push((indice, r));
                if presa.as_ref().is_some_and(|p| &p.id == id) {
                    // Al suo posto resta uno spazio vuoto; la scheda segue il dito.
                    ui.painter().rect(r, CornerRadius::same(RAGGIO), t.sfondo, t.tratto(), StrokeKind::Inside);
                    continue;
                }
                if si_sposta && presa.is_none() && nuova_presa.is_none() {
                    if let Some(punto) = tenuta_e_mossa(ui, r) {
                        nuova_presa = Some(SchedaPresa { id: id.clone(), presa: punto - r.min, larghezza: r.width(), destinazione: None, sul_cestino: false });
                    }
                }
            }
            let in_trascinamento = trascinata.as_ref().is_some_and(|tr| &tr.id == id);
            if in_trascinamento {
                // Al suo posto resta uno spazio vuoto; la scheda vera segue il mouse.
                ui.painter().rect(
                    r,
                    CornerRadius::same(RAGGIO),
                    t.sfondo,
                    t.tratto(),
                    StrokeKind::Inside,
                );
                disegno_trascinata = Some((r, indice));
            } else {
                disegna_scheda(ui, t, r, scheda, attiva, risposta.hovered());
            }

            if risposta.drag_started() {
                let presa = puntatore.map_or(larghezza / 2.0, |p| p.x - r.left());
                inizio_trascinamento = Some(Trascinamento { id: id.clone(), presa });
                da_selezionare = Some(id.clone());
            }
            if risposta.drag_stopped() {
                if let Some(a) = destinazione.filter(|_| in_trascinamento) {
                    rilascio = Some((id.clone(), a));
                }
            }

            // Con il dito, tenere premuto apre il menu: niente descrizione.
            let risposta = if trascinata.is_none() && !TOCCO {
                risposta.on_hover_text(app.tr.t("rinomina_suggerimento"))
            } else {
                risposta
            };
            if risposta.double_clicked() {
                inizia_rinomina = Some(id.clone());
            } else if risposta.clicked() {
                da_selezionare = Some(id.clone());
            }
            if presa.is_some() || nuova_presa.is_some() {
                continue;
            }
            risposta.context_menu(|ui| {
                ui.set_min_width(180.0);
                let mut voce = |ui: &mut Ui, testo: &str, c: ComandoScheda, attiva: bool| {
                    if ui.add_enabled(attiva, egui::Button::new(testo)).clicked() {
                        comando = Some((id.clone(), c));
                    }
                };
                voce(ui, app.tr.t("menu_rinomina"), ComandoScheda::Rinomina, true);
                // Su Android non ci sono widget sul desktop.
                if !cfg!(target_os = "android") {
                    voce(ui, app.tr.t("apri_widget"), ComandoScheda::ApriWidget, true);
                }
                let fissa = if *fissata { "togli_fissata" } else { "fissa_nota" };
                voce(ui, app.tr.t(fissa), ComandoScheda::Fissa, true);
                voce(ui, app.tr.t("cronologia"), ComandoScheda::Cronologia, true);
                voce(ui, app.tr.t("menu_esporta"), ComandoScheda::Esporta, true);
                ui.separator();
                voce(ui, app.tr.t("sposta_sinistra"), ComandoScheda::SpostaSinistra, si_sposta);
                voce(ui, app.tr.t("sposta_destra"), ComandoScheda::SpostaDestra, si_sposta);
                ui.separator();
                // In rosso come il pulsante Cancella della nota.
                if ui.add(egui::Button::new(egui::RichText::new(app.tr.t("cancella")).color(t.pericolo))).clicked() {
                    comando = Some((id.clone(), ComandoScheda::Cancella));
                }
            });
        }
        };
        if scorrevoli {
            let uscita = schermo::scorrevole(egui::ScrollArea::horizontal())
                .id_salt("schede_scorrevoli")
                // Mentre una scheda segue il dito la riga sta ferma.
                .scroll_source(if presa.is_none() { egui::scroll_area::ScrollSource::ALL } else { egui::scroll_area::ScrollSource::NONE })
                .max_width(spazio_schede)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    let riga = ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = SPAZIO_SCHEDE;
                        disegna_schede(ui);
                        piu_premuto |= pulsante_piu(ui, t, &aiuto_nuova);
                    });
                    // La scheda presa, portata vicino a un bordo della riga, fa
                    // scorrere le altre: cosi' si arriva anche a quelle nascoste.
                    if let (Some(_), Some(dito)) = (presa.as_ref(), puntatore) {
                        let vista = ui.clip_rect();
                        if dito.y < vista.bottom() + 40.0 {
                            if dito.x < vista.left() + 40.0 {
                                ui.scroll_with_delta(vec2(8.0, 0.0));
                            } else if dito.x > vista.right() - 40.0 {
                                ui.scroll_with_delta(vec2(-8.0, 0.0));
                            }
                        }
                    }
                    riga
                });
            // Una piccola freccia sul bordo dice che ci sono altre schede da quella parte.
            let scorso = uscita.state.clamped_offset().x;
            let a_destra = uscita.content_size.x - uscita.inner_rect.width() - scorso;
            altre_schede(ui, t, uscita.inner_rect, scorso > 1.0, a_destra > 1.0);
        } else {
            disegna_schede(ui);
        }
        if frecce {
            // Spinge la freccia destra in fondo alla riga.
            let usato = SPAZIO_SCHEDE + visibili.min(n - prima) as f32 * passo;
            let resto = spazio_schede + SPAZIO_SCHEDE - usato;
            if resto > 0.0 {
                ui.add_space(resto);
            }
            let altre = prima + visibili < n;
            if pulsante_icona(ui, t, Icona::Successivo, app.tr.t("schede_successive"), altre).clicked() {
                app.interfaccia.prima_scheda = (prima + visibili).min(n.saturating_sub(visibili));
            }
        }
        // Come nei browser: il "+" subito dopo l'ultima scheda.
        if !scorrevoli {
            piu_premuto |= pulsante_piu(ui, t, &aiuto_nuova);
        }

        // La scheda trascinata, sopra tutto, alla posizione del puntatore
        // (solo in orizzontale, dentro la riga delle schede).
        if let (Some(tr), Some(p), Some((posto, indice))) = (&trascinata, puntatore, disegno_trascinata) {
            let x = (p.x - tr.presa).clamp(inizio_schede, fine_schede - larghezza);
            let r = Rect::from_min_size(pos2(x, posto.top()), posto.size());
            egui::Area::new(Id::unique("scheda_che_segue"))
                .order(Order::Foreground)
                .fixed_pos(r.min)
                .interactable(false)
                .show(ui.ctx(), |ui| {
                    let (r, _) = ui.allocate_exact_size(r.size(), Sense::hover());
                    let attiva = selezionata.as_deref() == Some(tr.id.as_str());
                    disegna_scheda(ui, t, r, &schede[indice], attiva, true);
                });
        }
    });

    // Telefono: la scheda presa segue il dito. Lasciata nella riga cambia
    // posto (una linea dice dove), lasciata sul Cestino in basso va nel cestino.
    let mut cestina = None;
    if let Some(nuovo) = nuova_presa {
        egui::Popup::close_all(ui.ctx());
        presa = Some(nuovo);
    }
    if let Some(mut p) = presa.take() {
        schermo::imposta_trascina_nota(ui.ctx(), true);
        let giu = ui.input(|i| i.pointer.any_down());
        if let Some(dito) = puntatore {
            let fascia = posti.iter().fold(Rect::NOTHING, |a, (_, r)| a.union(*r));
            let fascia = Rect::from_x_y_ranges(ui.max_rect().x_range(), fascia.y_range()).expand2(vec2(0.0, 30.0));
            p.sul_cestino = schermo::zona_cestino(ui.ctx()).is_some_and(|z| z.contains(dito));
            p.destinazione = None;
            if fascia.contains(dito) {
                let altre: Vec<&(usize, Rect)> = posti.iter().filter(|(i, _)| schede[*i].id != p.id).collect();
                let a = altre.iter().filter(|(_, r)| r.center().x < dito.x).count();
                p.destinazione = Some(a);
                // La linea tra le schede dove andra' a finire.
                let x = match (a.checked_sub(1).and_then(|i| altre.get(i)), altre.get(a)) {
                    (Some((_, prima)), _) => prima.right() + SPAZIO_SCHEDE / 2.0,
                    (None, Some((_, dopo))) => dopo.left() - SPAZIO_SCHEDE / 2.0,
                    _ => dito.x,
                };
                let alto = fascia.top() + 30.0;
                ui.painter().clone().with_layer_id(egui::LayerId::new(Order::Foreground, Id::unique("linea_scheda")))
                    .line_segment([pos2(x, alto), pos2(x, alto + ALTEZZA)], egui::Stroke::new(3.0, t.accento));
            }
            if let Some(indice) = schede.iter().position(|s| s.id == p.id) {
                // Un po' sopra il dito: si vede dove la si sta lasciando.
                let r = Rect::from_min_size(dito - p.presa - vec2(0.0, 30.0), vec2(p.larghezza, ALTEZZA));
                egui::Area::new(Id::unique("scheda_presa_che_segue"))
                    .order(Order::Foreground)
                    .fixed_pos(r.min)
                    .interactable(false)
                    .show(ui.ctx(), |ui| {
                        let (r, _) = ui.allocate_exact_size(r.size(), Sense::hover());
                        ombra(ui, r);
                        let attiva = selezionata.as_deref() == Some(schede[indice].id.as_str());
                        disegna_scheda(ui, t, r, &schede[indice], attiva, true);
                    });
            }
            ui.ctx().request_repaint();
        }
        if giu {
            ui.data_mut(|d| d.insert_temp(id_presa, p));
        } else {
            ui.data_mut(|d| d.remove::<SchedaPresa>(id_presa));
            if p.sul_cestino {
                cestina = Some(p.id);
            } else if let Some(a) = p.destinazione {
                rilascio = Some((p.id, a));
            }
        }
    }
    if let Some(id) = cestina {
        app.sposta_nel_cestino(&id);
    }

    // Stato del trascinamento per il fotogramma successivo.
    if let Some(nuovo) = inizio_trascinamento {
        ui.data_mut(|d| d.insert_temp(id_trascinamento, nuovo));
    }
    if rilascio.is_some() || !ui.input(|i| i.pointer.any_down()) {
        ui.data_mut(|d| d.remove::<Trascinamento>(id_trascinamento));
    }
    if let Some((id, a)) = rilascio {
        app.sposta_scheda(&id, a);
    }

    if piu_premuto {
        app.conferma_rinomina();
        app.nuova_nota();
    }
    match fine_rinomina {
        Some(true) => app.conferma_rinomina(),
        Some(false) => app.interfaccia.rinomina = None,
        None => {}
    }
    if let Some(id) = da_selezionare {
        app.conferma_rinomina();
        app.interfaccia.selezionata = Some(id);
    }
    let inizia_rinomina = match comando {
        Some((id, ComandoScheda::Rinomina)) => Some(id),
        Some((id, ComandoScheda::ApriWidget)) => {
            app.apri_widget(&id);
            inizia_rinomina
        }
        Some((id, ComandoScheda::Fissa)) => {
            app.alterna_fissata(&id);
            inizia_rinomina
        }
        Some((id, ComandoScheda::Cronologia)) => {
            app.interfaccia.selezionata = Some(id.clone());
            app.apri_cronologia(&id);
            inizia_rinomina
        }
        Some((id, ComandoScheda::Esporta)) => {
            super::barra_laterale::esporta_nota(app, &id);
            inizia_rinomina
        }
        Some((id, ComandoScheda::SpostaSinistra)) => {
            app.sposta_scheda_di(&id, -1);
            inizia_rinomina
        }
        Some((id, ComandoScheda::SpostaDestra)) => {
            app.sposta_scheda_di(&id, 1);
            inizia_rinomina
        }
        Some((id, ComandoScheda::Cancella)) => {
            app.sposta_nel_cestino(&id);
            inizia_rinomina
        }
        None => inizia_rinomina,
    };
    if let Some(id) = inizia_rinomina {
        app.conferma_rinomina();
        let titolo = app.nota(&id).map(|n| n.titolo.clone()).unwrap_or_default();
        app.interfaccia.selezionata = Some(id.clone());
        app.interfaccia.rinomina = Some((id, titolo));
    }
}

/// Telefono: scheda tenuta premuta e poi mossa con il dito. Si guarda il
/// dito direttamente (come nella panoramica): la riga che scorre e il menu
/// della scheda non lasciano a egui segnalare il trascinamento.
#[derive(Clone)]
struct SchedaPresa {
    id: String,
    /// Dove il dito ha preso la scheda, rispetto al suo angolo.
    presa: egui::Vec2,
    larghezza: f32,
    /// Posizione tra le schede in cui finirebbe lasciandola adesso.
    destinazione: Option<usize>,
    sul_cestino: bool,
}

/// Il dito, appoggiato in `r`, e' rimasto fermo almeno `TENUTA_PER_SPOSTARE`
/// e poi si e' mosso: restituisce dove era stato appoggiato. Un movimento
/// subito e' uno scorrimento, non uno spostamento.
///
/// Se il dito si muove gia' durante l'attesa (per esempio per far scorrere
/// le schede piano) quella pressione non prende piu' nessuna scheda.
fn tenuta_e_mossa(ui: &Ui, r: Rect) -> Option<Pos2> {
    let (o, p, s, adesso, giu) = ui.input(|i| {
        (i.pointer.press_origin(), i.pointer.latest_pos(), i.pointer.press_start_time(), i.time, i.pointer.primary_down())
    });
    let (Some(o), Some(p), Some(s), true) = (o, p, s, giu) else { return None };
    // Per questa pressione (riconosciuta dal suo inizio): il dito si e' mosso troppo presto?
    let chiave = Id::unique("pressione_mossa_presto");
    let mut mossa: Option<(f64, bool)> = ui.data(|d| d.get_temp(chiave));
    if mossa.is_none_or(|(inizio, _)| inizio != s) {
        mossa = Some((s, false));
    }
    let (_, mut presto) = mossa.unwrap_or((s, false));
    let tenuta = adesso - s >= TENUTA_PER_SPOSTARE;
    if !tenuta && o.distance(p) > FERMO {
        presto = true;
    }
    ui.data_mut(|d| d.insert_temp(chiave, (s, presto)));
    (r.contains(o) && !presto && tenuta && o.distance(p) > 12.0).then_some(o)
}

/// Sotto la scheda presa, un'ombra: sembra sollevata dal foglio.
fn ombra(ui: &Ui, r: Rect) {
    for (allarga, alfa) in [(6.0, 18), (3.0, 34), (1.0, 50)] {
        ui.painter().rect_filled(r.translate(vec2(0.0, 4.0)).expand(allarga), CornerRadius::same(RAGGIO + 4), egui::Color32::from_black_alpha(alfa));
    }
}

/// Disegna una scheda: fondo, contorno, striscia del colore, puntina, titolo.
fn disegna_scheda(ui: &mut Ui, t: &Tavolozza, r: Rect, scheda: &Scheda, attiva: bool, sopra: bool) {
    // Come le linguette di un raccoglitore: una linea sottile corre sotto
    // tutta la riga; la scheda aperta ha fondo, bordo in alto e ai lati e
    // nessuna linea sotto, cosi' si unisce alla nota. Le altre sono solo testo.
    // Il fondo della scheda aperta e' quello della nota sotto: sul telefono la
    // nota non ha riquadro (sfondo), sul computer e' nel riquadro (pannello).
    let telefono = crate::interfaccia::schermo::telefono(ui.ctx());
    let fondo_nota = if telefono { t.sfondo } else { t.pannello };
    // Sul computer la linea sotto le schede e' il bordo del riquadro della
    // nota (vedi `principale::area_di_lavoro`); sul telefono la si disegna qui.
    let linea_sotto = telefono;
    let pittore = ui.painter();
    let base = r.bottom() - 0.5;
    let mezzo = SPAZIO_SCHEDE / 2.0;
    let linea = t.tratto();
    let colore_bordo = if scheda.conflitto { t.pericolo } else { t.bordo };
    if attiva {
        // Dove si apre nel riquadro della nota (vedi `principale::area_di_lavoro`).
        ui.ctx().data_mut(|d| d.insert_temp(Id::unique("linguetta_aperta"), r));
        // Fondo e bordo in un'unica forma (angoli in alto arrotondati, in basso
        // dritti): due disegni separati non coincidono e lasciano un alone.
        let angoli = CornerRadius { nw: RAGGIO, ne: RAGGIO, sw: 0, se: 0 };
        let tratto = egui::Stroke::new(if scheda.conflitto { TRATTO_ATTIVO } else { 1.0 }, colore_bordo);
        pittore.rect(r, angoli, fondo_nota, tratto, StrokeKind::Inside);
        // Senza la base: li' la linguetta si apre nella nota.
        let varco = Rect::from_min_max(pos2(r.left() + tratto.width, r.bottom() - 1.5), pos2(r.right() - tratto.width, r.bottom() + 0.5));
        pittore.rect_filled(varco, 0.0, fondo_nota);
        // La base continua solo fuori dalla scheda aperta.
        if linea_sotto {
            pittore.line_segment([pos2(r.left() - mezzo, base), pos2(r.left(), base)], linea);
            pittore.line_segment([pos2(r.right(), base), pos2(r.right() + mezzo, base)], linea);
        }
    } else {
        if sopra {
            // Il fondo al passaggio del mouse e' grande come la linguetta aperta
            // (selezionandola non cambia dimensione) e scende fino alla linea;
            // dalle schede vicine lo separa la distanza tra le schede.
            let angoli = CornerRadius { nw: RAGGIO, ne: RAGGIO, sw: 0, se: 0 };
            pittore.rect_filled(r, angoli, t.superficie_attiva);
        }
        if scheda.conflitto {
            pittore.rect_stroke(r.shrink(1.0), CornerRadius::same(RAGGIO), egui::Stroke::new(TRATTO_ATTIVO, t.pericolo), StrokeKind::Inside);
        }
        if linea_sotto {
            pittore.line_segment([pos2(r.left() - mezzo, base), pos2(r.right() + mezzo, base)], linea);
        }
    }
    // Il colore della nota e' una striscia sottile in cima alla linguetta.
    let mut area_testo = r.shrink2(vec2(10.0, 0.0));
    if let Some(tinta) = scheda.tinta {
        let striscia = Rect::from_min_max(pos2(r.left() + 4.0, r.top() + 1.0), pos2(r.right() - 4.0, r.top() + 4.0));
        pittore.rect_filled(striscia, CornerRadius::same(2), tinta.segno);
    }
    // Le note fissate hanno una piccola puntina prima del titolo.
    if scheda.fissata {
        let lato = 12.0;
        let icona = Rect::from_min_size(pos2(area_testo.left(), r.center().y - lato / 2.0), egui::Vec2::splat(lato));
        disegna_icona(ui.painter(), icona, Icona::PuntinaAttiva, t.attenuato);
        area_testo.min.x += lato + 4.0;
    }
    // Il titolo si disegna direttamente: un'area di egui occuperebbe spazio
    // nella riga e la scheda dopo partirebbe sopra la fine di questa.
    let colore = if attiva { t.testo } else { t.attenuato };
    let carattere = egui::TextStyle::Button.resolve(ui.style());
    let mut lavoro = egui::text::LayoutJob::simple_singleline(scheda.titolo.clone(), carattere, colore);
    lavoro.wrap = egui::text::TextWrapping::truncate_at_width(area_testo.width());
    let galleria = ui.fonts_mut(|f| f.layout_job(lavoro));
    let posto = pos2(area_testo.center().x - galleria.size().x / 2.0, area_testo.center().y - galleria.size().y / 2.0);
    ui.painter().galley(posto, galleria, colore);
}
/// Il riquadro con il numero delle note, come il contatore delle schede dei
/// browser.
pub fn disegna_contatore(pittore: &egui::Painter, r: Rect, quante: usize, colore: egui::Color32) {
    pittore.rect_stroke(r, CornerRadius::same(RAGGIO), egui::Stroke::new(TRATTO_ATTIVO, colore), StrokeKind::Inside);
    let numero = if quante > 99 { "99+".to_owned() } else { quante.to_string() };
    pittore.text(r.center(), Align2::CENTER_CENTER, numero, FontId::proportional(12.5), colore);
}
/// Altezza del pulsante Cestino in fondo alla panoramica (con lo spazio sopra).
pub const ALTEZZA_CESTINO_PANORAMICA: f32 = misura(44.0, 56.0) + 8.0;
/// Una nota nella panoramica.
struct Casella {
    id: String,
    titolo: String,
    testo: String,
    tinta: Option<TintaNota>,
    fissata: bool,
    quando: String,
    /// Colonna a destra: etichetta e numero ("Righe" 12, "Parole" 85,
    /// "Spunte" 2/5 se la nota ha una lista).
    conteggi: Vec<(String, String)>,
}

/// Casella trascinata nella panoramica: quale, dove e' stata presa rispetto
/// al suo angolo, e dove finirebbe lasciandola adesso.
#[derive(Clone)]
struct CasellaTrascinata {
    id: String,
    presa: egui::Vec2,
    destinazione: usize,
    sul_cestino: bool,
}

/// Tenendo il dito fermo su una scheda almeno per questo tempo (secondi, da
/// quando egui vede il dito: in tutto circa mezzo secondo) e poi muovendolo,
/// la scheda si stacca e segue il dito.
const TENUTA_PER_SPOSTARE: f64 = 0.4;
/// Quanto (punti) il dito puo' muoversi durante l'attesa restando "fermo".
const FERMO: f32 = 8.0;

/// Telefono: tutte le note come schede grandi, a griglia, come la panoramica
/// delle schede dei browser. Ogni scheda mostra titolo, prime righe, data,
/// colore e puntina; un tocco apre la nota, tenendo premuto si apre il suo
/// menu. Tenendola premuta e trascinandola si cambia l'ordine delle note; sul
/// pulsante Cestino in fondo la nota va nel cestino (con Annulla). L'ultima
/// scheda, "+", crea una nota nuova. Con una ricerca in corso ci sono solo le
/// note trovate e il trascinamento e' spento, come nella riga delle schede.
/// La casella di una nota per la panoramica (o per il Cestino).
fn casella_di(app: &Applicazione, n: &crate::modello::nota::Nota, t: &Tavolozza) -> Casella {
    // Quando e' stata modificata, in breve: "oggi 21:15", "ieri 9:02", "3 mar 10:40".
    let q = ora_locale::quando(n.modificata, app.tr.codice());
    let chiave = match q.giorni_fa {
        0 => "data_breve_oggi",
        1 => "data_breve_ieri",
        _ => "data_breve_il",
    };
    let quando = app.tr.f(chiave, &[("data", &q.data), ("ora", &q.ora)]);
    // Senza titolo ne' testo la scheda lo dice, invece di restare bianca.
    let vuota = n.titolo.trim().is_empty() && n.testo.trim().is_empty();
    let titolo = if vuota { app.tr.t("nota_vuota").to_owned() } else { n.titolo_visibile(app.tr.t("senza_titolo")).to_owned() };
    let c = super::riga_stato::conteggi(n);
    let mut conteggi = vec![
        (app.tr.t("etichetta_righe").to_owned(), c.righe.to_string()),
        (app.tr.t("etichetta_parole").to_owned(), c.parole.to_string()),
    ];
    if let Some((fatte, tutte)) = c.spunte {
        conteggi.push((app.tr.t("etichetta_spunte").to_owned(), format!("{fatte}/{tutte}")));
    }
    Casella {
        conteggi,
        id: n.id.clone(),
        titolo,
        testo: anteprima(&n.testo),
        tinta: tinta_nota(&n.colore, t.scuro),
        fissata: n.fissata,
        quando,
    }
}

/// Telefono, pagina Cestino: le note del cestino come schede, come nella
/// panoramica (senza "+" e senza trascinarle). Un tocco apre la nota, a
/// pagina intera, e da li' si scorre tra le altre del cestino; tenendo
/// premuto compaiono Ripristina ed Elimina definitivamente (con ANNULLA).
/// La nota del cestino da cui si e' saliti ha il bordo nel colore principale.
/// `true` se le schede sono scorse in cima.
pub fn caselle_cestino(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza, note: &[String], altezza_massima: f32) -> bool {
    let caselle: Vec<Casella> = note.iter().filter_map(|id| app.nota(id)).map(|n| casella_di(app, n, t)).collect();
    let colonne = if ui.available_width() > 520.0 { 3 } else { 2 };
    let larghezza = (ui.available_width() - SPAZIO * (colonne - 1) as f32) / colonne as f32;
    let altezza = misura(130.0, 150.0);
    let selezionata = app.interfaccia.selezionata.clone();
    let mut apri = None;
    let mut ripristina = None;
    let mut elimina = None;
    let griglia = schermo::scorrevole(egui::ScrollArea::vertical())
        .id_salt("schede_cestino")
        .max_height(altezza_massima)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for riga in caselle.chunks(colonne) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = SPAZIO;
                    for casella in riga {
                        let (r, _) = ui.allocate_exact_size(vec2(larghezza, altezza), Sense::hover());
                        let risposta = ui.interact(r, Id::unique(("casella_cestino", casella.id.as_str())), Sense::click());
                        let attiva = selezionata.as_deref() == Some(casella.id.as_str());
                        disegna_casella(ui, t, r, casella, attiva);
                        if risposta.clicked() {
                            apri = Some(casella.id.clone());
                        }
                        egui::Popup::context_menu(&risposta).show(|ui| {
                            ui.set_min_width(180.0);
                            // Verde e rosso come i pulsanti della nota del cestino.
                            let voce = egui::Button::new(egui::RichText::new(app.tr.t("menu_ripristina")).color(t.successo));
                            if ui.add(voce).clicked() {
                                ripristina = Some(casella.id.clone());
                            }
                            let voce = egui::Button::new(egui::RichText::new(app.tr.t("menu_elimina_definitivamente")).color(t.pericolo));
                            if ui.add(voce).clicked() {
                                elimina = Some(casella.id.clone());
                            }
                        });
                    }
                });
                ui.add_space(SPAZIO);
            }
        });
    if let Some(id) = ripristina {
        // La nota torna tra le altre; si resta nel Cestino.
        app.ripristina(&id);
    }
    if let Some(id) = elimina {
        app.elimina_con_annulla(vec![id]);
    }
    if let Some(id) = apri {
        super::pagine::apri_nota_del_cestino(app, id);
    }
    griglia.state.clamped_offset().y <= 1.0
}

pub fn panoramica(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    // Disegnata in posti diversi (a riposo, mentre sale o scende, accanto a
    // Cestino e Impostazioni): con lo stesso `scope_id` la griglia resta
    // scorsa dov'era e non salta a fine animazione.
    ui.scope_builder(egui::UiBuilder::new().scope_id(Id::unique("ambito_panoramica")), |ui| griglia_panoramica(ui, app, t));
}

fn griglia_panoramica(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let ricerca = app.interfaccia.ricerca.clone();
    let tutte = app.note_ordinate();
    let fissate_totali = ordine::fissate_in_testa(&tutte);
    let note: Vec<Casella> = tutte.into_iter().filter(|n| n.corrisponde(&ricerca)).map(|n| casella_di(app, n, t)).collect();
    let n = note.len();
    let mut apri = None;
    let mut chiudi = false;
    let mut nuova = false;
    let mut comando: Option<(String, ComandoScheda)> = None;

    let si_trascina = ricerca.trim().is_empty();
    let id_trascinata = Id::unique("panoramica_trascinata");
    let mut trascinata: Option<CasellaTrascinata> = if si_trascina { ui.data(|d| d.get_temp(id_trascinata)) } else { None };
    if trascinata.as_ref().is_some_and(|tr| !note.iter().any(|c| c.id == tr.id)) {
        trascinata = None;
    }
    let puntatore: Option<Pos2> = ui.input(|i| i.pointer.interact_pos());
    let giu = ui.input(|i| i.pointer.any_down());

    // In alto: Note (6) e, a destra, Nuova · Impostazioni · ⌄ (torna alla
    // nota); sotto, la ricerca sempre aperta, come sul computer. Trascinando
    // la testata verso il basso si torna alla nota (`Programma::smista_gesti`).
    let inizio_testata = ui.cursor().min;
    let mut impostazioni = false;
    ui.horizontal(|ui| {
        ui.set_height(ALTEZZA);
        ui.label(egui::RichText::new(app.tr.f("panoramica_titolo", &[("n", &n.to_string())])).size(TITOLO_PAGINA).color(t.testo));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            chiudi = pulsante_icona(ui, t, Icona::Giu, app.tr.t("chiudi_panoramica_giu"), true).clicked();
            ui.add_space(6.0);
            impostazioni = pulsante_icona(ui, t, Icona::Ingranaggio, app.tr.t("aiuto_impostazioni"), true).clicked();
            nuova |= crate::interfaccia::componenti::pulsante_icona_accento(ui, t, Icona::Piu, app.tr.t("aiuto_nuova_nota")).clicked();
        });
    });
    super::ricerca::mostra(ui, app, t);
    let testata = Rect::from_min_max(inizio_testata, pos2(ui.max_rect().right(), ui.cursor().min.y));
    schermo::imposta_zona_chiudi_panoramica(ui.ctx(), Some(testata));
    ui.add_space(SPAZIO);
    if impostazioni {
        app.interfaccia.pagina = crate::applicazione::Pagina::Impostazioni;
    }

    // In fondo il Cestino, grande: un tocco apre la sua pagina; mentre si
    // trascina una scheda diventa rosso e lasciandola li' la nota ci finisce
    // dentro (con il riquadro Annulla, `principale::avviso_annulla`).
    let nel_cestino = app.note_cestino_visibili().len();
    let mut zona_cestino = Rect::NOTHING;
    let mut apri_cestino = false;
    egui::Panel::bottom("cestino_panoramica")
        .frame(egui::Frame::new().inner_margin(egui::Margin { left: 0, right: 0, top: 8, bottom: 0 }))
        .show_separator_line(false)
        .show(ui, |ui| {
            let alto = ALTEZZA_CESTINO_PANORAMICA - 8.0;
            let (r, risposta) = ui.allocate_exact_size(vec2(ui.available_width(), alto), Sense::click());
            zona_cestino = r;
            let sopra = trascinata.is_some() && puntatore.is_some_and(|p| r.contains(p));
            // Come gli altri pulsanti; mentre si trascina una scheda diventa rosso.
            let (fondo, contorno, colore) = if sopra {
                (t.pericolo.gamma_multiply(PERICOLO_SOPRA), egui::Stroke::new(TRATTO_ATTIVO, t.pericolo), t.pericolo)
            } else if trascinata.is_some() {
                (t.pericolo.gamma_multiply(PERICOLO_LEGGERO), egui::Stroke::new(TRATTO_ATTIVO, t.pericolo), t.pericolo)
            } else {
                (crate::interfaccia::componenti::fondo_pulsante(t, risposta.is_pointer_button_down_on()), t.tratto(), t.testo)
            };
            ui.painter().rect(r, CornerRadius::same(RAGGIO), fondo, contorno, StrokeKind::Inside);
            let scritta = if trascinata.is_some() {
                app.tr.t("cestino_rilascia").to_owned()
            } else {
                app.tr.f("cestino_pulsante", &[("n", &nel_cestino.to_string())])
            };
            let icona = Rect::from_min_size(pos2(r.left() + 16.0, r.center().y - 10.0), egui::Vec2::splat(20.0));
            disegna_icona(ui.painter(), icona, Icona::Cestino, colore);
            // La scritta resta dentro il pulsante: se e' troppo lunga si accorcia con "…".
            let mut lavoro = egui::text::LayoutJob::simple_singleline(scritta, FontId::proportional(15.0), colore);
            lavoro.wrap = egui::text::TextWrapping::truncate_at_width(r.right() - 12.0 - (icona.right() + 12.0));
            let galleria = ui.fonts_mut(|f| f.layout_job(lavoro));
            let y = r.center().y - galleria.size().y / 2.0;
            ui.painter().galley(pos2(icona.right() + 12.0, y), galleria, colore);
            apri_cestino = trascinata.is_none() && risposta.on_hover_text(app.tr.t("aiuto_apri_cestino")).clicked();
        });
    if apri_cestino {
        super::pagine::apri_cestino(app);
    }    let colonne = if ui.available_width() > 520.0 { 3 } else { 2 };
    let larghezza = (ui.available_width() - SPAZIO * (colonne - 1) as f32) / colonne as f32;
    let altezza = misura(130.0, 150.0);
    let passo = vec2(larghezza + SPAZIO, altezza + SPAZIO);
    let selezionata = app.interfaccia.selezionata.clone();
    let mut inizio: Option<CasellaTrascinata> = None;
    let mut disegno_trascinata: Option<(Rect, usize)> = None;
    let griglia = schermo::scorrevole(egui::ScrollArea::vertical()).auto_shrink(false).show(ui, |ui| {
        // Tra una riga e l'altra lo stesso spazio che c'e' tra due schede
        // affiancate (solo `SPAZIO`, senza quello che egui aggiunge da se').
        ui.spacing_mut().item_spacing.y = 0.0;
        // Dove finirebbe la casella trascinata: quella sotto il dito, nel suo
        // gruppo (fissate o no). Sul Cestino resta dov'era.
        let origine = ui.cursor().min;
        let mut ordine_mostrato: Vec<usize> = (0..n).collect();
        if let (Some(tr), Some(p)) = (trascinata.as_mut(), puntatore) {
            if let Some(da) = note.iter().position(|c| c.id == tr.id) {
                tr.sul_cestino = zona_cestino.contains(p);
                let a = if tr.sul_cestino {
                    da
                } else {
                    let centro = p - tr.presa + vec2(larghezza, altezza) / 2.0 - origine;
                    let colonna = (centro.x / passo.x).floor().clamp(0.0, (colonne - 1) as f32) as usize;
                    let riga = (centro.y / passo.y).floor().max(0.0) as usize;
                    ordine::limita_al_gruppo(da, (riga * colonne + colonna).min(n - 1), fissate_totali, n)
                };
                tr.destinazione = a;
                let elemento = ordine_mostrato.remove(da);
                ordine_mostrato.insert(a, elemento);
                // Vicino ai bordi la griglia scorre da sola.
                let visibile = ui.clip_rect();
                if p.y < visibile.top() + 60.0 {
                    ui.scroll_with_delta(vec2(0.0, 8.0));
                } else if p.y > visibile.bottom() - 60.0 && !tr.sul_cestino {
                    ui.scroll_with_delta(vec2(0.0, -8.0));
                }
                ui.ctx().request_repaint();
            }
        }
        let caselle: Vec<Option<usize>> = ordine_mostrato.iter().copied().map(Some).chain([None]).collect();
        for riga in caselle.chunks(colonne) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = SPAZIO;
                for casella in riga {
                    let Some(i) = *casella else {
                        // La scheda "+": una nota nuova.
                        let (r, risposta) = ui.allocate_exact_size(vec2(larghezza, altezza), Sense::click());
                        // Con il dito niente "passaggio": si illumina solo premuto.
                        let sopra = (risposta.hovered() && !TOCCO) || risposta.is_pointer_button_down_on();
                        crate::interfaccia::componenti::disegna_fondo_pulsante(ui.painter(), r, t, sopra);
                        disegna_icona(ui.painter(), Rect::from_center_size(r.center(), egui::Vec2::splat(26.0)), Icona::Piu, t.accento);
                        if risposta.on_hover_text(app.tr.t("aiuto_nuova_nota")).clicked() {
                            nuova = true;
                        }
                        continue;
                    };
                    let casella = &note[i];
                    let senso = if si_trascina { Sense::click_and_drag() } else { Sense::click() };
                    let (r, _) = ui.allocate_exact_size(vec2(larghezza, altezza), Sense::hover());
                    let risposta = ui.interact(r, Id::unique(("casella", casella.id.as_str())), senso);
                    let attiva = selezionata.as_deref() == Some(casella.id.as_str());
                    if trascinata.as_ref().is_some_and(|tr| tr.id == casella.id) {
                        // Al suo posto resta uno spazio vuoto; la casella segue il dito.
                        ui.painter().rect(r, CornerRadius::same(RAGGIO), t.sfondo, t.tratto(), StrokeKind::Inside);
                        disegno_trascinata = Some((r, i));
                        continue;
                    }
                    disegna_casella(ui, t, r, casella, attiva);
                    // Il dito, tenuto premuto qui e poi mosso, sposta la casella.
                    // Si guarda il dito direttamente: dopo la pressione lunga egui
                    // apre il menu della casella e non segnala piu' il trascinamento.
                    // Un movimento subito, invece, e' uno scorrimento.
                    if si_trascina && trascinata.is_none() && inizio.is_none() {
                        if let Some(presa) = tenuta_e_mossa(ui, r).map(|o| o - r.min) {
                            inizio = Some(CasellaTrascinata { id: casella.id.clone(), presa, destinazione: i, sul_cestino: false });
                        }
                    }
                    if risposta.clicked() {
                        apri = Some(casella.id.clone());
                    }
                    if trascinata.is_none() && inizio.is_none() {
                        risposta.context_menu(|ui| {
                            ui.set_min_width(180.0);
                            let mut voce = |ui: &mut Ui, testo: &str, c: ComandoScheda| {
                                if ui.button(testo).clicked() {
                                    comando = Some((casella.id.clone(), c));
                                }
                            };
                            let fissa = if casella.fissata { "togli_fissata" } else { "fissa_nota" };
                            voce(ui, app.tr.t(fissa), ComandoScheda::Fissa);
                            voce(ui, app.tr.t("cronologia"), ComandoScheda::Cronologia);
                            voce(ui, app.tr.t("menu_esporta"), ComandoScheda::Esporta);
                            ui.separator();
                            // In rosso come il Cestino della nota.
                            let cancella = egui::Button::new(egui::RichText::new(app.tr.t("cancella")).color(t.pericolo));
                            if ui.add(cancella).clicked() {
                                comando = Some((casella.id.clone(), ComandoScheda::Cancella));
                            }
                        });
                    }
                }
            });
            ui.add_space(SPAZIO);
        }
    });
    // Con la griglia in cima, trascinando verso il basso da qualsiasi punto si
    // torna alla nota (come per chiudere tante finestre di Android).
    if griglia.state.clamped_offset().y <= 1.0 {
        schermo::imposta_zona_chiudi_panoramica(ui.ctx(), Some(testata.union(griglia.inner_rect)));
    }

    // La casella trascinata, sopra tutto, sotto il dito.
    if let (Some(tr), Some(p), Some((posto, i))) = (&trascinata, puntatore, disegno_trascinata) {
        let r = Rect::from_min_size(p - tr.presa, posto.size());
        egui::Area::new(Id::unique("casella_che_segue"))
            .order(Order::Foreground)
            .fixed_pos(r.min)
            .interactable(false)
            .show(ui.ctx(), |ui| {
                let (r, _) = ui.allocate_exact_size(r.size(), Sense::hover());
                ombra(ui, r);
                let attiva = selezionata.as_deref() == Some(note[i].id.as_str());
                disegna_casella(ui, t, r, &note[i], attiva);
            });
    }

    // Stato del trascinamento per il fotogramma successivo; al rilascio la
    // casella resta dove e' stata lasciata (o va nel cestino).
    let mut rilascio = None;
    if inizio.is_some() || trascinata.is_some() {
        schermo::imposta_trascina_nota(ui.ctx(), true);
    }
    if let Some(nuovo) = inizio {
        // Mentre si trascina non si apre il menu della casella.
        egui::Popup::close_all(ui.ctx());
        ui.data_mut(|d| d.insert_temp(id_trascinata, nuovo));
    } else if let Some(tr) = trascinata {
        if giu {
            ui.data_mut(|d| d.insert_temp(id_trascinata, tr));
        } else {
            ui.data_mut(|d| d.remove::<CasellaTrascinata>(id_trascinata));
            rilascio = Some(tr);
        }
    } else {
        ui.data_mut(|d| d.remove::<CasellaTrascinata>(id_trascinata));
    }
    if let Some(tr) = rilascio {
        if tr.sul_cestino {
            app.sposta_nel_cestino(&tr.id);
        } else {
            app.sposta_scheda(&tr.id, tr.destinazione);
        }
        // Il dito si alza sulla casella: non deve anche aprirla.
        apri = None;
    }

    if let Some(id) = apri {
        app.conferma_rinomina();
        app.interfaccia.selezionata = Some(id);
        app.interfaccia.panoramica = false;
    }
    if nuova {
        app.nuova_nota();
        app.interfaccia.panoramica = false;
    }
    if chiudi {
        app.interfaccia.panoramica = false;
    }

    match comando {
        Some((id, ComandoScheda::Fissa)) => app.alterna_fissata(&id),
        Some((id, ComandoScheda::Cronologia)) => {
            app.interfaccia.selezionata = Some(id.clone());
            app.apri_cronologia(&id);
        }
        Some((id, ComandoScheda::Esporta)) => super::barra_laterale::esporta_nota(app, &id),
        Some((id, ComandoScheda::Cancella)) => app.sposta_nel_cestino(&id),
        _ => {}
    }
}

/// Disegna una casella della panoramica: fondo, striscia del colore in alto,
/// puntina, titolo, prime righe e data in fondo.
fn disegna_casella(ui: &mut Ui, t: &Tavolozza, r: Rect, casella: &Casella, attiva: bool) {
    let fondo = match (casella.tinta, attiva) {
        (_, true) => t.selezione,
        (Some(tinta), false) => tinta.fondo,
        (None, false) => t.superficie,
    };
    let contorno = if attiva { egui::Stroke::new(TRATTO_ATTIVO, t.accento) } else { t.tratto() };
    ui.painter().rect(r, CornerRadius::same(RAGGIO), fondo, contorno, StrokeKind::Inside);
    if let Some(tinta) = casella.tinta {
        let striscia = Rect::from_min_size(r.min, vec2(r.width(), STRISCIA));
        ui.painter().rect_filled(striscia, CornerRadius { nw: RAGGIO, ne: RAGGIO, sw: 0, se: 0 }, tinta.segno);
    }
    // In fondo, su tutta la larghezza, quando e' stata modificata.
    let tutta = r.shrink2(vec2(10.0, 10.0));
    let data = ui.fonts_mut(|f| {
        let mut lavoro = egui::text::LayoutJob::simple_singleline(casella.quando.clone(), FontId::proportional(11.5), t.attenuato);
        lavoro.wrap = egui::text::TextWrapping::truncate_at_width(tutta.width());
        f.layout_job(lavoro)
    });
    let altezza_data = data.size().y;
    ui.painter_at(tutta).galley(pos2(tutta.left(), tutta.bottom() - altezza_data), data, t.attenuato);
    // Sopra, a destra, la colonna dei contatori, divisa dal resto da una linea
    // sottile (come le altre linee dell'app), un po' piu' corta della colonna.
    let fondo_colonna = tutta.bottom() - altezza_data - 6.0;
    let colonna = Rect::from_min_max(pos2(tutta.right() - LARGHEZZA_CONTEGGI, tutta.top()), pos2(tutta.right(), fondo_colonna));
    let x_linea = colonna.left() - 8.0;
    // Sulla scheda aperta (fondo nel colore principale) la linea normale non si vedrebbe.
    let linea = if attiva { egui::Stroke::new(1.0, t.accento.gamma_multiply(0.6)) } else { t.tratto() };
    ui.painter().line_segment([pos2(x_linea, tutta.top() + 2.0), pos2(x_linea, fondo_colonna - 2.0)], linea);
    disegna_conteggi(ui, t, colonna, &casella.conteggi);
    let dentro = Rect::from_min_max(tutta.min, pos2(x_linea - 8.0, fondo_colonna));
    let pittore = ui.painter_at(dentro);
    let mut sinistra = dentro.left();
    if casella.fissata {
        let posto = Rect::from_min_size(pos2(sinistra, dentro.top() + 3.0), egui::Vec2::splat(12.0));
        disegna_icona(&pittore, posto, Icona::PuntinaAttiva, t.attenuato);
        sinistra += 16.0;
    }
    let mut riga_titolo = egui::text::LayoutJob::simple_singleline(
        casella.titolo.clone(),
        crate::interfaccia::tema::carattere_titolo_piccolo(),
        t.testo,
    );
    riga_titolo.wrap = egui::text::TextWrapping::truncate_at_width(dentro.right() - sinistra);
    let galleria = ui.fonts_mut(|f| f.layout_job(riga_titolo));
    let altezza_titolo = galleria.size().y;
    pittore.galley(pos2(sinistra, dentro.top()), galleria, t.testo);
    let mut righe = egui::text::LayoutJob::simple(casella.testo.clone(), FontId::proportional(12.5), t.attenuato, dentro.width());
    righe.wrap.max_rows = 4;
    let galleria = ui.fonts_mut(|f| f.layout_job(righe));
    pittore.galley(pos2(dentro.left(), dentro.top() + altezza_titolo + 6.0), galleria, t.attenuato);
}
/// Larghezza della colonna dei contatori nelle schede della panoramica.
const LARGHEZZA_CONTEGGI: f32 = 46.0;

/// La colonna dei contatori: per ognuno l'etichetta piccola e grigia sopra
/// e il numero sotto, allineati a destra, il gruppo centrato in altezza.
fn disegna_conteggi(ui: &Ui, t: &Tavolozza, colonna: Rect, conteggi: &[(String, String)]) {
    let pittore = ui.painter_at(colonna.expand(2.0));
    let gallerie: Vec<_> = conteggi
        .iter()
        .map(|(etichetta, numero)| {
            ui.fonts_mut(|f| {
                (
                    f.layout_no_wrap(etichetta.clone(), FontId::proportional(10.5), t.attenuato),
                    f.layout_no_wrap(numero.clone(), FontId::proportional(14.0), t.testo),
                )
            })
        })
        .collect();
    const SPAZIO_CONTEGGI: f32 = 8.0;
    let alto: f32 = gallerie.iter().map(|(e, n)| e.size().y + n.size().y).sum::<f32>()
        + SPAZIO_CONTEGGI * gallerie.len().saturating_sub(1) as f32;
    let mut y = colonna.center().y - alto / 2.0;
    for (etichetta, numero) in gallerie {
        let (alto_e, alto_n) = (etichetta.size().y, numero.size().y);
        pittore.galley(pos2(colonna.right() - etichetta.size().x, y), etichetta, t.attenuato);
        y += alto_e;
        pittore.galley(pos2(colonna.right() - numero.size().x, y), numero, t.testo);
        y += alto_n + SPAZIO_CONTEGGI;
    }
}

/// Le prime righe del testo per la panoramica, con i segni delle liste piu'
/// leggibili: "[ ] pane" diventa "– pane", "[x] latte" "✓ latte" (il carattere Inter ha ✓, non ✔).
fn anteprima(testo: &str) -> String {
    testo
        .lines()
        .filter(|riga| !riga.trim().is_empty())
        .take(6)
        .map(|riga| {
            let riga = riga.trim_start();
            if let Some(resto) = riga.strip_prefix("[ ] ") {
                format!("– {resto}")
            } else if let Some(resto) = riga.strip_prefix("[x] ").or_else(|| riga.strip_prefix("[X] ")) {
                format!("✓ {resto}")
            } else {
                riga.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Larghezza di una scheda nella fascia che scorre: quanto serve al titolo
/// (con puntina e striscia del colore), tra la minima e la massima.
fn larghezza_naturale(ui: &Ui, scheda: &Scheda) -> f32 {
    let testo = ui.painter().layout_no_wrap(scheda.titolo.clone(), FontId::proportional(14.0), egui::Color32::WHITE).size().x;
    let extra = if scheda.fissata { 16.0 } else { 0.0 } + if scheda.tinta.is_some() { STRISCIA } else { 0.0 };
    (testo + extra + 28.0).clamp(LARGHEZZA_MINIMA, LARGHEZZA_MASSIMA)
}
/// Piccole frecce sui bordi della riga quando ci sono altre schede da scorrere.
fn altre_schede(ui: &Ui, t: &Tavolozza, fascia: Rect, prima: bool, dopo: bool) {
    let pittore = ui.painter();
    let lato = 10.0;
    for (c, icona) in [(prima, Icona::Precedente), (dopo, Icona::Successivo)] {
        if !c {
            continue;
        }
        let x = if icona == Icona::Precedente { fascia.left() + 1.0 } else { fascia.right() - lato - 1.0 };
        let striscia = Rect::from_min_size(pos2(x, fascia.top() + 1.0), vec2(lato, fascia.height() - 2.0));
        pittore.rect_filled(striscia, CornerRadius::same(3), t.sfondo);
        let freccia = Rect::from_center_size(striscia.center(), vec2(lato * 0.7, lato * 1.1));
        disegna_icona(pittore, freccia, icona, t.attenuato);
    }
}

/// Il pulsante "+" alla fine delle schede: una nota nuova, come Ctrl+N.
/// Il "+" dopo l'ultima scheda, come una linguetta piccola: niente
/// riquadro, solo il "+" nel colore principale; al passaggio del mouse lo
/// stesso fondo delle schede. Sul telefono ha sotto la stessa linea delle schede.
fn pulsante_piu(ui: &mut Ui, t: &Tavolozza, aiuto: &str) -> bool {
    let (r, risposta) = ui.allocate_exact_size(vec2(LARGHEZZA_PIU, ALTEZZA), Sense::click());
    if risposta.hovered() || risposta.is_pointer_button_down_on() {
        let angoli = CornerRadius { nw: RAGGIO, ne: RAGGIO, sw: 0, se: 0 };
        ui.painter().rect_filled(r, angoli, t.superficie_attiva);
    }
    if schermo::telefono(ui.ctx()) {
        let base = r.bottom() - 0.5;
        let mezzo = SPAZIO_SCHEDE / 2.0;
        ui.painter().line_segment([pos2(r.left() - mezzo, base), pos2(r.right(), base)], t.tratto());
    }
    let lato = misura(12.0, 16.0);
    disegna_icona(ui.painter(), Rect::from_center_size(r.center(), egui::Vec2::splat(lato)), Icona::Piu, t.accento);
    risposta.on_hover_text(aiuto).clicked()
}

fn nuova_nota(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let aiuto = app.tr.t("aiuto_nuova_nota").to_owned();
    if pulsante_piu(ui, t, &aiuto) {
        app.conferma_rinomina();
        app.nuova_nota();
    }
}
