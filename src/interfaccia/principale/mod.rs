//! La finestra principale: area di lavoro a sinistra, menu a destra.
//!
//! ```text
//! ┌───────────────────────────────────────────┬─────────────────────┐
//! │ « [scheda] [scheda] [scheda] » [+]        │ [cerca.......][×][»]│
//! │ ┌─ editor ─────── ☐ 📌 🕒 ● ⧉ Cancella ┐  │ ┌─ menu ──────────┐ │
//! │ │ Titolo                                │  │ │ [logo] schedario │ │
//! │ │ Modificata oggi alle 10:42            │  │ │ ▸ Cestino       │ │
//! │ │ Testo della nota...                   │  │ │ ▸ Aspetto       │ │
//! │ └───────────────────────────────────────┘  │ │ ▸ Rete locale   │ │
//! │ Salvato                         12 parole  │ │ ▸ ...           │ │
//! └───────────────────────────────────────────┴─────────────────────┘
//! ```
//!
//! Il pulsante » riduce il menu a una striscia sottile (con « per riaprirlo
//! e la lente per cercare): il testo prende tutta la larghezza.
//!
//! Sul telefono in verticale (vedi `schermo::telefono`) non c'e' il menu di
//! destra: le schede usano tutta la larghezza, la nota sta sotto e in fondo
//! c'e' una barra per il pollice:
//!
//! ```text
//! ┌──────────────────────────────┐
//! │ ‹ Spese │ Ciao │ Idee │ Tg › │  schede (scorrono con il dito)
//! │ Spese                     ⋮  │
//! │ oggi 21:15 · 10 parole       │
//! │ Testo...                     │
//! │   ⚙       🔍      [6]     ＋  │  Impostazioni · Cerca · Note · Nuova
//! └──────────────────────────────┘
//! ```
//!
//! Impostazioni, Cerca e il Cestino sono pagine a schermo intero (`pagine`);
//! Note apre la panoramica, tutte le note come schede grandi a griglia
//! (`schede::panoramica`). Trascinando il dito di lato sulla nota si passa
//! alla nota accanto (vedi `Programma::smista_gesti`).

pub mod barra_laterale;
pub mod barra_tastiera;
pub mod dialoghi;
pub mod editor;
pub mod pagine;
pub mod ricerca;
pub mod riga_stato;
pub mod schede;

use egui::{Align2, CentralPanel, Frame, Margin, Panel, Rect, Sense, Ui, UiBuilder, pos2};

use super::componenti::Icona;
use super::schermo;
use super::tema::Tavolozza;
use crate::applicazione::{Applicazione, Pagina};

pub const LARGHEZZA_BARRA: f32 = 300.0;
/// Larghezza del menu chiuso: solo i pulsanti « e lente.
pub const LARGHEZZA_BARRA_CHIUSA: f32 = schermo::misura(38.0, 50.0);

/// Sotto questa altezza (in punti), con la tastiera aperta, la nota mostra
/// solo il testo. Succede con il telefono in orizzontale: lo schermo e' alto
/// circa 360 punti e la tastiera ne prende piu' della meta'.
const ALTEZZA_PER_SCRIVERE: f32 = 300.0;

pub fn mostra(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    // Telefono: con la tastiera chiusa non si puo' scrivere, quindi la
    // lineetta lampeggiante del cursore distrae soltanto. Ricompare appena la
    // tastiera si apre (anche la selezione resta visibile).
    if schermo::TOCCO && !barra_tastiera::tastiera_aperta() {
        ui.visuals_mut().text_cursor.stroke.color = egui::Color32::TRANSPARENT;
    }
    // Il testo della nota e le maniglie della selezione le registra chi le
    // disegna, a ogni fotogramma (servono ai gesti del dito).
    schermo::imposta_zona_testo(ui.ctx(), None);
    schermo::imposta_maniglie_selezione(ui.ctx(), Vec::new());
    // Telefono, mentre si scrive: la barra sopra la tastiera.
    barra_tastiera::mostra(ui, app, t);
    app.interfaccia.scrittura_compatta =
        schermo::TOCCO && barra_tastiera::tastiera_aperta() && ui.available_height() < ALTEZZA_PER_SCRIVERE;
    if schermo::telefono(ui.ctx()) {
        telefono(ui, app, t);
        return;
    }
    // Disposizione del computer (anche telefono in orizzontale e tablet).
    app.interfaccia.pagina = Pagina::Nota;
    app.interfaccia.panoramica = false;
    schermo::zona_sfoglia(ui.ctx(), None);
    let fondo = Frame::new().fill(t.sfondo);
    let chiuso = app.preferenze.menu_chiuso;
    if app.interfaccia.scrittura_compatta {
        area_di_lavoro(ui, app, t, 6);
        return;
    }

    Panel::right("barra_laterale")
        .exact_size(if chiuso { LARGHEZZA_BARRA_CHIUSA } else { LARGHEZZA_BARRA })
        .resizable(false)
        .show_separator_line(false)
        .frame(fondo.inner_margin(Margin { left: 0, right: MARGINE_FINESTRA, top: MARGINE_FINESTRA, bottom: MARGINE_FINESTRA }))
        .show(ui, |ui| {
            if chiuso {
                ricerca::menu_chiuso(ui, app, t);
            } else {
                ricerca::mostra(ui, app, t);
                // Il menu comincia alla stessa altezza della nota, che sta subito
                // sotto le schede: niente scalino tra le due colonne. Il campo e'
                // piu' basso delle schede di `FESSURA`, lo spazio che resta sotto.
                ui.add_space(ricerca::FESSURA - ui.spacing().item_spacing.y);
                barra_laterale::mostra(ui, app, t);
            }
        });

    area_di_lavoro(ui, app, t, MARGINE_FINESTRA);
}

/// Computer: spazio attorno alla finestra e tra la nota e il menu. Gli spazi
/// dentro (tra le schede, sotto la ricerca) sono di 4 punti.
const MARGINE_FINESTRA: i8 = 8;

/// Schede, editor e riga di stato.
fn area_di_lavoro(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza, margine: i8) {
    CentralPanel::default()
        .frame(Frame::new().fill(t.sfondo).inner_margin(Margin::same(margine)))
        .show(ui, |ui| {
            if app.interfaccia.scrittura_compatta {
                // Solo il testo: schede, menu e riga di stato tornano chiudendo la tastiera.
                editor::mostra(ui, app, t);
                return;
            }
            let linguetta = egui::Id::unique("linguetta_aperta");
            ui.ctx().data_mut(|d| d.remove::<Rect>(linguetta));
            schede::mostra(ui, app, t);
            // Le schede sono linguette: la nota comincia subito sotto, cosi'
            // quella aperta si unisce alla nota (senza lo spazio tra gli elementi).
            ui.add_space(-ui.spacing().item_spacing.y);
            // Sul telefono una riga sotto la nota solo quando c'e' qualcosa da
            // dire (errore di salvataggio, ordine preso da un altro dispositivo).
            // Sul computer niente riga: la nota arriva fino in fondo, con i
            // contatori e i messaggi dentro (`editor`, `riga_stato`).
            if schermo::telefono(ui.ctx()) && riga_stato::ha_qualcosa_telefono(app) {
                Panel::bottom("piede")
                    .frame(Frame::new().inner_margin(Margin { left: 0, right: 0, top: 8, bottom: 0 }))
                    .show_separator_line(false)
                    .show(ui, |ui| riga_stato::mostra(ui, app, t));
            }
            editor::mostra(ui, app, t);
            // Computer: sotto la linguetta aperta il bordo del riquadro della
            // nota si interrompe (sul telefono la nota non ha riquadro).
            // La linguetta scende fino al riquadro (che comincia `inizio_editor`).
            if !schermo::telefono(ui.ctx()) {
                let inizio = ui.ctx().data(|d| d.get_temp::<f32>(egui::Id::unique("inizio_editor")));
                if let (Some(r), Some(alto)) = (ui.ctx().data(|d| d.get_temp::<Rect>(linguetta)), inizio) {
                    let fondo = alto + 1.5;
                    let varco = Rect::from_min_max(pos2(r.left() + 1.0, r.bottom() - 1.0), pos2(r.right() - 1.0, fondo));
                    let pittore = ui.painter();
                    pittore.rect_filled(varco, 0.0, t.pannello);
                    // I lati solo dove c'e' davvero spazio tra linguetta e riquadro:
                    // sopra i bordi gia' disegnati farebbero un alone.
                    if alto > r.bottom() + 0.5 {
                        for x in [r.left() + 0.5, r.right() - 0.5] {
                            pittore.line_segment([pos2(x, r.bottom()), pos2(x, alto)], t.tratto());
                        }
                    }
                }
            }
        });
}

/// Telefono in verticale: schede in alto e la nota (o la panoramica);
/// oppure una pagina a schermo intero. In basso, sopra la nota, il "+" a
/// sinistra, la maniglia al centro (verso l'alto: tutte le note) e ☐ a
/// destra (`editor`).
fn telefono(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    // Zone che registra chi le disegna, a ogni fotogramma: dove il dito
    // sfoglia le note (editor), il Cestino (riga della data o panoramica), la
    // maniglia. Chi trascina una scheda lo ridice ogni volta.
    schermo::zona_sfoglia(ui.ctx(), None);
    schermo::imposta_zona_cestino(ui.ctx(), None);
    schermo::imposta_zona_maniglia(ui.ctx(), None);
    schermo::imposta_zona_apri_panoramica(ui.ctx(), None);
    schermo::imposta_zona_chiudi_panoramica(ui.ctx(), None);
    schermo::imposta_area_pulsanti(ui.ctx(), None);
    let trascinava = schermo::si_trascina_nota(ui.ctx());
    schermo::imposta_trascina_nota(ui.ctx(), false);
    app.interfaccia.trascinava_nota = trascinava;
    schermo::imposta_zona_chiudi_pagina(ui.ctx(), None);
    let pagina = app.interfaccia.pagina;
    let ctx = ui.ctx().clone();
    // Ai lati della panoramica: il Cestino a sinistra, le Impostazioni a
    // destra. Seguono il dito che le tira di lato, altrimenti entrano ed
    // escono con un'animazione breve (anche con ⚙, il pulsante Cestino, «‹» o
    // Indietro).
    let tirata = schermo::scorri_laterale(&ctx);
    let di_lato = |quale: Pagina, chiave: &str| {
        let dito = tirata.filter(|(p, _)| *p == quale).map(|(_, v)| v);
        let obiettivo = dito.unwrap_or(if pagina == quale { 1.0 } else { 0.0 });
        let tempo = if dito.is_some() { 0.0 } else { DURATA_PANORAMICA };
        ctx.animate_value_with_time(egui::Id::unique(chiave), obiettivo, tempo)
    };
    let impostazioni = di_lato(Pagina::Impostazioni, "apertura_impostazioni");
    let cestino = di_lato(Pagina::Cestino, "apertura_cestino");
    // Aperta una nota del cestino, la sua "panoramica" e' la pagina Cestino:
    // sale dal basso sopra la nota, come la panoramica delle note.
    let nota_del_cestino =
        pagina == Pagina::Nota && app.interfaccia.selezionata.as_deref().is_some_and(|id| pagine::nel_cestino_visibile(app, id));
    if app.interfaccia.panoramica && !nota_del_cestino {
        // `verso`: da che parte sta la pagina (1 a destra, -1 a sinistra).
        for (quale, quanto, verso) in [(Pagina::Impostazioni, impostazioni, 1.0), (Pagina::Cestino, cestino, -1.0)] {
            if (pagina == Pagina::Nota || pagina == quale) && quanto > 0.001 && quanto < 0.999 {
                pagine_affiancate(ui, app, t, quale, quanto, verso);
                return;
            }
        }
    }
    // Impostazioni e Cestino tirati verso il basso: scendono seguendo il
    // dito e sotto c'e' la nota; lasciandoli oltre un terzo (o con un gesto
    // veloce) finiscono di scendere da soli (`pagina_uscente`).
    let dito_giu = schermo::chiudi_pagina(&ctx);
    let uscente = schermo::pagina_uscente(&ctx);
    let obiettivo = dito_giu.unwrap_or(if uscente.is_some() { 1.0 } else { 0.0 });
    let tempo = if dito_giu.is_some() { 0.0 } else { DURATA_PANORAMICA };
    let id_discesa = egui::Id::unique("discesa_pagina");
    let giu = ctx.animate_value_with_time(id_discesa, obiettivo, tempo);
    if uscente.is_some() && giu >= 0.999 {
        schermo::imposta_pagina_uscente(&ctx, None);
        ctx.animate_value_with_time(id_discesa, 0.0, 0.0);
    }
    let sopra = if pagina != Pagina::Nota { Some(pagina) } else { uscente.filter(|_| giu < 0.999) };
    let pagina_che_scende = sopra.filter(|_| giu > 0.001);
    if pagina != Pagina::Nota && pagina_che_scende.is_none() {
        CentralPanel::default()
            .frame(Frame::new().fill(t.sfondo).inner_margin(Margin::same(8)))
            .show(ui, |ui| pagine::mostra(ui, app, t, pagina));
        // ANNULLA dopo Elimina definitivamente (pagina Cestino).
        let area = ui.max_rect();
        avviso_annulla(ui.ctx(), app, t, area, 0.0);
        return;
    }
    if pagina_che_scende.is_some() && uscente.is_some() {
        // La pagina esce davvero: dietro c'e' la nota, non la panoramica, e
        // tornando alla panoramica questa salira' dal basso. (Mentre il dito
        // la tira soltanto, l'animazione resta com'e': se la pagina risale e
        // si passa di lato alla panoramica, questa e' gia' aperta.)
        ctx.animate_value_with_time(egui::Id::unique("apertura_panoramica"), 0.0, 0.0);
    }
    let area = ui.available_rect_before_wrap();
    let scrivendo = barra_tastiera::tastiera_aperta();
    avviso_uscita(ui.ctx(), app, t, area);
    // Quanto e' aperta la panoramica (0 chiusa, 1 aperta): segue il dito che
    // la trascina, altrimenti ci arriva da sola con un'animazione breve (anche
    // aprendola con un tocco sulla maniglia o chiudendola con ⌄ o Indietro).
    let trascina = schermo::trascina_panoramica(ui.ctx());
    let obiettivo = trascina.unwrap_or(if app.interfaccia.panoramica && pagina_che_scende.is_none() { 1.0 } else { 0.0 });
    let tempo = if trascina.is_some() || pagina_che_scende.is_some() { 0.0 } else { DURATA_PANORAMICA };
    let aperta_per = if pagina_che_scende.is_some() {
        // Dietro la pagina che scende c'e' la nota, non la panoramica.
        0.0
    } else {
        ui.ctx().animate_value_with_time(egui::Id::unique("apertura_panoramica"), obiettivo, tempo)
    };
    // (Solo salendo: aprendo una scheda del Cestino la pagina, che parte da
    // aperta, deve invece scendere e lasciare la nota.)
    if aperta_per >= 0.999 && nota_del_cestino && app.interfaccia.panoramica && trascina.is_none() {
        // Salita del tutto sopra la nota del cestino: ora e' la pagina Cestino
        // (gia' al suo posto, senza entrare di lato).
        app.interfaccia.pagina = Pagina::Cestino;
        ctx.animate_value_with_time(egui::Id::unique("apertura_cestino"), 1.0, 0.0);
        CentralPanel::default()
            .frame(Frame::new().fill(t.sfondo).inner_margin(Margin::same(8)))
            .show(ui, |ui| pagine::mostra(ui, app, t, Pagina::Cestino));
        avviso_annulla(ui.ctx(), app, t, area, 0.0);
        return;
    }
    if aperta_per >= 0.999 {
        CentralPanel::default()
            .frame(Frame::new().fill(t.sfondo).inner_margin(Margin::same(8)))
            .show(ui, |ui| schede::panoramica(ui, app, t));
        // Sopra il pulsante Cestino in fondo alla panoramica.
        avviso_annulla(ui.ctx(), app, t, area, schede::ALTEZZA_CESTINO_PANORAMICA + 8.0);
        return;
    }
    // "+", maniglia e ☐ si allineano su quest'area.
    schermo::imposta_area_pulsanti(ui.ctx(), Some(area));
    // Una nota lasciata vuota (creata con "+" senza scrivere nulla) sparisce
    // appena se ne apre un'altra.
    let aperta = app.interfaccia.selezionata.clone();
    if app.interfaccia.aperta_prima != aperta {
        if let Some(prima) = app.interfaccia.aperta_prima.take() {
            app.scarta_se_vuota(&prima);
        }
        app.interfaccia.aperta_prima = aperta;
    }
    // Margine 8 come la panoramica e le pagine (con 4 della nota: il testo a 12).
    area_di_lavoro(ui, app, t, 8);
    // Mentre si scrive restano solo il testo e ☐ sopra la tastiera.
    if !scrivendo && app.cartella.is_some() {
        pulsante_nuova_nota(ui.ctx(), app, t, area);
        maniglia(ui.ctx(), app, t, area);
    }
    // Annulla sopra i pulsanti fluttuanti, cosi' non li copre.
    let rialzo = if scrivendo { 0.0 } else { editor::LATO_PULSANTE + editor::MARGINE_PULSANTE };
    avviso_annulla(ui.ctx(), app, t, area, rialzo);
    // La nota del cestino tirata giu' dalla testata: scende sopra la nota a
    // cui si torna (gia' aperta sotto); sollevato il dito finisce di scendere
    // o risale, e allora torna aperta lei.
    if let Some(scende) = schermo::nota_che_scende(&ctx) {
        let dito = schermo::chiudi_pagina(&ctx);
        let obiettivo = dito.unwrap_or(if scende.esce { 1.0 } else { 0.0 });
        let tempo = if dito.is_some() { 0.0 } else { DURATA_PANORAMICA };
        let id_discesa = egui::Id::unique("discesa_nota_cestino");
        let quanto = ctx.animate_value_with_time(id_discesa, obiettivo, tempo);
        if dito.is_none() && (quanto - obiettivo).abs() < 0.001 {
            schermo::imposta_nota_che_scende(&ctx, None);
            ctx.animate_value_with_time(id_discesa, 0.0, 0.0);
            if !scende.esce && pagine::nel_cestino_visibile(app, &scende.id) {
                app.interfaccia.prima_del_cestino = app.interfaccia.selezionata.take();
                app.interfaccia.selezionata = Some(scende.id);
            }
            ctx.request_repaint();
        } else {
            nota_in_discesa(&ctx, app, t, area, scende.id, quanto);
        }
    }
    // A meta' strada la panoramica copre la nota dal basso, fin dove e' arrivata.
    // (Sopra una nota del cestino, la pagina Cestino.)
    if aperta_per > 0.001 {
        panoramica_in_movimento(ui.ctx(), app, t, area, aperta_per, nota_del_cestino);
    }
    if let Some(quale) = pagina_che_scende {
        pagina_in_discesa(ui.ctx(), app, t, area, quale, giu);
    }
}

/// La nota del cestino tirata giu': sotto la riga delle schede, spostata in
/// basso di `giu` (0-1) dell'altezza. Non si tocca.
fn nota_in_discesa(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza, area: Rect, id: String, giu: f32) {
    // Da dove comincia la nota (sotto le schede, `editor::mostra`).
    let inizio: f32 = ctx.data(|d| d.get_temp(egui::Id::unique("inizio_editor"))).unwrap_or(area.top());
    let nota = Rect::from_min_max(pos2(area.left(), inizio.max(area.top())), area.max);
    let posto = nota.translate(egui::vec2(0.0, nota.height() * giu));
    egui::Area::new(egui::Id::unique("nota_cestino_in_discesa"))
        .order(egui::Order::Foreground)
        .fade_in(false)
        .fixed_pos(posto.min)
        .interactable(false)
        .show(ctx, |ui| {
            ui.set_clip_rect(nota);
            ui.painter().rect_filled(posto, 0.0, t.sfondo);
            ui.painter().line_segment([posto.left_top(), posto.right_top()], t.tratto());
            // Con lo stesso margine della nota aperta (`area_di_lavoro`).
            let dentro = posto.shrink2(egui::vec2(8.0, 0.0));
            ui.scope_builder(egui::UiBuilder::new().max_rect(dentro), |ui| {
                ui.set_min_size(dentro.size());
                editor::altra_nota(ui, app, t, id);
            });
        });
}

/// Quanto dura (secondi) l'apertura o la chiusura della panoramica, quando
/// nessun dito la trascina.
const DURATA_PANORAMICA: f32 = 0.22;

/// La panoramica e una pagina affiancate mentre la pagina entra o esce di
/// lato: `quanto` la pagina si vede (0-1), `verso` 1 se sta a destra (le
/// Impostazioni), -1 se sta a sinistra (il Cestino). Non si toccano.
fn pagine_affiancate(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza, quale: Pagina, quanto: f32, verso: f32) {
    CentralPanel::default().frame(Frame::new().fill(t.sfondo).inner_margin(Margin::same(8))).show(ui, |ui| {
        let area = ui.available_rect_before_wrap();
        // Tra le due pagine lo stesso margine che hanno dal bordo.
        let passo = area.width() + 16.0;
        let ritaglio = ui.clip_rect();
        let mut panoramica =
            ui.new_child(UiBuilder::new().max_rect(area.translate(egui::vec2(-verso * passo * quanto, 0.0))).id_salt("panoramica_di_lato"));
        panoramica.set_clip_rect(ritaglio);
        schede::panoramica(&mut panoramica, app, t);
        let mut accanto =
            ui.new_child(UiBuilder::new().max_rect(area.translate(egui::vec2(verso * passo * (1.0 - quanto), 0.0))).id_salt("pagina_di_lato"));
        accanto.set_clip_rect(ritaglio);
        pagine::mostra(&mut accanto, app, t, quale);
    });
}

/// Impostazioni o Cestino tirati verso il basso: disegnati sopra la nota,
/// spostati in basso di `giu` (0-1) dell'altezza. Non si toccano.
fn pagina_in_discesa(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza, area: Rect, quale: Pagina, giu: f32) {
    let posto = area.translate(egui::vec2(0.0, area.height() * giu));
    egui::Area::new(egui::Id::unique("pagina_in_discesa"))
        .order(egui::Order::Foreground)
        .fade_in(false)
        .fixed_pos(posto.min)
        .interactable(false)
        .show(ctx, |ui| {
            ui.set_clip_rect(area);
            ui.painter().rect_filled(posto, 0.0, t.sfondo);
            ui.painter().line_segment([posto.left_top(), posto.right_top()], t.tratto());
            let dentro = posto.shrink(8.0);
            ui.scope_builder(egui::UiBuilder::new().max_rect(dentro), |ui| {
                ui.set_min_size(dentro.size());
                pagine::mostra(ui, app, t, quale);
            });
        });
}

/// La panoramica mentre si apre o si chiude: disegnata sopra la nota,
/// spostata in basso di quanto manca ad aprirla. Non si tocca finche' non e'
/// aperta del tutto.
fn panoramica_in_movimento(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza, area: Rect, aperta_per: f32, cestino: bool) {
    let sotto = area.height() * (1.0 - aperta_per);
    let posto = area.translate(egui::vec2(0.0, sotto));
    egui::Area::new(egui::Id::unique("panoramica_in_movimento"))
        .order(egui::Order::Foreground)
        // Subito piena: con la dissolvenza di egui, appena il dito comincia a
        // tirarla giu' si vedrebbero per un attimo la nota e i pulsanti sotto.
        .fade_in(false)
        .fixed_pos(posto.min)
        .interactable(false)
        .show(ctx, |ui| {
            ui.set_clip_rect(area);
            ui.painter().rect_filled(posto, 0.0, t.sfondo);
            ui.painter().line_segment([posto.left_top(), posto.right_top()], t.tratto());
            let dentro = posto.shrink(8.0);
            ui.scope_builder(egui::UiBuilder::new().max_rect(dentro), |ui| {
                ui.set_min_size(dentro.size());
                if cestino {
                    pagine::mostra(ui, app, t, Pagina::Cestino);
                } else {
                    schede::panoramica(ui, app, t);
                }
            });
        });
}

/// Quanto resta l'avviso "Premere di nuovo Indietro per uscire" (e quanto
/// tempo c'e' per premere di nuovo).
pub const DURATA_AVVISO_USCITA: std::time::Duration = std::time::Duration::from_secs(2);

/// Telefono: dopo il primo Indietro sulla nota, un avviso piccolo in basso.
fn avviso_uscita(ctx: &egui::Context, app: &Applicazione, t: &Tavolozza, area: Rect) {
    let Some(quando) = app.interfaccia.uscita_chiesta.filter(|q| q.elapsed() < DURATA_AVVISO_USCITA) else { return };
    let testo = app.tr.t("avviso_uscita");
    let galleria = ctx.fonts_mut(|f| f.layout_no_wrap(testo.to_owned(), egui::FontId::proportional(14.0), t.sfondo));
    let dimensione = galleria.size() + egui::vec2(32.0, 20.0);
    let posto = Rect::from_center_size(
        pos2(area.center().x, area.bottom() - editor::LATO_PULSANTE - editor::MARGINE_PULSANTE - 40.0),
        dimensione,
    );
    egui::Area::new(egui::Id::unique("avviso_uscita"))
        .order(egui::Order::Foreground)
        .fixed_pos(posto.min)
        .interactable(false)
        .show(ctx, |ui| {
            let (r, _) = ui.allocate_exact_size(dimensione, Sense::hover());
            ui.painter().rect_filled(r, egui::CornerRadius::same(super::tema::RAGGIO_CONTENITORI), t.testo);
            ui.painter().galley(r.center() - galleria.size() / 2.0, galleria, t.sfondo);
        });
    ctx.request_repaint_after(DURATA_AVVISO_USCITA.saturating_sub(quando.elapsed()));
}

/// Il "+" fluttuante in basso a sinistra: una nota nuova (come Ctrl+N).
fn pulsante_nuova_nota(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza, area: Rect) {
    let lato = editor::LATO_PULSANTE;
    let margine = editor::MARGINE_PULSANTE;
    let posto = Rect::from_min_size(pos2(area.left() + margine, area.bottom() - margine - lato), egui::Vec2::splat(lato));
    let premuto = egui::Area::new(egui::Id::unique("pulsante_nuova_nota"))
        .order(egui::Order::Middle)
        .fixed_pos(posto.min)
        .show(ctx, |ui| {
            let (r, risposta) = ui.allocate_exact_size(posto.size(), Sense::click());
            super::componenti::disegna_pulsante_fluttuante(ui.painter(), r, t, risposta.is_pointer_button_down_on(), Icona::Piu);
            risposta.on_hover_text(app.tr.t("aiuto_nuova_nota")).clicked()
        })
        .inner;
    if premuto {
        app.conferma_rinomina();
        app.nuova_nota();
    }
}

/// La maniglia in basso al centro: un tocco, o un trascinamento verso l'alto
/// che parte da li' (vedi `Programma::smista_gesti`), apre tutte le note.
fn maniglia(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza, area: Rect) {
    let zona = Rect::from_center_size(
        pos2(area.center().x, area.bottom() - editor::MARGINE_PULSANTE - editor::LATO_PULSANTE / 2.0),
        egui::vec2(120.0, editor::LATO_PULSANTE),
    );
    // Il gesto verso l'alto parte da tutta la fascia in basso tra il "+" e ☐,
    // non solo dalla lineetta: e' piu' comodo da prendere con il pollice.
    let lato = editor::LATO_PULSANTE;
    let margine = editor::MARGINE_PULSANTE;
    let fascia = Rect::from_min_max(
        pos2(area.left() + margine + lato + 8.0, area.bottom() - lato - 2.0 * margine - 16.0),
        pos2(area.right() - margine - lato - 8.0, area.bottom()),
    );
    schermo::imposta_zona_maniglia(ctx, Some(fascia));
    let premuto = egui::Area::new(egui::Id::unique("maniglia_note"))
        .order(egui::Order::Middle)
        .fixed_pos(zona.min)
        .show(ctx, |ui| {
            let (r, risposta) = ui.allocate_exact_size(zona.size(), Sense::click_and_drag());
            super::componenti::disegna_lineetta(ui.painter(), r.center(), t, risposta.is_pointer_button_down_on());
            // Anche il trascinamento lento (il gesto veloce lo riconosce `smista_gesti`).
            let su = ui.input(|i| match (i.pointer.press_origin(), i.pointer.latest_pos()) {
                (Some(o), Some(p)) => r.contains(o) && o.y - p.y > 30.0,
                _ => false,
            });
            let trascinata = risposta.dragged() && su;
            // Nessuna descrizione: comparirebbe sotto il pollice mentre trascina.
            risposta.clicked() || trascinata
        })
        .inner;
    if premuto {
        app.conferma_rinomina();
        app.interfaccia.panoramica = true;
    }
}
/// Telefono: dopo aver messo una nota nel cestino, un riquadro grande sopra
/// la barra, nei colori rovesciati perche' si noti: "«Spese» nel cestino" e
/// ANNULLA. La linea in basso si accorcia con il tempo che resta.
fn avviso_annulla(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza, sopra: Rect, rialzo: f32) {
    let titolo_di = |app: &Applicazione, id: &str| app.nota(id).map(|n| n.titolo_visibile(app.tr.t("senza_titolo")).to_owned()).unwrap_or_default();
    // Prima l'eliminazione definitiva dal Cestino ("«Spese» eliminata",
    // "12 note eliminate"), poi lo spostamento nel cestino.
    let (scritta, rimasto, eliminazione) = if let Some((ids, rimasto)) = app.eliminazione_annullabile() {
        let scritta = match ids {
            [id] => app.tr.f("nota_eliminata", &[("titolo", &titolo_di(app, id))]),
            _ => app.tr.f("note_eliminate", &[("n", &ids.len().to_string())]),
        };
        (scritta, rimasto, true)
    } else if let (Some(rimasto), Some(id)) = (app.annulla_cestino_rimasto(), app.cancellazione_annullabile()) {
        (app.tr.f("avviso_cestino", &[("titolo", &titolo_di(app, id))]), rimasto, false)
    } else {
        return;
    };
    let annulla = app.tr.t("annulla").to_uppercase();
    const ALTO: f32 = 58.0;
    const MARGINE: f32 = 10.0;
    let posto = Rect::from_min_max(
        pos2(sopra.left() + MARGINE, sopra.bottom() - rialzo - ALTO - MARGINE),
        pos2(sopra.right() - MARGINE, sopra.bottom() - rialzo - MARGINE),
    );
    // Toccarlo mentre si scrive non chiude la tastiera.
    schermo::proteggi_fuoco(ctx, posto);
    let premuto = egui::Area::new(egui::Id::unique("avviso_annulla"))
        .order(egui::Order::Foreground)
        .fixed_pos(posto.min)
        .show(ctx, |ui| {
            let (r, _) = ui.allocate_exact_size(posto.size(), Sense::hover());
            let pulsante = Rect::from_min_max(pos2(r.right() - 124.0, r.top() + 8.0), pos2(r.right() - 8.0, r.bottom() - 13.0));
            let risposta = ui.interact(pulsante, egui::Id::unique("avviso_annulla_pulsante"), Sense::click());
            let pittore = ui.painter();
            pittore.rect_filled(r, egui::CornerRadius::same(super::tema::RAGGIO_CONTENITORI), t.testo);
            let fondo = if risposta.is_pointer_button_down_on() { t.accento.gamma_multiply(0.85) } else { t.accento };
            pittore.rect_filled(pulsante, egui::CornerRadius::same(super::tema::RAGGIO), fondo);
            pittore.text(pulsante.center(), Align2::CENTER_CENTER, &annulla, super::tema::carattere_titolo_piccolo(), t.testo_accento);
            let mut lavoro = egui::text::LayoutJob::simple_singleline(scritta.clone(), egui::FontId::proportional(15.0), t.sfondo);
            lavoro.wrap = egui::text::TextWrapping::truncate_at_width(pulsante.left() - r.left() - 26.0);
            let galleria = ui.fonts_mut(|f| f.layout_job(lavoro));
            let y = (r.top() + r.bottom() - 5.0) / 2.0 - galleria.size().y / 2.0;
            ui.painter().galley(pos2(r.left() + 14.0, y), galleria, t.sfondo);
            // Il tempo che resta.
            let piena = r.width() - 24.0;
            let linea = Rect::from_min_size(pos2(r.left() + 12.0, r.bottom() - 6.0), egui::vec2(piena * rimasto, 3.0));
            ui.painter().rect_filled(linea, egui::CornerRadius::same(2), t.accento);
            risposta.clicked()
        })
        .inner;
    if premuto && eliminazione {
        app.annulla_eliminazione();
    } else if premuto {
        app.annulla_cancellazione();
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(50));
}
