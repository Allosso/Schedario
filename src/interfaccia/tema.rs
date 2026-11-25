//! Tavolozze chiara e scura e stile comune a tutte le finestre.
//!
//! Tema chiaro "carta": toni caldi e un accento blu petrolio.
//! Tema scuro "ardesia": grigi bluastri e un accento azzurro.

use egui::{Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Theme, Vec2};

use super::schermo::misura;

/// Raggio degli angoli condiviso da pulsanti, schede e contenitori: poco
/// arrotondati, per uno stile sobrio come le linguette di un raccoglitore.
pub const RAGGIO: u8 = 6;
pub const RAGGIO_CONTENITORI: u8 = 8;
/// Larghezza massima delle descrizioni al passaggio del mouse (poi vanno a capo).
pub const LARGHEZZA_DESCRIZIONI: f32 = 360.0;
/// Spazio standard tra due controlli.
pub const SPAZIO: f32 = 6.0;

// Dimensioni dei testi, le stesse in tutto il programma (menu, pagine,
// dialoghi, widget). Il testo normale e' 14 (`TextStyle::Body`).
/// Suggerimenti, etichette sopra i campi, stati ("Sincronizzazione spenta"), errori.
pub const TESTO_SECONDARIO: f32 = 12.5;
/// Voci piccole di un elenco: dispositivi collegati, indirizzi aggiunti a mano.
pub const TESTO_VOCE: f32 = 13.0;
/// Quando non c'e' niente da mostrare: "Nessuna nota", "Scegli una nota".
pub const TESTO_VUOTO: f32 = 14.0;
/// Titolo dei dialoghi (Informazioni, conferme, Cronologia).
pub const TITOLO_DIALOGO: f32 = 16.0;
/// Telefono: titolo delle pagine a schermo intero (panoramica, Cestino, Impostazioni).
pub const TITOLO_PAGINA: f32 = 18.0;
/// Contorni che dicono "scelto" o "attenzione": scheda aperta, scelta attiva, conflitto.
pub const TRATTO_ATTIVO: f32 = 1.5;
/// Il rosso del Cestino mentre ci si trascina una nota: leggero, e piu' pieno
/// con la nota proprio sopra.
pub const PERICOLO_LEGGERO: f32 = 0.12;
pub const PERICOLO_SOPRA: f32 = 0.3;

#[derive(Clone, Copy, Debug)]
pub struct Tavolozza {
    pub scuro: bool,
    pub sfondo: Color32,
    pub pannello: Color32,
    pub superficie: Color32,
    pub superficie_attiva: Color32,
    pub testo: Color32,
    pub attenuato: Color32,
    pub bordo: Color32,
    pub accento: Color32,
    pub testo_accento: Color32,
    pub pericolo: Color32,
    pub successo: Color32,
    pub selezione: Color32,
    /// Fondo delle parole trovate dalla ricerca.
    pub evidenzia: Color32,
}

impl Tavolozza {
    pub fn chiara() -> Self {
        Self {
            scuro: false,
            sfondo: Color32::from_rgb(0xEF, 0xEB, 0xE4),
            pannello: Color32::from_rgb(0xFA, 0xF8, 0xF4),
            superficie: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            superficie_attiva: Color32::from_rgb(0xE8, 0xEE, 0xF1),
            testo: Color32::from_rgb(0x2A, 0x2A, 0x28),
            attenuato: Color32::from_rgb(0x74, 0x70, 0x6A),
            bordo: Color32::from_rgb(0xD3, 0xCC, 0xC1),
            accento: Color32::from_rgb(0x2F, 0x6B, 0x7E),
            testo_accento: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            pericolo: Color32::from_rgb(0xB8, 0x43, 0x36),
            successo: Color32::from_rgb(0x3B, 0x7F, 0x55),
            selezione: Color32::from_rgb(0xD6, 0xE5, 0xEA),
            evidenzia: Color32::from_rgb(0xFF, 0xE0, 0x7A),
        }
    }

    pub fn scura() -> Self {
        Self {
            scuro: true,
            sfondo: Color32::from_rgb(0x17, 0x1A, 0x1E),
            pannello: Color32::from_rgb(0x20, 0x24, 0x29),
            superficie: Color32::from_rgb(0x29, 0x2E, 0x34),
            superficie_attiva: Color32::from_rgb(0x33, 0x3B, 0x44),
            testo: Color32::from_rgb(0xE4, 0xE2, 0xDD),
            attenuato: Color32::from_rgb(0x98, 0x9E, 0xA5),
            bordo: Color32::from_rgb(0x3B, 0x42, 0x4A),
            accento: Color32::from_rgb(0x6C, 0xAE, 0xC4),
            testo_accento: Color32::from_rgb(0x10, 0x1A, 0x1F),
            pericolo: Color32::from_rgb(0xE0, 0x6B, 0x5C),
            successo: Color32::from_rgb(0x67, 0xB5, 0x83),
            selezione: Color32::from_rgb(0x2E, 0x46, 0x52),
            evidenzia: Color32::from_rgb(0x6E, 0x5A, 0x1C),
        }
    }

    pub fn per(scuro: bool) -> Self {
        if scuro { Self::scura() } else { Self::chiara() }
    }

    pub fn tratto(&self) -> Stroke {
        Stroke::new(1.0, self.bordo)
    }
}

/// Tinte di un colore di nota: `fondo` riempie il widget, `segno` colora il
/// contorno, la striscia sulla scheda e il pallino del selettore.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TintaNota {
    pub fondo: Color32,
    pub segno: Color32,
}

/// Tinte di un colore di nota (vedi `modello::nota::COLORI`) per il tema
/// indicato. `None` per "nessun colore" o per un nome sconosciuto.
pub fn tinta_nota(colore: &str, scuro: bool) -> Option<TintaNota> {
    let rgb = |c: u32| Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8);
    // (fondo chiaro, segno chiaro, fondo scuro, segno scuro)
    let (fc, sc, fs, ss) = match colore {
        "giallo" => (0xFFF3C4, 0xC99A06, 0x38331E, 0xE6C24F),
        "arancio" => (0xFFE4CC, 0xD2701E, 0x3B2C1F, 0xE89558),
        "rosa" => (0xFADDE5, 0xC94F72, 0x3B242B, 0xE07A97),
        "viola" => (0xE9E0F5, 0x8059C2, 0x2E2840, 0xA98AE0),
        "azzurro" => (0xDAECF7, 0x3584B8, 0x1E3140, 0x6CB2E0),
        "verde" => (0xDDEFDA, 0x46914A, 0x213526, 0x72BE76),
        _ => return None,
    };
    let (fondo, segno) = if scuro { (fs, ss) } else { (fc, sc) };
    Some(TintaNota { fondo: rgb(fondo), segno: rgb(segno) })
}

/// Applica tavolozza, caratteri e spaziature a tutte le finestre.
pub fn applica(ctx: &egui::Context, t: &Tavolozza) {
    ctx.set_theme(if t.scuro { Theme::Dark } else { Theme::Light });
    ctx.all_styles_mut(|stile| {
        let mut v = if t.scuro { egui::Visuals::dark() } else { egui::Visuals::light() };
        v.panel_fill = t.sfondo;
        v.window_fill = t.pannello;
        v.window_stroke = t.tratto();
        v.window_corner_radius = CornerRadius::same(RAGGIO_CONTENITORI);
        v.menu_corner_radius = CornerRadius::same(RAGGIO);
        v.extreme_bg_color = t.superficie;
        v.text_edit_bg_color = Some(t.superficie);
        v.faint_bg_color = t.pannello;
        v.override_text_color = Some(t.testo);
        v.weak_text_color = Some(t.attenuato);
        v.hyperlink_color = t.accento;
        v.selection.bg_fill = t.selezione;
        v.selection.stroke = Stroke::new(TRATTO_ATTIVO, t.accento);
        v.error_fg_color = t.pericolo;
        v.warn_fg_color = t.pericolo;
        v.window_shadow = egui::Shadow::NONE;
        v.popup_shadow = egui::Shadow::NONE;
        v.text_cursor.stroke = Stroke::new(2.0, t.accento);

        // Pulsanti come le schede (vedi `componenti`): il fondo dei pulsanti
        // (`weak_bg_fill`) e' quello su cui stanno e si schiarisce sotto il
        // mouse; il bordo resta sottile e neutro. Caselle e simili (`bg_fill`)
        // restano piene.
        let angoli = CornerRadius::same(RAGGIO);
        for (stato, riempimento, pulsante, tratto) in [
            (&mut v.widgets.noninteractive, t.pannello, t.pannello, t.bordo),
            (&mut v.widgets.inactive, t.superficie, Color32::TRANSPARENT, t.bordo),
            (&mut v.widgets.hovered, t.superficie_attiva, t.superficie_attiva, t.bordo),
            (&mut v.widgets.active, t.selezione, t.superficie_attiva, t.bordo),
            (&mut v.widgets.open, t.superficie_attiva, t.superficie_attiva, t.bordo),
        ] {
            stato.bg_fill = riempimento;
            stato.weak_bg_fill = pulsante;
            stato.bg_stroke = Stroke::new(1.0, tratto);
            stato.fg_stroke = Stroke::new(1.0, t.testo);
            stato.corner_radius = angoli;
            stato.expansion = 0.0;
        }
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.testo);
        stile.visuals = v;

        stile.text_styles = [
            (TextStyle::Small, FontId::new(TESTO_SECONDARIO, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
        ]
        .into();
        // Le descrizioni compaiono solo se il mouse si ferma per mezzo secondo:
        // passando sopra i pulsanti non coprono quello che c'e' sotto.
        // Con il dito la descrizione compare tenendo premuto per quasi mezzo
        // secondo: proprio quando il tocco diventa una pressione lunga, che
        // non preme il pulsante (vedi `programma`).
        stile.interaction.tooltip_delay = misura(0.5, 0.45);
        stile.spacing.tooltip_width = misura(LARGHEZZA_DESCRIZIONI, 280.0);
        stile.interaction.show_tooltips_only_when_still = true;
        stile.spacing.item_spacing = Vec2::splat(SPAZIO);
        stile.spacing.button_padding = Vec2::new(10.0, 5.0);
        // Caselle, voci dei menu, elenchi: piu' alti per il dito.
        stile.spacing.interact_size = Vec2::new(28.0, misura(28.0, 40.0));
        stile.spacing.icon_width = misura(14.0, 20.0);
        stile.spacing.icon_width_inner = misura(8.0, 12.0);
        stile.spacing.window_margin = Margin::same(12);
        stile.spacing.menu_margin = Margin::same(6);
    });
}

/// Caratteri del programma: Inter (licenza OFL, `assets/caratteri`) per tutto
/// il testo; quelli predefiniti di egui restano dietro, per i simboli che
/// Inter non ha (emoji, alfabeti rari).
pub fn caratteri() -> egui::FontDefinitions {
    let mut definizioni = egui::FontDefinitions::default();
    definizioni.font_data.insert(
        "Inter".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../../assets/caratteri/Inter-Regular.ttf"))),
    );
    definizioni.families.entry(FontFamily::Proportional).or_default().insert(0, "Inter".to_owned());
    // Inter SemiBold per i titoli delle note; per i simboli mancanti, gli stessi di sopra.
    definizioni.font_data.insert(
        "Inter SemiBold".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../../assets/caratteri/Inter-SemiBold.ttf"))),
    );
    let mut titoli = vec!["Inter SemiBold".to_owned()];
    titoli.extend(definizioni.families[&FontFamily::Proportional].iter().cloned());
    definizioni.families.insert(FontFamily::Name(FAMIGLIA_TITOLI.into()), titoli);
    definizioni
}

/// Famiglia di caratteri dei titoli delle note (Inter SemiBold).
const FAMIGLIA_TITOLI: &str = "titoli";

/// Interlinea del testo delle note, in multipli della dimensione del carattere.
pub const INTERLINEA: f32 = 1.45;

/// Carattere del testo delle note, nella dimensione scelta dall'utente.
pub fn carattere_nota(dimensione: f32) -> FontId {
    FontId::new(dimensione, FontFamily::Proportional)
}

/// Titolo della nota nell'editor: piu' grande del testo e in grassetto.
pub fn carattere_titolo() -> FontId {
    FontId::new(22.0, FontFamily::Name(FAMIGLIA_TITOLI.into()))
}

/// Titolo di una nota in piccolo: schede della panoramica, barra del widget.
pub fn carattere_titolo_piccolo() -> FontId {
    FontId::new(15.0, FontFamily::Name(FAMIGLIA_TITOLI.into()))
}
