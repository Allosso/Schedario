//! L'area di scrittura della nota selezionata (o di quella del cestino).

use egui::{Align, Frame, Id, Layout, RichText, ScrollArea, TextEdit, Ui, UiBuilder, vec2};

use crate::applicazione::{Applicazione, Avviso};
use crate::interfaccia::componenti::{
    self, Icona, pulsante, pulsante_colorato,
    pulsante_strumento,
};
use crate::interfaccia::schermo::{self, misura};
use crate::interfaccia::tema::{self, TESTO_SECONDARIO, Tavolozza};
use crate::interfaccia::testo_nota;
use crate::piattaforma::ora_locale;

/// Altezza comune di Apri widget, barra degli strumenti e Cancella.
const ALTEZZA_PULSANTI: f32 = componenti::ALTEZZA_PULSANTE;
/// Altezza delle due righe sopra il testo (titolo e data), e spazio tra loro.
const ALTEZZA_RIGA: f32 = misura(32.0, 44.0);
const SPAZIO_RIGHE: f32 = 4.0;
/// Spazio minimo tra la fine della barra degli strumenti e Cancella.
const DISTANZA_DA_CANCELLA: f32 = 12.0;

pub fn mostra(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let area = ui.available_rect_before_wrap();
    // Dove comincia il riquadro della nota: la linguetta aperta scende fin qui.
    ui.ctx().data_mut(|d| d.insert_temp(Id::unique("inizio_editor"), area.top()));
    // Telefono: trascinando il dito di lato la nota lo segue e dal bordo entra
    // la nota accanto (vedi `Programma::smista_gesti`); sollevando il dito lo
    // scorrimento si completa da solo.
    let mut spostata = area;
    if schermo::telefono(ui.ctx()) && !app.interfaccia.scrittura_compatta {
        schermo::zona_sfoglia(ui.ctx(), Some(area));
        let dx = spostamento_sfoglia(ui.ctx(), app, area);
        if dx.abs() > 0.5 {
            spostata = area.translate(vec2(dx, 0.0));
            accanto(ui, app, t, area, dx);
        }
    }
    let ritaglio = ui.clip_rect().intersect(area);
    ui.scope_builder(UiBuilder::new().max_rect(spostata), |ui| {
        ui.set_clip_rect(ritaglio);
        // Sul telefono niente riquadro attorno alla nota: il testo usa tutta
        // la larghezza dello schermo.
        let cornice = if schermo::telefono(ui.ctx()) {
            cornice_telefono()
        } else {
            // In alto a sinistra l'angolo e' dritto: li' si appoggia la prima
            // linguetta, che altrimenti lascerebbe una tacca.
            let r = tema::RAGGIO_CONTENITORI;
            componenti::riquadro(t).inner_margin(14).corner_radius(egui::CornerRadius { nw: 0, ne: r, sw: r, se: r })
        };
        cornice.show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            contenuto(ui, app, t);
        });
    });
}

/// Pulsanti fluttuanti del telefono (☐ a destra, "+" a sinistra, la
/// maniglia al centro): lato e distanza dai bordi.
pub const LATO_PULSANTE: f32 = misura(36.0, 46.0);
pub const MARGINE_PULSANTE: f32 = 14.0;
/// Mentre si scrive ☐ sta sopra la barra della tastiera, con questo spazio
/// sotto: cosi' non si tocca uno per l'altro.
const SOPRA_LA_TASTIERA: f32 = 44.0;

/// Telefono: il pulsante tondo ☐ nell'angolo in basso a destra della nota,
/// sopra il testo. Toccarlo non toglie il fuoco al testo (resta la tastiera).
/// `true` se e' stato toccato.
fn pulsante_lista(ctx: &egui::Context, t: &Tavolozza, tr: &crate::lingue::Traduttore, area: egui::Rect) -> bool {
    let margine = margine_pulsante(ctx);
    let sotto = if super::barra_tastiera::tastiera_aperta() { SOPRA_LA_TASTIERA } else { margine };
    let posto = egui::Rect::from_min_size(
        area.right_bottom() - vec2(LATO_PULSANTE + margine, LATO_PULSANTE + sotto),
        egui::Vec2::splat(LATO_PULSANTE),
    );
    schermo::proteggi_fuoco(ctx, posto);
    egui::Area::new(Id::unique("pulsante_lista"))
        .order(egui::Order::Middle)
        .fixed_pos(posto.min)
        .show(ctx, |ui| {
            let (r, risposta) = ui.allocate_exact_size(posto.size(), egui::Sense::click());
            // Stesso stile degli altri pulsanti: angoli poco arrotondati, bordo sottile.
            componenti::disegna_pulsante_fluttuante(ui.painter(), r, t, risposta.is_pointer_button_down_on(), Icona::Lista);
            risposta.on_hover_text(tr.t("aiuto_pulsante_lista")).clicked()
        })
        .inner
}

/// Distanza di ☐ (e dei contatori) dal bordo dell'area del testo. Sul
/// computer l'area e' gia' rientrata nel riquadro della nota: ☐ sta dove
/// finisce il testo, con lo stesso margine dal bordo.
fn margine_pulsante(ctx: &egui::Context) -> f32 {
    if schermo::telefono(ctx) { MARGINE_PULSANTE } else { 0.0 }
}

/// Quanto dura (secondi) la fine dello scorrimento tra due note.
const DURATA_SFOGLIA: f32 = 0.16;

/// Di quanto e' spostata la nota: segue il dito mentre sfoglia; sollevato il
/// dito, arriva da sola al bordo (si passa alla nota accanto) o torna al suo
/// posto. Quando la nota accanto e' arrivata, diventa lei la nota aperta.
fn spostamento_sfoglia(ctx: &egui::Context, app: &mut Applicazione, area: egui::Rect) -> f32 {
    let id = Id::unique("sfoglia_animata");
    let dito = schermo::sfoglia(ctx);
    let fine = schermo::conclusione_sfoglia(ctx);
    let (obiettivo, tempo) = match (dito, fine) {
        (Some(dx), _) => (dx, 0.0),
        (None, Some((arrivo, _))) => (arrivo, DURATA_SFOGLIA),
        _ => (0.0, 0.0),
    };
    let dx = ctx.animate_value_with_time(id, obiettivo, tempo);
    if let (None, Some((arrivo, verso))) = (dito, fine) {
        if (dx - arrivo).abs() < 0.5 || area.width() < 1.0 {
            schermo::imposta_conclusione_sfoglia(ctx, None);
            if verso != 0 {
                if let Some(aperta) = app.interfaccia.selezionata.clone() {
                    if let Some(vicina) = app.nota_vicina(&aperta, verso as isize) {
                        app.conferma_rinomina();
                        app.interfaccia.selezionata = Some(vicina);
                    } else if verso == 1 && app.pagina_bianca_dopo(&aperta) {
                        // Oltre l'ultima nota c'e' una pagina bianca: una nota nuova.
                        app.conferma_rinomina();
                        app.nuova_nota();
                    }
                }
            }
            ctx.animate_value_with_time(id, 0.0, 0.0);
            return 0.0;
        }
    }
    dx
}

/// Sul telefono niente riquadro attorno alla nota: il testo usa tutta la
/// larghezza dello schermo.
fn cornice_telefono() -> Frame {
    // Con gli 8 dell'area di lavoro il testo resta a 12 dal bordo; schede,
    // panoramica e pagine stanno a 8 come le altre.
    Frame::NONE.inner_margin(egui::Margin { left: 4, right: 4, top: 0, bottom: 0 })
}

/// La nota accanto che entra dal bordo mentre il dito sposta quella aperta:
/// disegnata come la nota aperta (intestazione, caselle, voci fatte,
/// scorrimento), dove sara' quando sara' lei la nota aperta, cosi' alla fine
/// non c'e' nessuno scatto. Mentre il dito sfoglia egui non riceve tocchi
/// (`Programma::smista_gesti`): la nota accanto si vede ma non si modifica.
/// Oltre l'ultima nota c'e' una pagina bianca: "+ Nuova nota".
fn accanto(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza, area: egui::Rect, dx: f32) {
    let Some(aperta) = app.interfaccia.selezionata.clone() else { return };
    let verso = if dx < 0.0 { 1 } else { -1 };
    let vicina = app.nota_vicina(&aperta, verso);
    if vicina.is_none() && !(verso == 1 && app.pagina_bianca_dopo(&aperta)) {
        return;
    }
    // Dove sta la nota accanto: subito a destra (o a sinistra) di quella aperta.
    let posto = area.translate(vec2(if dx < 0.0 { area.width() + dx } else { dx - area.width() }, 0.0));
    let ritaglio = ui.clip_rect().intersect(area);
    let pittore = ui.painter_at(area);
    pittore.rect_filled(posto, 0.0, t.sfondo);
    pittore.line_segment([posto.left_top(), posto.left_bottom()], t.tratto());
    pittore.line_segment([posto.right_top(), posto.right_bottom()], t.tratto());
    let Some(vicina) = vicina else {
        let dentro = posto.shrink2(vec2(12.0, 0.0));
        let titolo = format!("+ {}", app.tr.t("barra_nuova_nota"));
        let galleria = ui.fonts_mut(|f| {
            let mut lavoro = egui::text::LayoutJob::simple_singleline(titolo, tema::carattere_titolo(), t.accento);
            lavoro.wrap = egui::text::TextWrapping::truncate_at_width(dentro.width());
            f.layout_job(lavoro)
        });
        let alto = galleria.size().y;
        pittore.galley(egui::pos2(dentro.left(), dentro.top() + (ALTEZZA_PULSANTI - alto) / 2.0), galleria, t.accento);
        return;
    };
    // Un figlio che non occupa posto: la nota aperta si dispone come sempre.
    let mut figlio = ui.new_child(UiBuilder::new().max_rect(posto).id_salt("nota_accanto").layout(*ui.layout()));
    figlio.set_clip_rect(ritaglio);
    altra_nota(&mut figlio, app, t, vicina);
}

/// Telefono: disegna la nota `id` come se fosse quella aperta (intestazione,
/// caselle, scorrimento), senza che lo diventi: quello che il disegno consuma
/// o ricorda (fuoco del titolo, parola trovata) resta della nota aperta
/// davvero. Per la nota accanto mentre si sfoglia e per la nota del cestino
/// che scende.
pub fn altra_nota(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza, id: String) {
    let aperta = app.interfaccia.selezionata.clone();
    let fuoco = std::mem::take(&mut app.interfaccia.fuoco_titolo);
    let ricerca = (
        app.interfaccia.ricerca_attuale_di.clone(),
        app.interfaccia.ricerca_attuale,
        app.interfaccia.ricerca_mostra_attuale,
    );
    app.interfaccia.selezionata = Some(id);
    cornice_telefono().show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        contenuto(ui, app, t);
    });
    app.interfaccia.selezionata = aperta;
    app.interfaccia.fuoco_titolo = fuoco;
    (app.interfaccia.ricerca_attuale_di, app.interfaccia.ricerca_attuale, app.interfaccia.ricerca_mostra_attuale) = ricerca;
}
/// Computer: "🕘 Modificata oggi alle 18:24", senza bordo; sotto il mouse il
/// fondo si schiarisce come un pulsante. Un clic apre la Cronologia.
fn data_cronologia(ui: &mut Ui, t: &Tavolozza, data: &str, aiuto: &str, altezza: f32) -> egui::Response {
    let carattere = egui::FontId::proportional(12.5);
    let galleria = ui.painter().layout_no_wrap(data.to_owned(), carattere, t.attenuato);
    let icona = 14.0;
    let larghezza = (8.0 + icona + 6.0 + galleria.size().x + 10.0).min(ui.available_width().max(0.0));
    let (r, risposta) = ui.allocate_exact_size(vec2(larghezza, altezza), egui::Sense::click());
    let sopra = risposta.hovered() || risposta.is_pointer_button_down_on();
    if sopra {
        ui.painter().rect_filled(r, egui::CornerRadius::same(tema::RAGGIO), t.superficie_attiva);
    }
    let colore = if sopra { t.testo } else { t.attenuato };
    let posto = egui::Rect::from_min_size(egui::pos2(r.left() + 8.0, r.center().y - icona / 2.0), egui::Vec2::splat(icona));
    componenti::disegna_icona(ui.painter(), posto, Icona::Orologio, colore);
    let inizio = egui::pos2(posto.right() + 6.0, r.center().y - galleria.size().y / 2.0);
    ui.painter().with_clip_rect(r).galley_with_override_text_color(inizio, galleria, colore);
    risposta.on_hover_text(aiuto)
}

/// Cosa e' stato scelto nell'intestazione della nota sul telefono.
#[derive(Default)]
struct AzioniTelefono {
    cambiato: bool,
    fissa: bool,
    cronologia: bool,
    cancella: bool,
}

/// Telefono: il titolo (toccandolo lo si rinomina) e sotto una riga in cui
/// ogni pezzo si tocca:
///
/// ```text
/// ● · 📌 · oggi 21:15 · 10 parole              🗑
/// ```
///
/// il pallino e' il colore della nota (lo cambia), la puntina e' piena se la
/// nota e' fissata (la fissa o la toglie), la data apre la Cronologia, il
/// Cestino in fondo, rosso, la sposta nel cestino. Mentre si scrive resta
/// solo il titolo.
#[allow(clippy::too_many_arguments)]
fn intestazione_telefono(
    ui: &mut Ui,
    app: &mut Applicazione,
    t: &Tavolozza,
    id: &str,
    id_titolo: Id,
    titolo: &mut String,
    colore: &mut String,
    fissata: bool,
) -> AzioniTelefono {
    let mut azioni = AzioniTelefono::default();
    let scrivendo = super::barra_tastiera::tastiera_aperta();
    let suggerimento = app.tr.t("titolo_suggerimento").to_owned();
    let larghezza = ui.available_width();
    let fuoco = std::mem::take(&mut app.interfaccia.fuoco_titolo);
    let prima = testo_nota::Prima::leggi(ui, id_titolo);
    let uscita = TextEdit::singleline(titolo)
        .id(id_titolo)
        .font(tema::carattere_titolo())
        .hint_text(RichText::new(suggerimento).font(tema::carattere_titolo()).color(t.attenuato))
        .frame(Frame::NONE)
        .desired_width(larghezza)
        .min_size(vec2(0.0, ALTEZZA_PULSANTI))
        .align(egui::Align2::LEFT_CENTER)
        .show(ui);
    if fuoco {
        uscita.response.request_focus();
    }
    crate::interfaccia::tastiera::campo(ui, &uscita, titolo, false, false);
    azioni.cambiato |= uscita.response.changed();
    if uscita.response.has_focus() {
        let r = uscita.response.rect;
        let y = r.bottom() - 1.0;
        ui.painter().line_segment([egui::pos2(r.left(), y), egui::pos2(r.right(), y)], egui::Stroke::new(1.5, t.accento));
    }
    azioni.cambiato |= testo_nota::menu_campo(ui, &app.tr, id_titolo, &uscita, &prima, titolo);
    if scrivendo {
        ui.add_space(SPAZIO_RIGHE);
        return azioni;
    }
    let Some(nota) = app.nota(id) else { return azioni };
    let q = ora_locale::quando(nota.modificata, app.tr.codice());
    let chiave = match q.giorni_fa {
        0 => "data_breve_oggi",
        1 => "data_breve_ieri",
        _ => "data_breve_il",
    };
    let quando = app.tr.f(chiave, &[("data", &q.data), ("ora", &q.ora)]);
    let trascina = app.interfaccia.trascinava_nota;
    let puntatore = ui.input(|i| i.pointer.latest_pos());
    // Colore e Fissa: due pulsanti quadrati grandi come un dito, ben distanziati.
    let lato = componenti::LATO_ICONA;
    let riquadro = |ui: &Ui, r: egui::Rect, premuto: bool| componenti::disegna_fondo_pulsante(ui.painter(), r, t, premuto);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        // Il colore della nota.
        let (r, pallino) = ui.allocate_exact_size(vec2(lato, lato), egui::Sense::click());
        riquadro(ui, r, pallino.is_pointer_button_down_on());
        componenti::disegna_pallino_colore(ui.painter(), r.center(), 8.0, colore.as_str(), t);
        let pallino = pallino.on_hover_text(app.tr.t("colore_nota"));
        if componenti::menu_colori(&pallino, t, colore, &app.tr) {
            azioni.cambiato = true;
        }
        // Fissata o no.
        let (r, puntina) = ui.allocate_exact_size(vec2(lato, lato), egui::Sense::click());
        riquadro(ui, r, puntina.is_pointer_button_down_on());
        let (icona, tinta, aiuto) = if fissata {
            (Icona::PuntinaAttiva, t.accento, app.tr.t("aiuto_togli_fissata"))
        } else {
            (Icona::Puntina, t.attenuato, app.tr.t("aiuto_fissa"))
        };
        componenti::disegna_icona(ui.painter(), egui::Rect::from_center_size(r.center(), egui::Vec2::splat(15.0)), icona, tinta);
        azioni.fissa = puntina.on_hover_text(aiuto).clicked();
        // Quando e' stata modificata: toccandola, le versioni precedenti.
        // Solo la data: righe, parole e spunte stanno nelle schede della panoramica.
        let data = egui::Label::new(RichText::new(quando).color(t.attenuato).size(TESTO_SECONDARIO))
            .selectable(false)
            .truncate()
            .sense(egui::Sense::click());
        let spazio = ui.available_width() - lato - 4.0;
        azioni.cronologia = ui
            .allocate_ui(vec2(spazio.max(0.0), lato), |ui| ui.add(data))
            .inner
            .on_hover_text(app.tr.t("aiuto_data_cronologia"))
            .clicked();
        // Il Cestino, in fondo a destra: lontano dal pollice e sempre in vista.
        // Mentre si trascina una scheda e' il posto dove lasciarla.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (r, cestino) = ui.allocate_exact_size(vec2(lato, lato), egui::Sense::click());
            schermo::imposta_zona_cestino(ui.ctx(), Some(r.expand(10.0)));
            // Un quadrato come il colore e la puntina; mentre si trascina una
            // scheda diventa rosso: e' il posto dove lasciarla.
            if trascina {
                let sopra = puntatore.is_some_and(|p| r.expand(10.0).contains(p));
                let fondo = if sopra { t.pericolo.gamma_multiply(tema::PERICOLO_SOPRA) } else { t.pericolo.gamma_multiply(tema::PERICOLO_LEGGERO) };
                ui.painter().rect(r, egui::CornerRadius::same(tema::RAGGIO), fondo, egui::Stroke::new(1.5, t.pericolo), egui::StrokeKind::Inside);
            } else {
                riquadro(ui, r, cestino.is_pointer_button_down_on());
            }
            componenti::disegna_icona(ui.painter(), egui::Rect::from_center_size(r.center(), egui::Vec2::splat(18.0)), Icona::Cestino, t.pericolo);
            azioni.cancella = cestino.on_hover_text(app.tr.t("aiuto_cestina_nota")).clicked() && !trascina;
        });
    });
    ui.add_space(SPAZIO_RIGHE);
    azioni
}
fn contenuto(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let Some(id) = app.interfaccia.selezionata.clone().filter(|id| app.nota(id).is_some()) else {
        ui.centered_and_justified(|ui| {
            ui.add(egui::Label::new(RichText::new(app.tr.t("scegli_nota")).color(t.attenuato).size(tema::TESTO_VUOTO)).selectable(false));
        });
        return;
    };
    let nel_cestino = app.nota(&id).is_some_and(|n| n.nel_cestino);
    let inizio = ui.cursor().min;
    // "Inserisci un titolo": un invito chiaro. Che la scheda usi intanto la
    // prima riga del testo lo dice la descrizione al passaggio del mouse.
    let suggerimento_titolo = app.tr.t("titolo_suggerimento").to_owned();
    let suggerimento_testo = app.tr.t("testo_suggerimento").to_owned();

    let mut cambiato = false;
    let mut azione_cestino = false;
    let mut elimina = false;
    let mut alterna_fissata = false;
    let mut apri_cronologia = false;
    let mut apri_widget = false;
    // Si lavora su copie: il campo e i menu hanno bisogno anche dei testi tradotti.
    let (mut titolo, mut testo, mut colore_nota, fissata) = app
        .nota(&id)
        .map(|n| (n.titolo.clone(), n.testo.clone(), n.colore.clone(), n.fissata))
        .unwrap_or_default();
    let id_titolo = Id::unique(("titolo", id.as_str()));
    let id_testo = Id::unique(("testo", id.as_str()));

    // Telefono con la tastiera aperta e poco spazio (in orizzontale): solo
    // il testo, senza titolo e strumenti; tornano chiudendo la tastiera.
    let compatta = app.interfaccia.scrittura_compatta;
    let telefono = schermo::telefono(ui.ctx());
    if telefono && !nel_cestino && !compatta {
        let azioni = intestazione_telefono(ui, app, t, &id, id_titolo, &mut titolo, &mut colore_nota, fissata);
        cambiato |= azioni.cambiato;
        alterna_fissata = azioni.fissa;
        apri_cronologia = azioni.cronologia;
        azione_cestino = azioni.cancella;
    } else if !compatta {
    // Due righe, come sul telefono:
    //   titolo ...........................  [ ⧉ Apri widget ] [ Cancella ]
    //   [ ● Colore ] [ 📌 Fissa ]  🕘 Modificata oggi alle 18:24
    // A destra le azioni su tutta la nota; la data apre la Cronologia. Nel
    // cestino: Ripristina ed Elimina definitivamente, e solo la data.
    let larghezza = ui.available_width();
    let testo_widget = app.tr.t("apri_widget").to_owned();
    // Su Android non ci sono widget sul desktop: niente Apri widget.
    let con_widget = !cfg!(target_os = "android");
    let larghezza_widget = componenti::larghezza_strumento(ui, Some(&testo_widget), ALTEZZA_PULSANTI);
    // Telefono, nota del cestino: lo schermo e' stretto, quindi il titolo ha
    // la sua riga e Ripristina / Elimina definitivamente stanno nella riga sotto.
    let pulsanti_sotto = telefono && nel_cestino;
    ui.allocate_ui_with_layout(vec2(larghezza, ALTEZZA_RIGA), Layout::left_to_right(Align::Center), |ui| {
        let larghezza_titolo = if pulsanti_sotto {
            larghezza
        } else if nel_cestino {
            larghezza - 320.0
        } else {
            // Il titolo arriva fino ai pulsanti a destra.
            let scritta = egui::WidgetText::from(RichText::new(app.tr.t("cancella")).strong())
                .into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, egui::TextStyle::Button)
                .size()
                .x;
            let cancella = scritta + 2.0 * ui.spacing().button_padding.x;
            let widget = if con_widget { larghezza_widget + ui.spacing().item_spacing.x } else { 0.0 };
            larghezza - cancella - widget - DISTANZA_DA_CANCELLA
        }
        .max(80.0);
        let fuoco = std::mem::take(&mut app.interfaccia.fuoco_titolo);
        let prima = testo_nota::Prima::leggi(ui, id_titolo);
        let uscita = ui
            .add_enabled_ui(!nel_cestino, |ui| {
                TextEdit::singleline(&mut titolo)
                    .id(id_titolo)
                    .font(tema::carattere_titolo())
                    .hint_text(RichText::new(suggerimento_titolo).font(tema::carattere_titolo()).color(t.attenuato))
                    .frame(Frame::NONE)
                    .desired_width(larghezza_titolo)
                    // Alto come i pulsanti, con il testo al centro: sulla stessa linea.
                    .min_size(vec2(0.0, ALTEZZA_PULSANTI))
                    .align(egui::Align2::LEFT_CENTER)
                    .show(ui)
            })
            .inner;
        if fuoco {
            uscita.response.request_focus();
        }
        crate::interfaccia::tastiera::campo(ui, &uscita, &titolo, false, false);
        cambiato |= uscita.response.changed();
        // Solo finche' il campo non e' in uso: mentre si scrive darebbe fastidio.
        if !nel_cestino && !uscita.response.has_focus() {
            uscita.response.response.clone().on_hover_text(app.tr.t("aiuto_titolo_nota"));
        }
        // Una linea sottile sotto il titolo dice che si puo' scrivere: grigia
        // al passaggio del mouse, nel colore principale mentre si scrive.
        if !nel_cestino && (uscita.response.hovered() || uscita.response.has_focus()) {
            let colore = if uscita.response.has_focus() { t.accento } else { t.bordo };
            let r = uscita.response.rect;
            let y = r.bottom() - 1.0;
            ui.painter().line_segment([egui::pos2(r.left(), y), egui::pos2(r.right(), y)], egui::Stroke::new(1.5, colore));
        }
        if !nel_cestino {
            cambiato |= testo_nota::menu_campo(ui, &app.tr, id_titolo, &uscita, &prima, &mut titolo);
        }
        if pulsanti_sotto {
            return;
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if nel_cestino {
                // Da destra a sinistra: Elimina definitivamente a destra di Ripristina.
                elimina = pulsante_colorato(ui, t, app.tr.t("elimina_definitivamente"), t.pericolo)
                    .on_hover_text(app.tr.t("aiuto_elimina_definitivamente"))
                    .clicked();
                azione_cestino = pulsante_colorato(ui, t, app.tr.t("ripristina"), t.successo)
                    .on_hover_text(app.tr.t("aiuto_ripristina"))
                    .clicked();
            } else {
                // Scritta rossa: si riconosce subito che toglie la nota.
                azione_cestino = pulsante_colorato(ui, t, app.tr.t("cancella"), t.pericolo)
                    .on_hover_text(app.tr.t("aiuto_cancella"))
                    .clicked();
                // Subito a sinistra di Cancella: anche lui riguarda tutta la nota.
                if con_widget {
                    let descrizione = app.tr.t("aiuto_apri_widget");
                    apri_widget = pulsante_strumento(ui, t, Icona::Widget, Some(&testo_widget), descrizione, ALTEZZA_PULSANTI, true)
                        .clicked();
                }
            }
        });
    });
    if pulsanti_sotto {
        ui.add_space(SPAZIO_RIGHE);
        ui.allocate_ui_with_layout(vec2(larghezza, ALTEZZA_RIGA), Layout::left_to_right(Align::Center), |ui| {
            azione_cestino = pulsante_colorato(ui, t, app.tr.t("ripristina"), t.successo)
                .on_hover_text(app.tr.t("aiuto_ripristina"))
                .clicked();
            elimina = pulsante_colorato(ui, t, app.tr.t("elimina_definitivamente"), t.pericolo)
                .on_hover_text(app.tr.t("aiuto_elimina_definitivamente"))
                .clicked();
        });
    }

    ui.add_space(SPAZIO_RIGHE);
    // Quando e' stata modificata (al passaggio del mouse, creata).
    let date = app.nota(&id).map(|n| (n.modificata, n.creata)).map(|(modificata, creata)| {
        let lingua = app.tr.codice();
        let quando = |ms: i64, prefisso: &str| {
            let q = ora_locale::quando(ms, lingua);
            let chiave = match q.giorni_fa {
                0 => format!("{prefisso}_oggi"),
                1 => format!("{prefisso}_ieri"),
                _ => format!("{prefisso}_il"),
            };
            app.tr.f(&chiave, &[("data", &q.data), ("ora", &q.ora)])
        };
        (quando(modificata, "modificata"), quando(creata, "creata"))
    });
    ui.allocate_ui_with_layout(vec2(larghezza, ALTEZZA_RIGA), Layout::left_to_right(Align::Center), |ui| {
        let Some((modificata, creata)) = date else { return };
        if nel_cestino {
            let testo = RichText::new(modificata).color(t.attenuato).size(TESTO_SECONDARIO);
            ui.add(egui::Label::new(testo).selectable(false)).on_hover_text(creata);
            // Sul telefono i contatori stanno solo nelle schede (panoramica, Cestino).
            if !telefono {
                super::riga_stato::contatori(ui, app, t, &id);
            }
            return;
        }
        // Il colore della nota e Fissa, con il loro nome (qui lo spazio c'e').
        let scritta_colore = app.tr.t("strumento_colore").to_owned();
        if componenti::scelta_colore(ui, t, &mut colore_nota, &app.tr, Some(&scritta_colore), ALTEZZA_PULSANTI) {
            cambiato = true;
        }
        let (icona_fissa, scritta_fissa, aiuto_fissa) = if fissata {
            (Icona::PuntinaAttiva, app.tr.t("strumento_fissata"), app.tr.t("aiuto_togli_fissata"))
        } else {
            (Icona::Puntina, app.tr.t("strumento_fissa"), app.tr.t("aiuto_fissa"))
        };
        // Fissata: la puntina nel colore principale.
        alterna_fissata = pulsante_strumento(ui, t, icona_fissa, Some(scritta_fissa), aiuto_fissa, ALTEZZA_PULSANTI, fissata).clicked();
        // La data con l'orologio: un clic apre la Cronologia.
        let aiuto = format!("{}\n{}", app.tr.t("aiuto_cronologia"), creata);
        apri_cronologia = data_cronologia(ui, t, &modificata, &aiuto, ALTEZZA_PULSANTI).clicked();
        // A destra, sotto Apri widget e Cancella: righe, parole, spunte.
        super::riga_stato::contatori(ui, app, t, &id);
    });
    ui.add_space(SPAZIO_RIGHE);
    }
    if nel_cestino {
        componenti::nota_attenuata(ui, t, app.tr.t("nota_nel_cestino"));
    }
    let mut apri_originale = None;
    let mut conflitto_risolto = false;
    if let Some((originale, quando)) = app.nota(&id).and_then(|n| n.conflitto_di.clone().map(|o| (o, n.modificata))) {
        let titolo_originale =
            app.nota(&originale).map(|n| n.titolo_visibile(app.tr.t("senza_titolo")).to_owned());
        // Un contenitore come gli altri (angoli e margine del riquadro), con
        // il contorno rosso di "attenzione" come la scheda in conflitto.
        Frame::new()
            .fill(t.superficie)
            .stroke(egui::Stroke::new(tema::TRATTO_ATTIVO, t.pericolo))
            .corner_radius(tema::RAGGIO_CONTENITORI)
            .inner_margin(10)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let testo = app.tr.f(
                    "conflitto_avviso",
                    &[
                        ("titolo", titolo_originale.as_deref().unwrap_or("—")),
                        ("data", &ora_locale::data_ora(quando, app.tr.codice())),
                    ],
                );
                ui.add(egui::Label::new(RichText::new(testo).color(t.testo).size(tema::TESTO_VOCE)).wrap());
                ui.horizontal(|ui| {
                    if titolo_originale.is_some()
                        && pulsante(ui, t, app.tr.t("conflitto_apri")).on_hover_text(app.tr.t("aiuto_conflitto_apri")).clicked()
                    {
                        apri_originale = Some(originale.clone());
                    }
                    if pulsante(ui, t, app.tr.t("conflitto_risolto")).on_hover_text(app.tr.t("aiuto_conflitto_risolto")).clicked() {
                        conflitto_risolto = true;
                    }
                });
            });
    }
    ui.separator();

    // Ricerca: "Parola 2 di 5" con ‹ › per passare da una parola trovata
    // all'altra in questa nota. Cambiando ricerca o nota si riparte dalla prima.
    // (Per una nota del cestino vale la ricerca del Cestino.)
    let ricerca = app.ricerca_per(&id).to_owned();
    let trovate = crate::modello::nota::Nota::occorrenze(&testo, &ricerca).len();
    let chiave_ricerca = (ricerca.clone(), id.clone());
    if app.interfaccia.ricerca_attuale_di.as_ref() != Some(&chiave_ricerca) {
        app.interfaccia.ricerca_attuale_di = Some(chiave_ricerca);
        app.interfaccia.ricerca_attuale = 0;
        app.interfaccia.ricerca_mostra_attuale = trovate > 0;
    }
    if trovate > 0 && app.interfaccia.ricerca_attuale >= trovate {
        app.interfaccia.ricerca_attuale = trovate - 1;
    }
    if trovate > 0 {
        ui.horizontal(|ui| {
            let posizione = app.interfaccia.ricerca_attuale;
            let scritta =
                app.tr.f("ricerca_posizione", &[("i", &(posizione + 1).to_string()), ("n", &trovate.to_string())]);
            ui.label(RichText::new(scritta).color(t.attenuato).size(TESTO_SECONDARIO));
            let precedente = componenti::pulsante_icona(ui, t, Icona::Precedente, app.tr.t("aiuto_ricerca_precedente"), trovate > 1);
            let successiva = componenti::pulsante_icona(ui, t, Icona::Successivo, app.tr.t("aiuto_ricerca_successiva"), trovate > 1);
            // In giro: dopo l'ultima si torna alla prima.
            if precedente.clicked() {
                app.interfaccia.ricerca_attuale = (posizione + trovate - 1) % trovate;
                app.interfaccia.ricerca_mostra_attuale = true;
            }
            if successiva.clicked() {
                app.interfaccia.ricerca_attuale = (posizione + 1) % trovate;
                app.interfaccia.ricerca_mostra_attuale = true;
            }
        });
    }
    let evidenzia = testo_nota::Evidenzia {
        parole: &ricerca,
        attuale: (trovate > 0).then_some(app.interfaccia.ricerca_attuale),
        mostra_attuale: std::mem::take(&mut app.interfaccia.ricerca_mostra_attuale),
    };

    // Testo: la barra di scorrimento compare solo se serve.
    let carattere = tema::carattere_nota(app.preferenze.dimensione_testo);
    // Telefono: il pulsante tondo ☐ in basso a destra sulla nota.
    // (Non mentre il dito sfoglia le note: si sposterebbe con la nota.)
    // Anche sul computer: la voce di lista e' uno strumento per scrivere, sta sul testo.
    let con_pulsante_lista = !nel_cestino
        && !compatta
        && schermo::sfoglia(ui.ctx()).is_none()
        && schermo::conclusione_sfoglia(ui.ctx()).is_none();
    let area_testo = ui.max_rect();
    let scorrimento = schermo::scorrevole(ScrollArea::vertical())
        .id_salt(("scorrimento", id.as_str()))
        .auto_shrink(false)
        .show(ui, |ui| {
            // Lo spazio lasciato in fondo per il pulsante ☐ va tolto dall'altezza
            // del campo: altrimenti anche un testo corto e' piu' alto dell'area,
            // e trascinandolo sale e resta nascosto.
            // (Anche lo spazio che egui lascia tra il campo e lo spazio vuoto.)
            let spazio_sotto = if telefono { LATO_PULSANTE + SOPRA_LA_TASTIERA + 8.0 } else { LATO_PULSANTE + 8.0 };
            let riserva = if con_pulsante_lista {
                spazio_sotto + 2.0 * ui.spacing().item_spacing.y
            } else {
                0.0
            };
            let altezza = (ui.available_height() - riserva).max(0.0);
            cambiato |= testo_nota::testo(
                ui,
                t,
                &app.tr,
                &mut testo,
                id_testo,
                carattere,
                &suggerimento_testo,
                !nel_cestino,
                altezza,
                evidenzia,
            );
            // Le ultime righe non restano sotto il pulsante ☐.
            if con_pulsante_lista {
                ui.add_space(spazio_sotto);
            }
        });
    // Telefono, tastiera chiusa: se il testo non scorre piu' giu' (corto, o
    // letto fino in fondo) la panoramica si apre trascinando verso l'alto da
    // tutta la meta' bassa della nota, non solo dalla fascia della maniglia.
    let in_fondo = scorrimento.state.clamped_offset().y + scorrimento.inner_rect.height() >= scorrimento.content_size.y - 1.0;
    // Telefono, nota del cestino: trascinando in giu' la testata (o tutta la
    // nota, se il testo e' in cima) si torna alle note (`Programma::smista_gesti`).
    if telefono && nel_cestino && !compatta {
        let testata = egui::Rect::from_min_max(inizio, egui::pos2(ui.max_rect().right(), scorrimento.inner_rect.top()));
        let in_cima = scorrimento.state.clamped_offset().y <= 1.0;
        let zona = if in_cima { testata.union(scorrimento.inner_rect) } else { testata };
        schermo::imposta_zona_chiudi_pagina(ui.ctx(), Some(zona.intersect(ui.clip_rect())));
    }
    if telefono && !compatta && in_fondo && !super::barra_tastiera::tastiera_aperta() {
        if let Some(nota) = schermo::area_pulsanti(ui.ctx()) {
            let meta = egui::Rect::from_min_max(egui::pos2(nota.left(), nota.center().y), nota.max);
            schermo::imposta_zona_apri_panoramica(ui.ctx(), Some(meta));
        }
    }
    // Allineato al "+" e alla maniglia: tutti e tre partono dalla stessa area.
    let area_pulsanti = schermo::area_pulsanti(ui.ctx()).unwrap_or(area_testo);
    if con_pulsante_lista && pulsante_lista(ui.ctx(), t, &app.tr, area_pulsanti) {
        cambiato |= if ui.memory(|m| m.has_focus(id_testo)) {
            // Mentre si scrive: la riga del cursore diventa una voce (o torna testo).
            testo_nota::voce_lista(ui.ctx(), id_testo, &mut testo)
        } else {
            // Mentre si legge: una voce nuova in fondo, pronta da scrivere.
            testo_nota::aggiungi_voce_in_fondo(ui.ctx(), id_testo, &mut testo)
        };
    }
    // Computer: in basso al centro della nota, sopra la linea di ☐, i
    // messaggi brevi (non c'e' piu' la riga di stato sotto la nota).
    if !telefono && !compatta {
        let sopra = area_pulsanti.bottom() - LATO_PULSANTE - 8.0;
        super::riga_stato::avvisi(ui.ctx(), app, t, area_pulsanti, sopra);
    }

    if let Some(nota) = app.nota_mut(&id) {
        if cambiato {
            nota.titolo = titolo;
            nota.testo = testo;
            nota.colore = colore_nota;
        }
        if conflitto_risolto {
            nota.conflitto_di = None;
            cambiato = true;
        }
    }
    if cambiato {
        app.registra_modifica(&id);
    }
    if alterna_fissata {
        app.alterna_fissata(&id);
    }
    if apri_cronologia {
        app.apri_cronologia(&id);
    }
    if apri_widget {
        app.apri_widget(&id);
    }
    if let Some(originale) = apri_originale {
        app.interfaccia.selezionata = Some(originale);
    }
    if elimina && telefono {
        // Telefono: come dal menu del Cestino, niente conferma ma ANNULLA per
        // 10 secondi. Si passa alla nota del cestino accanto (come sfogliando);
        // se non ce ne sono altre, alle schede del Cestino.
        let vicina = app.nota_vicina(&id, 1).or_else(|| app.nota_vicina(&id, -1));
        app.elimina_con_annulla(vec![id.clone()]);
        if let Some(vicina) = vicina {
            app.interfaccia.selezionata = Some(vicina);
        } else {
            app.interfaccia.pagina = crate::applicazione::Pagina::Cestino;
            app.interfaccia.panoramica = true;
            ui.ctx().animate_value_with_time(Id::unique("apertura_cestino"), 1.0, 0.0);
        }
    } else if elimina {
        // Prima chiede conferma: l'eliminazione vale anche sugli altri computer.
        app.avviso = Some(Avviso::ConfermaEliminazione(id.clone()));
    }
    if azione_cestino {
        if nel_cestino {
            app.ripristina(&id);
        } else {
            app.sposta_nel_cestino(&id);
        }
    }
}
