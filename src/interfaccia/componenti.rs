//! Controlli riutilizzati da finestra principale e widget.
//!
//! Le icone sono disegnate con forme geometriche invece che con simboli di
//! testo: appaiono identiche su Windows, Linux e macOS, qualunque carattere
//! sia installato.

use egui::{
    Align2, Button, Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect, Response, RichText, Sense, Shape,
    Stroke, StrokeKind, Ui, Vec2, pos2, vec2,
};

use super::schermo::misura;
use super::tema::{RAGGIO, RAGGIO_CONTENITORI, TESTO_SECONDARIO, TITOLO_DIALOGO, TRATTO_ATTIVO, Tavolozza};

// Stile comune dei pulsanti, lo stesso delle schede: il fondo e' quello su
// cui stanno (la nota, il menu), il bordo sottile e neutro, gli angoli di
// `RAGGIO`; passando sopra o premendo il fondo si schiarisce. Il colore sta
// solo nella scritta o nell'icona, dove dice qualcosa: rosso per Cancella e
// il Cestino, verde per Ripristina, il colore principale per l'azione piu'
// importante e per cio' che e' attivo. I pulsanti `Button` di egui prendono
// lo stesso stile da `tema::applica`.

/// Fondo di un pulsante: niente (si vede quello sotto), un po' piu' chiaro
/// sotto il mouse o il dito.
pub fn fondo_pulsante(t: &Tavolozza, sopra: bool) -> Color32 {
    if sopra { t.superficie_attiva } else { Color32::TRANSPARENT }
}

/// Fondo e bordo di un pulsante disegnato a mano.
pub fn disegna_fondo_pulsante(pittore: &egui::Painter, r: Rect, t: &Tavolozza, sopra: bool) {
    pittore.rect(r, CornerRadius::same(RAGGIO), fondo_pulsante(t, sopra), t.tratto(), StrokeKind::Inside);
}

/// Pulsante standard.
pub fn pulsante(ui: &mut Ui, t: &Tavolozza, testo: &str) -> Response {
    ui.add(Button::new(RichText::new(testo).color(t.testo)).corner_radius(RAGGIO).min_size(vec2(0.0, ALTEZZA_PULSANTE)))
}

/// Pulsante che occupa tutta la larghezza disponibile.
pub fn pulsante_largo(ui: &mut Ui, t: &Tavolozza, testo: &str) -> Response {
    let larghezza = ui.available_width();
    ui.add(Button::new(RichText::new(testo).color(t.testo)).corner_radius(RAGGIO).min_size(vec2(larghezza, ALTEZZA_PULSANTE)))
}

/// Pulsante dell'azione principale: come gli altri, con la scritta nel colore
/// principale.
pub fn pulsante_accento(ui: &mut Ui, t: &Tavolozza, testo: &str) -> Response {
    ui.add(
        Button::new(RichText::new(testo).color(t.accento).strong())
            .corner_radius(RAGGIO)
            .min_size(vec2(0.0, ALTEZZA_PULSANTE_ACCENTO)),
    )
}

/// Pulsante con la scritta colorata (Cancella in rosso, Ripristina in verde).
pub fn pulsante_colorato(ui: &mut Ui, _t: &Tavolozza, testo: &str, colore: Color32) -> Response {
    ui.add(Button::new(RichText::new(testo).color(colore).strong()).corner_radius(RAGGIO).min_size(vec2(0.0, ALTEZZA_PULSANTE)))
}

/// Come `pulsante_colorato`, ma su tutta la larghezza disponibile.
pub fn pulsante_largo_colorato(ui: &mut Ui, _t: &Tavolozza, testo: &str, colore: Color32) -> Response {
    let larghezza = ui.available_width();
    ui.add(Button::new(RichText::new(testo).color(colore).strong()).corner_radius(RAGGIO).min_size(vec2(larghezza, ALTEZZA_PULSANTE)))
}

/// Una scelta tra poche (tema, lingua) o un interruttore a pulsante (Mostra
/// il codice): come un pulsante; quando e' scelta, scritta e bordo sono nel
/// colore principale.
pub fn scelta(ui: &mut Ui, t: &Tavolozza, testo: &str, scelto: bool) -> Response {
    let scritta = if scelto { RichText::new(testo).color(t.accento).strong() } else { RichText::new(testo).color(t.testo) };
    let mut pulsante = Button::new(scritta).corner_radius(RAGGIO).min_size(vec2(0.0, ALTEZZA_PULSANTE));
    if scelto {
        pulsante = pulsante.stroke(Stroke::new(TRATTO_ATTIVO, t.accento));
    }
    ui.add(pulsante)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Icona {
    PrimoPianoAttivo,
    PrimoPianoSpento,
    Riduci,
    Chiudi,
    Precedente,
    Successivo,
    Ridimensiona,
    /// Puntina: nota fissata (piena) o no (vuota).
    Puntina,
    PuntinaAttiva,
    /// Orologio: cronologia delle versioni.
    Orologio,
    /// Casella spuntata: voce di lista.
    Lista,
    /// Piu': nuova nota.
    Piu,
    /// Lente: ricerca.
    Lente,
    /// Finestrella con la barra in alto: apri come widget.
    Widget,
    /// Tre righe: apre il menu (telefono).
    Menu,
    /// Freccia che torna indietro: annulla l'ultima modifica.
    Annulla,
    /// Freccia che va avanti: ripete la modifica annullata.
    Ripeti,
    /// Freccia in giu': chiude la tastiera del telefono.
    Giu,
    /// Tre puntini in colonna: altre azioni sulla nota (telefono).
    Altro,
    /// Ingranaggio: impostazioni (telefono).
    Ingranaggio,
    /// Cestino dei rifiuti.
    Cestino,
}

/// Altezza dei pulsanti con testo: piu' alti sul telefono, per il dito.
pub const ALTEZZA_PULSANTE: f32 = misura(30.0, 42.0);
const ALTEZZA_PULSANTE_ACCENTO: f32 = misura(32.0, 44.0);
/// Lato dei pulsanti quadrati con icona (sul telefono come i quadrati della
/// riga della data e della barra sopra la tastiera).
pub const LATO_ICONA: f32 = misura(26.0, 40.0);

/// Pulsante quadrato con icona disegnata e suggerimento al passaggio del mouse.
pub fn pulsante_icona(ui: &mut Ui, t: &Tavolozza, icona: Icona, suggerimento: &str, attivo: bool) -> Response {
    pulsante_icona_lato(ui, t, icona, suggerimento, attivo, LATO_ICONA)
}

/// Come `pulsante_icona`, con il lato scelto.
pub fn pulsante_icona_lato(ui: &mut Ui, t: &Tavolozza, icona: Icona, suggerimento: &str, attivo: bool, lato: f32) -> Response {
    icona_colorata(ui, t, icona, suggerimento, attivo, lato, t.testo)
}

/// Pulsante con l'icona nel colore principale: il "+" di una nota nuova,
/// ovunque si trovi (come il "+" delle schede e quello fluttuante).
pub fn pulsante_icona_accento(ui: &mut Ui, t: &Tavolozza, icona: Icona, suggerimento: &str) -> Response {
    icona_colorata(ui, t, icona, suggerimento, true, LATO_ICONA, t.accento)
}

fn icona_colorata(ui: &mut Ui, t: &Tavolozza, icona: Icona, suggerimento: &str, attivo: bool, lato: f32, colore: Color32) -> Response {
    let senso = if attivo { Sense::click() } else { Sense::hover() };
    let (rettangolo, risposta) = ui.allocate_exact_size(Vec2::splat(lato), senso);
    let sopra = attivo && risposta.hovered();
    let pittore = ui.painter();
    if icona != Icona::Ridimensiona {
        disegna_fondo_pulsante(pittore, rettangolo, t, sopra || (attivo && risposta.is_pointer_button_down_on()));
    }
    let colore = if attivo { colore } else { t.bordo };
    disegna_icona(pittore, rettangolo.shrink(lato * 0.27), icona, colore);
    if attivo {
        risposta.on_hover_text(suggerimento)
    } else {
        risposta
    }
}

pub fn disegna_icona(pittore: &egui::Painter, r: Rect, icona: Icona, colore: Color32) {
    let tratto = Stroke::new(1.6, colore);
    let c = r.center();
    match icona {
        Icona::PrimoPianoAttivo => {
            pittore.circle_filled(c, r.width() * 0.42, colore);
        }
        Icona::PrimoPianoSpento => {
            pittore.circle_stroke(c, r.width() * 0.40, tratto);
        }
        Icona::Riduci => {
            pittore.line_segment([pos2(r.left(), c.y), pos2(r.right(), c.y)], tratto);
        }
        Icona::Chiudi => {
            pittore.line_segment([r.left_top(), r.right_bottom()], tratto);
            pittore.line_segment([r.right_top(), r.left_bottom()], tratto);
        }
        Icona::Precedente | Icona::Successivo => {
            let verso = if icona == Icona::Precedente { -1.0 } else { 1.0 };
            let punta = pos2(c.x + verso * r.width() * 0.25, c.y);
            let dietro = c.x - verso * r.width() * 0.25;
            pittore.line_segment([pos2(dietro, r.top()), punta], tratto);
            pittore.line_segment([punta, pos2(dietro, r.bottom())], tratto);
        }
        Icona::Ridimensiona => {
            for passo in [0.0, 0.35, 0.7] {
                let d = r.width() * passo;
                pittore.line_segment([pos2(r.right() - d, r.bottom()), pos2(r.right(), r.bottom() - d)], tratto);
            }
        }
        Icona::Puntina | Icona::PuntinaAttiva => {
            // Puntina da bacheca vista di lato: corpo, base larga e ago.
            let corpo = Rect::from_min_max(
                pos2(c.x - r.width() * 0.22, r.top()),
                pos2(c.x + r.width() * 0.22, r.top() + r.height() * 0.45),
            );
            let base = r.top() + r.height() * 0.55;
            if icona == Icona::PuntinaAttiva {
                pittore.rect_filled(corpo, CornerRadius::same(1), colore);
            } else {
                pittore.rect_stroke(corpo, CornerRadius::same(1), tratto, StrokeKind::Inside);
            }
            pittore.line_segment([pos2(c.x, corpo.bottom()), pos2(c.x, base)], tratto);
            pittore.line_segment([pos2(c.x - r.width() * 0.42, base), pos2(c.x + r.width() * 0.42, base)], tratto);
            pittore.line_segment([pos2(c.x, base), pos2(c.x, r.bottom())], tratto);
        }
        Icona::Orologio => {
            pittore.circle_stroke(c, r.width() * 0.5, tratto);
            pittore.line_segment([c, pos2(c.x, r.top() + r.height() * 0.18)], tratto);
            pittore.line_segment([c, pos2(c.x + r.width() * 0.25, c.y + r.height() * 0.1)], tratto);
        }
        Icona::Lista => {
            pittore.rect_stroke(r, CornerRadius::same(2), tratto, StrokeKind::Inside);
            pittore.line_segment([pos2(r.left() + r.width() * 0.2, c.y), pos2(r.left() + r.width() * 0.42, r.bottom() - r.height() * 0.22)], tratto);
            pittore.line_segment([pos2(r.left() + r.width() * 0.42, r.bottom() - r.height() * 0.22), pos2(r.right() - r.width() * 0.15, r.top() + r.height() * 0.2)], tratto);
        }
        Icona::Piu => {
            let tratto = Stroke::new(2.0, colore);
            pittore.line_segment([pos2(c.x, r.top()), pos2(c.x, r.bottom())], tratto);
            pittore.line_segment([pos2(r.left(), c.y), pos2(r.right(), c.y)], tratto);
        }
        Icona::Lente => {
            let raggio = r.width() * 0.32;
            let centro = pos2(r.left() + raggio + 0.5, r.top() + raggio + 0.5);
            pittore.circle_stroke(centro, raggio, tratto);
            let d = raggio * std::f32::consts::FRAC_1_SQRT_2;
            pittore.line_segment([pos2(centro.x + d, centro.y + d), r.right_bottom()], Stroke::new(2.0, colore));
        }
        Icona::Annulla | Icona::Ripeti => {
            // Mezzo cerchio con la punta della freccia a un capo.
            let verso = if icona == Icona::Annulla { 1.0 } else { -1.0 };
            let raggio = r.width() * 0.38;
            let centro = pos2(c.x, c.y + r.height() * 0.08);
            let punti: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let a = std::f32::consts::PI * (i as f32 / 12.0);
                    pos2(centro.x + verso * raggio * a.cos(), centro.y - raggio * a.sin())
                })
                .collect();
            let punta = punti[12];
            pittore.add(Shape::line(punti, tratto));
            // La punta: un triangolo pieno che scende dal capo dell'arco.
            let d = r.width() * 0.3;
            let triangolo = vec![
                pos2(punta.x - d * 0.55, punta.y - d * 0.15),
                pos2(punta.x + d * 0.55, punta.y - d * 0.15),
                pos2(punta.x, punta.y + d * 0.65),
            ];
            pittore.add(Shape::convex_polygon(triangolo, colore, Stroke::NONE));
        }
        Icona::Giu => {
            let alto = c.y - r.height() * 0.18;
            let basso = c.y + r.height() * 0.18;
            pittore.line_segment([pos2(r.left(), alto), pos2(c.x, basso)], tratto);
            pittore.line_segment([pos2(c.x, basso), pos2(r.right(), alto)], tratto);
        }
        Icona::Ingranaggio => {
            // Un anello con otto denti attorno.
            let raggio = r.width() * 0.30;
            pittore.circle_stroke(c, raggio, tratto);
            pittore.circle_stroke(c, r.width() * 0.11, tratto);
            let denti = Stroke::new(2.4, colore);
            for i in 0..8 {
                let a = std::f32::consts::TAU * i as f32 / 8.0;
                let verso = egui::vec2(a.cos(), a.sin());
                pittore.line_segment([c + verso * raggio, c + verso * r.width() * 0.48], denti);
            }
        }
        Icona::Cestino => {
            // Coperchio con la maniglia, e il secchio che si stringe in basso.
            let alto = r.top() + r.height() * 0.22;
            pittore.line_segment([pos2(r.left(), alto), pos2(r.right(), alto)], tratto);
            let maniglia = [
                pos2(c.x - r.width() * 0.16, alto),
                pos2(c.x - r.width() * 0.16, r.top()),
                pos2(c.x + r.width() * 0.16, r.top()),
                pos2(c.x + r.width() * 0.16, alto),
            ];
            pittore.add(Shape::line(maniglia.to_vec(), tratto));
            let secchio = vec![
                pos2(r.left() + r.width() * 0.12, alto),
                pos2(r.left() + r.width() * 0.2, r.bottom()),
                pos2(r.right() - r.width() * 0.2, r.bottom()),
                pos2(r.right() - r.width() * 0.12, alto),
            ];
            pittore.add(Shape::line(secchio, tratto));
            for x in [-0.12, 0.12] {
                let x = c.x + r.width() * x;
                pittore.line_segment([pos2(x, alto + r.height() * 0.18), pos2(x, r.bottom() - r.height() * 0.14)], tratto);
            }
        }
        Icona::Altro => {
            for parte in [0.1, 0.5, 0.9] {
                pittore.circle_filled(pos2(c.x, r.top() + r.height() * parte), 1.9, colore);
            }
        }
        Icona::Menu => {
            for parte in [0.15, 0.5, 0.85] {
                let y = r.top() + r.height() * parte;
                pittore.line_segment([pos2(r.left(), y), pos2(r.right(), y)], tratto);
            }
        }
        Icona::Widget => {
            let finestra = Rect::from_min_max(pos2(r.left(), r.top() + r.height() * 0.1), pos2(r.right(), r.bottom() - r.height() * 0.1));
            pittore.rect_stroke(finestra, CornerRadius::same(2), tratto, StrokeKind::Inside);
            let barra = Rect::from_min_max(finestra.min, pos2(finestra.right(), finestra.top() + finestra.height() * 0.3));
            pittore.rect_filled(barra, CornerRadius { nw: 2, ne: 2, sw: 0, se: 0 }, colore);
        }
    }
}

/// Carattere della scritta dei pulsanti con icona.
fn carattere_strumento() -> FontId {
    FontId::proportional(14.0)
}

/// Larghezza di un pulsante con icona (e scritta, se c'e') alto `altezza`.
pub fn larghezza_strumento(ui: &Ui, testo: Option<&str>, altezza: f32) -> f32 {
    match testo {
        None => altezza,
        Some(testo) => {
            let scritta = ui.painter().layout_no_wrap(testo.to_owned(), carattere_strumento(), Color32::WHITE).size().x;
            10.0 + 15.0 + 7.0 + scritta + 12.0
        }
    }
}

/// Fondo e contorno di un pulsante con icona; restituisce il colore
/// dell'icona (quello principale per l'azione piu' importante).
fn fondo_strumento(ui: &Ui, t: &Tavolozza, r: Rect, sopra: bool, accento: bool) -> Color32 {
    disegna_fondo_pulsante(ui.painter(), r, t, sopra);
    if accento { t.accento } else { t.testo }
}

/// Rettangolo dell'icona: a sinistra se c'e' la scritta, altrimenti al centro.
fn posto_icona(r: Rect, con_testo: bool) -> Rect {
    let lato = 15.0;
    if con_testo {
        Rect::from_min_size(pos2(r.left() + 10.0, r.center().y - lato / 2.0), Vec2::splat(lato))
    } else {
        Rect::from_center_size(r.center(), Vec2::splat(lato))
    }
}

fn scritta_strumento(ui: &Ui, r: Rect, testo: &str, colore: Color32) {
    ui.painter().text(pos2(r.left() + 10.0 + 15.0 + 7.0, r.center().y), Align2::LEFT_CENTER, testo, carattere_strumento(), colore);
}

/// Pulsante con icona e scritta (o solo icona, se lo spazio non basta), con
/// la descrizione al passaggio del mouse. `accento`: pieno nel colore
/// principale, per l'azione piu' usata.
pub fn pulsante_strumento(
    ui: &mut Ui,
    t: &Tavolozza,
    icona: Icona,
    testo: Option<&str>,
    descrizione: &str,
    altezza: f32,
    accento: bool,
) -> Response {
    let larghezza = larghezza_strumento(ui, testo, altezza);
    let (r, risposta) = ui.allocate_exact_size(vec2(larghezza, altezza), Sense::click());
    let colore = fondo_strumento(ui, t, r, risposta.hovered() || risposta.is_pointer_button_down_on(), accento);
    disegna_icona(ui.painter(), posto_icona(r, testo.is_some()), icona, colore);
    if let Some(testo) = testo {
        scritta_strumento(ui, r, testo, t.testo);
    }
    risposta.on_hover_text(descrizione)
}

/// Contenuto di una parte della barra degli strumenti.
pub enum Simbolo<'a> {
    Icona(Icona),
    /// Pallino del colore della nota (nome del colore, vuoto per nessuno).
    Colore(&'a str),
}

pub struct Strumento<'a> {
    pub simbolo: Simbolo<'a>,
    pub testo: Option<&'a str>,
    pub descrizione: &'a str,
    /// Spento: disegnato in grigio, non si puo' premere.
    pub attivo: bool,
}

/// Larghezza della barra degli strumenti.
pub fn larghezza_barra_strumenti(ui: &Ui, strumenti: &[Strumento], altezza: f32) -> f32 {
    strumenti.iter().map(|s| larghezza_strumento(ui, s.testo, altezza)).sum()
}

/// Barra degli strumenti: un solo riquadro arrotondato diviso in parti da
/// sottili linee; al passaggio del mouse si colora solo la parte sotto il
/// mouse. Restituisce una risposta per ogni strumento, nello stesso ordine.
pub fn barra_strumenti(ui: &mut Ui, t: &Tavolozza, id: egui::Id, strumenti: &[Strumento], altezza: f32) -> Vec<Response> {
    let totale = larghezza_barra_strumenti(ui, strumenti, altezza);
    let (barra, _) = ui.allocate_exact_size(vec2(totale, altezza), Sense::hover());
    let pittore = ui.painter().clone();
    let mut risposte = Vec::with_capacity(strumenti.len());
    let mut x = barra.left();
    let ultimo = strumenti.len().saturating_sub(1);
    for (i, strumento) in strumenti.iter().enumerate() {
        let larghezza = larghezza_strumento(ui, strumento.testo, altezza);
        let r = Rect::from_min_size(pos2(x, barra.top()), vec2(larghezza, altezza));
        let senso = if strumento.attivo { Sense::click() } else { Sense::hover() };
        let risposta = ui.interact(r, id.with(i), senso);
        let colore = if strumento.attivo { t.testo } else { t.bordo };
        if strumento.attivo && risposta.hovered() {
            let angoli = CornerRadius {
                nw: if i == 0 { RAGGIO } else { 0 },
                sw: if i == 0 { RAGGIO } else { 0 },
                ne: if i == ultimo { RAGGIO } else { 0 },
                se: if i == ultimo { RAGGIO } else { 0 },
            };
            pittore.rect_filled(r, angoli, t.superficie_attiva);
        }
        let posto = posto_icona(r, strumento.testo.is_some());
        match strumento.simbolo {
            Simbolo::Icona(icona) => disegna_icona(&pittore, posto, icona, colore),
            Simbolo::Colore(colore) => disegna_campione(&pittore, posto.center(), 6.5, colore, t, false, false),
        }
        if let Some(testo) = strumento.testo {
            scritta_strumento(ui, r, testo, colore);
        }
        if i > 0 {
            pittore.line_segment([pos2(x, barra.top() + 7.0), pos2(x, barra.bottom() - 7.0)], t.tratto());
        }
        risposte.push(if strumento.attivo { risposta.on_hover_text(strumento.descrizione) } else { risposta });
        x += larghezza;
    }
    // Il contorno sopra tutto, cosi' le parti colorate non lo coprono.
    pittore.rect_stroke(barra, CornerRadius::same(RAGGIO), t.tratto(), StrokeKind::Inside);
    risposte
}

/// Pulsante con il colore della nota; un clic apre una riga di colori.
/// `etichetta` aggiunge la scritta accanto al pallino. Restituisce `true` se
/// il colore e' cambiato.
pub fn scelta_colore(
    ui: &mut Ui,
    t: &Tavolozza,
    colore: &mut String,
    tr: &crate::lingue::Traduttore,
    etichetta: Option<&str>,
    altezza: f32,
) -> bool {
    let larghezza = larghezza_strumento(ui, etichetta, altezza);
    let (r, risposta) = ui.allocate_exact_size(vec2(larghezza, altezza), Sense::click());
    let contenuto = fondo_strumento(ui, t, r, risposta.hovered(), false);
    disegna_campione(ui.painter(), posto_icona(r, etichetta.is_some()).center(), 6.5, colore, t, false, false);
    if let Some(etichetta) = etichetta {
        scritta_strumento(ui, r, etichetta, contenuto);
    }
    let risposta = risposta.on_hover_text(tr.t("colore_nota"));
    menu_colori(&risposta, t, colore, tr)
}

/// La riga di colori che si apre cliccando su `risposta`. `true` se il colore
/// e' cambiato.
pub fn menu_colori(risposta: &Response, t: &Tavolozza, colore: &mut String, tr: &crate::lingue::Traduttore) -> bool {
    // Un pulsante in basso (sopra la tastiera del telefono) apre i colori
    // verso l'alto: sotto ci sarebbe la tastiera a coprirli.
    let in_basso = risposta.rect.center().y > risposta.ctx.content_rect().center().y;
    let verso = if in_basso { egui::RectAlign::TOP_START } else { egui::RectAlign::BOTTOM_START };
    let menu = egui::Popup::menu(risposta).align(verso).show(|ui| riga_colori(ui, t, colore, tr));
    let Some(menu) = menu else { return false };
    // Sul telefono, toccare un colore non deve chiudere la tastiera.
    super::schermo::proteggi_fuoco(&risposta.ctx, menu.response.rect);
    menu.inner
}

/// I pallini dei colori uno accanto all'altro. `true` se il colore e' cambiato.
pub fn riga_colori(ui: &mut Ui, t: &Tavolozza, colore: &mut String, tr: &crate::lingue::Traduttore) -> bool {
    let mut cambiato = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for nome in std::iter::once("").chain(crate::modello::nota::COLORI) {
            let (r, campione) = ui.allocate_exact_size(Vec2::splat(misura(26.0, 40.0)), Sense::click());
            let scelto = colore.as_str() == nome;
            if campione.hovered() {
                ui.painter().rect_filled(r, CornerRadius::same(RAGGIO - 2), t.superficie_attiva);
            }
            disegna_campione(ui.painter(), r.center(), 8.0, nome, t, scelto, true);
            let etichetta = if nome.is_empty() { "colore_nessuno".to_owned() } else { format!("colore_{nome}") };
            if campione.on_hover_text(tr.t(&etichetta)).clicked() && !scelto {
                *colore = nome.to_owned();
                cambiato = true;
            }
        }
    });
    cambiato
}

/// Pulsante fluttuante ("+", ☐): come gli altri pulsanti, ma con il fondo
/// pieno della nota (sta sopra il testo); l'icona nel colore principale.
pub fn disegna_pulsante_fluttuante(pittore: &egui::Painter, r: Rect, t: &Tavolozza, premuto: bool, icona: Icona) {
    // Il fondo della nota: sul computer il riquadro, sul telefono lo schermo.
    let nota = if super::schermo::telefono(pittore.ctx()) { t.sfondo } else { t.pannello };
    let fondo = if premuto { t.superficie_attiva } else { nota };
    pittore.rect(r, CornerRadius::same(RAGGIO), fondo, t.tratto(), StrokeKind::Inside);
    disegna_icona(pittore, r.shrink(r.width() * 0.3), icona, t.accento);
}

/// La lineetta che dice "si trascina" (telefono): la maniglia della
/// panoramica e la barra sopra la tastiera. Nel colore principale mentre il
/// dito la tiene.
pub fn disegna_lineetta(pittore: &egui::Painter, centro: Pos2, t: &Tavolozza, premuta: bool) {
    let r = Rect::from_center_size(centro, vec2(44.0, 5.0));
    pittore.rect_filled(r, CornerRadius::same(3), if premuta { t.accento } else { t.attenuato });
}

/// Il pallino del colore della nota, da solo (telefono, riga della data).
pub fn disegna_pallino_colore(pittore: &egui::Painter, c: Pos2, raggio: f32, colore: &str, t: &Tavolozza) {
    disegna_campione(pittore, c, raggio, colore, t, false, false);
}

/// Pallino del colore di una nota. "Nessun colore" e' un cerchio vuoto; nella
/// scelta dei colori (`barrato`) e' barrato, per dire "togli il colore".
fn disegna_campione(pittore: &egui::Painter, c: Pos2, raggio: f32, colore: &str, t: &Tavolozza, scelto: bool, barrato: bool) {
    match super::tema::tinta_nota(colore, t.scuro) {
        Some(tinta) => {
            pittore.circle_filled(c, raggio, tinta.segno);
        }
        None => {
            let tratto = Stroke::new(1.4, t.attenuato);
            pittore.circle_stroke(c, raggio - 0.7, tratto);
            if barrato {
                let d = (raggio - 0.7) * std::f32::consts::FRAC_1_SQRT_2;
                pittore.line_segment([pos2(c.x - d, c.y + d), pos2(c.x + d, c.y - d)], tratto);
            }
        }
    }
    if scelto {
        pittore.circle_stroke(c, raggio + 3.0, Stroke::new(TRATTO_ATTIVO, t.accento));
    }
}

/// Freccia (aperta o chiusa) per le sezioni retraibili.
fn disegna_freccia(pittore: &egui::Painter, c: Pos2, aperta: bool, colore: Color32) {
    let s = 4.5;
    let punti = if aperta {
        vec![pos2(c.x - s, c.y - s * 0.5), pos2(c.x + s, c.y - s * 0.5), pos2(c.x, c.y + s * 0.7)]
    } else {
        vec![pos2(c.x - s * 0.5, c.y - s), pos2(c.x + s * 0.7, c.y), pos2(c.x - s * 0.5, c.y + s)]
    };
    pittore.add(Shape::convex_polygon(punti, colore, Stroke::NONE));
}

/// Titolo cliccabile di una sezione retraibile: restituisce `true` se cliccato.
pub fn titolo_sezione(ui: &mut Ui, t: &Tavolozza, titolo: &str, aperta: bool) -> bool {
    let larghezza = ui.available_width();
    let (r, risposta) = ui.allocate_exact_size(vec2(larghezza, misura(32.0, 44.0)), Sense::click());
    let sopra = risposta.hovered();
    let pittore = ui.painter();
    if super::schermo::telefono(ui.ctx()) {
        // Telefono: righe semplici divise da una linea sottile, come un elenco.
        if sopra {
            pittore.rect_filled(r, CornerRadius::same(RAGGIO), t.superficie_attiva);
        }
        pittore.line_segment([r.left_bottom(), r.right_bottom()], t.tratto());
    } else {
        disegna_fondo_pulsante(pittore, r, t, sopra);
    }
    disegna_freccia(pittore, pos2(r.left() + 16.0, r.center().y), aperta, t.attenuato);
    pittore.text(
        pos2(r.left() + 30.0, r.center().y),
        Align2::LEFT_CENTER,
        titolo,
        FontId::proportional(14.0),
        t.testo,
    );
    risposta.clicked()
}

/// Il logo: lo stesso disegno dell'icona del programma (vedi `icona.rs`),
/// con i suoi colori fissi in entrambi i temi.
pub fn logo(ui: &mut Ui, lato: f32) -> Response {
    let (r, risposta) = ui.allocate_exact_size(Vec2::splat(lato), Sense::hover());
    disegna_logo(ui.painter(), r);
    risposta
}

pub fn disegna_logo(pittore: &egui::Painter, r: Rect) {
    use super::icona::{FONDO, FORME, GRIGLIA, colore_egui};
    let scala = r.width() / GRIGLIA;
    let punto = |(x, y): (f32, f32)| r.min + vec2(x, y) * scala;
    for forma in std::iter::once(&FONDO).chain(FORME.iter()) {
        let rettangolo = Rect::from_min_max(punto(forma.da), punto(forma.a));
        let angolo = CornerRadius::same((forma.raggio * scala).round() as u8);
        pittore.rect_filled(rettangolo, angolo, colore_egui(forma.colore));
    }
}

/// Riquadro con contorno, usato per intestazione, editor e barra laterale.
pub fn riquadro(t: &Tavolozza) -> Frame {
    Frame::new()
        .fill(t.pannello)
        .stroke(t.tratto())
        .corner_radius(RAGGIO_CONTENITORI)
        .inner_margin(Margin::same(10))
}

/// Testo secondario, in colore attenuato e a capo automatico.
pub fn nota_attenuata(ui: &mut Ui, t: &Tavolozza, testo: &str) {
    ui.add(egui::Label::new(RichText::new(testo).color(t.attenuato).size(TESTO_SECONDARIO)).wrap());
}

/// Un errore (rete, salvataggio, scorciatoia...): come il testo secondario,
/// in rosso. Lo stesso in menu, pagine, dialoghi e widget.
pub fn errore(ui: &mut Ui, t: &Tavolozza, testo: &str) {
    ui.add(egui::Label::new(RichText::new(testo).color(t.pericolo).size(TESTO_SECONDARIO)).wrap());
}

/// Etichetta sopra un campo o un gruppo di scelte ("Tema", "Lingua", "Nome
/// di questo dispositivo").
pub fn etichetta(ui: &mut Ui, testo: &str) {
    ui.label(RichText::new(testo).size(TESTO_SECONDARIO));
}

/// Titolo di un dialogo, nel colore indicato (rosso per le conferme che
/// eliminano).
pub fn titolo_dialogo(ui: &mut Ui, testo: &str, colore: Color32) {
    ui.label(RichText::new(testo).strong().size(TITOLO_DIALOGO).color(colore));
}

/// Poche scelte su una riga, tutte larghe uguali (Tema: Sistema · Chiaro ·
/// Scuro; Lingua): ognuna e' una `scelta`. Restituisce quella toccata.
pub fn scelte(ui: &mut Ui, t: &Tavolozza, voci: &[(&str, &str, bool)]) -> Option<usize> {
    let mut toccata = None;
    let n = voci.len().max(1) as f32;
    let spazio = ui.spacing().item_spacing.x;
    let larghezza = ((ui.available_width() - spazio * (n - 1.0)) / n).max(40.0);
    ui.horizontal(|ui| {
        for (i, (testo, aiuto, scelto)) in voci.iter().enumerate() {
            let scritta =
                if *scelto { RichText::new(*testo).color(t.accento).strong() } else { RichText::new(*testo).color(t.testo) };
            let mut pulsante = Button::new(scritta).corner_radius(RAGGIO).min_size(vec2(larghezza, ALTEZZA_PULSANTE));
            if *scelto {
                pulsante = pulsante.stroke(Stroke::new(TRATTO_ATTIVO, t.accento));
            }
            if ui.add_sized(vec2(larghezza, ALTEZZA_PULSANTE), pulsante).on_hover_text(*aiuto).clicked() {
                toccata = Some(i);
            }
        }
    });
    toccata
}
