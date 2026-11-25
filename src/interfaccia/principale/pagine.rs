//! Telefono: le pagine a schermo intero che sul computer stanno nel menu di
//! destra.
//!
//! ```text
//! ┌──────────────────────────────┐
//! │ ‹  Cestino                   │
//! │ Note nel cestino: 3          │
//! │ ┌────────────┐ ┌───────────┐ │
//! │ │ Spese  │ R │ │ Idee  │ R │ │  schede come nella panoramica:
//! │ │ pane   │ P │ │ ...   │ P │ │  un tocco apre la nota
//! │ └────────────┘ └───────────┘ │
//! │ [ Svuota cestino ]           │
//! └──────────────────────────────┘
//! ```
//!
//! - **Impostazioni**: Aspetto, Rete locale, Cartella e backup, Applicazione;
//! - **Cestino**: le note cestinate, come schede (la "panoramica del
//!   cestino"). Un tocco apre la nota a pagina intera: di lato si scorre tra
//!   le altre del cestino, dal basso verso l'alto si torna alle schede del
//!   Cestino, trascinando in giu' la testata si torna alle note. Tenendo
//!   premuta una scheda compare un menu con Ripristina ed Elimina
//!   definitivamente, e dopo l'eliminazione per 10 secondi c'e' ANNULLA.
//!
//! Le due pagine stanno ai lati della panoramica: il Cestino a sinistra, le
//! Impostazioni a destra. «‹» o il tasto Indietro tornano alla panoramica;
//! trascinando in giu' si torna alla nota da cui si era partiti.

use egui::{RichText, Ui};

use crate::applicazione::{Applicazione, Avviso, Pagina};
use crate::interfaccia::componenti::{Icona, nota_attenuata, pulsante_icona, pulsante_largo_colorato};
use crate::interfaccia::schermo;
use crate::interfaccia::tema::{TITOLO_PAGINA, Tavolozza};

use super::barra_laterale;

/// Altezza della riga con «‹» e il titolo.
const ALTEZZA_TESTATA: f32 = 44.0;

/// La pagina si disegna in posti diversi (a riposo, mentre entra di lato,
/// mentre scende): con lo stesso `scope_id` dappertutto le sue aree
/// scorrevoli restano dove erano, senza saltare a fine animazione.
pub fn mostra(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza, pagina: Pagina) {
    let ambito = egui::Id::unique(("ambito_pagina", pagina as u8));
    ui.scope_builder(egui::UiBuilder::new().scope_id(ambito), |ui| contenuto(ui, app, t, pagina));
}

fn contenuto(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza, pagina: Pagina) {
    let titolo = match pagina {
        Pagina::Impostazioni => "barra_impostazioni",
        Pagina::Cestino => "sezione_cestino",
        Pagina::Nota => return,
    };
    let inizio = ui.cursor().min;
    ui.horizontal(|ui| {
        ui.set_height(ALTEZZA_TESTATA);
        if pulsante_icona(ui, t, Icona::Precedente, app.tr.t("aiuto_indietro"), true).clicked() {
            torna_alla_panoramica(app);
        }
        ui.add_space(4.0);
        ui.label(RichText::new(app.tr.t(titolo)).size(TITOLO_PAGINA).color(t.testo));
    });
    // Trascinando la testata verso il basso si torna alla nota
    // (`Programma::smista_gesti`); anche dal resto della pagina, quando e' in cima.
    let testata = egui::Rect::from_min_max(inizio, egui::pos2(ui.max_rect().right(), ui.cursor().min.y));
    ui.add_space(6.0);
    let in_cima = match pagina {
        Pagina::Impostazioni => barra_laterale::impostazioni(ui, app, t),
        Pagina::Cestino => cestino(ui, app, t),
        Pagina::Nota => true,
    };
    let zona = if in_cima { testata.union(ui.max_rect()) } else { testata };
    schermo::imposta_zona_chiudi_pagina(ui.ctx(), Some(zona));
}

/// Apre il Cestino (a sinistra della panoramica) e ricorda la nota aperta.
pub fn apri_cestino(app: &mut Applicazione) {
    app.conferma_rinomina();
    let aperta = app.interfaccia.selezionata.clone().filter(|id| app.nota(id).is_some_and(|n| !n.nel_cestino));
    if aperta.is_some() {
        app.interfaccia.prima_del_cestino = aperta;
    }
    app.interfaccia.pagina = Pagina::Cestino;
    app.interfaccia.panoramica = true;
}

/// Da Impostazioni o Cestino si torna alla panoramica (con «‹», Indietro o
/// di lato). Se si guardava una nota del cestino, torna aperta quella di
/// prima (o la prima delle note): la panoramica e' quella delle note.
pub fn torna_alla_panoramica(app: &mut Applicazione) {
    app.interfaccia.pagina = Pagina::Nota;
    app.interfaccia.panoramica = true;
    ripristina_nota_aperta(app, false);
}

/// Da Impostazioni o Cestino, trascinando in giu', si torna alla nota da cui
/// si era partiti: anche a una nota del cestino, se si era saliti da li'.
pub fn torna_alla_nota(app: &mut Applicazione) {
    app.interfaccia.pagina = Pagina::Nota;
    app.interfaccia.panoramica = false;
    ripristina_nota_aperta(app, true);
}

/// Da una nota del cestino, trascinando in giu' la sua testata, si torna
/// alle note (a quella aperta prima di entrare nel Cestino).
pub fn esci_dal_cestino(app: &mut Applicazione) {
    app.interfaccia.pagina = Pagina::Nota;
    app.interfaccia.panoramica = false;
    ripristina_nota_aperta(app, false);
}

/// Una nota del cestino, a pagina intera (dalle schede del Cestino).
pub fn apri_nota_del_cestino(app: &mut Applicazione, id: String) {
    app.conferma_rinomina();
    app.interfaccia.selezionata = Some(id);
    app.interfaccia.pagina = Pagina::Nota;
    app.interfaccia.panoramica = false;
}

/// La nota c'e' ed e' nel cestino (e non e' appena stata eliminata)?
pub fn nel_cestino_visibile(app: &Applicazione, id: &str) -> bool {
    app.nota(id).is_some_and(|n| n.nel_cestino) && !app.in_eliminazione(id)
}

/// La nota aperta non deve restare una del cestino (o nessuna), a meno che
/// `anche_cestino`: allora puo' restare quella del cestino aperta.
fn ripristina_nota_aperta(app: &mut Applicazione, anche_cestino: bool) {
    let attiva = |app: &Applicazione, id: &str| app.nota(id).is_some_and(|n| !n.nel_cestino && n.visibile());
    let aperta = app.interfaccia.selezionata.clone();
    if aperta.as_deref().is_some_and(|id| attiva(app, id) || (anche_cestino && nel_cestino_visibile(app, id))) {
        return;
    }
    let prima = app.interfaccia.prima_del_cestino.take().filter(|id| attiva(app, id));
    app.interfaccia.selezionata = prima.or_else(|| app.note_attive_id().into_iter().next());
}

/// Le note nel cestino come schede (come la panoramica delle note), con in
/// cima la ricerca del Cestino: con una ricerca in corso solo quelle che
/// corrispondono. `true` se le schede sono scorse in cima.
fn cestino(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) -> bool {
    if app.cartella.is_none() {
        return true;
    }
    // Senza quelle appena eliminate (finche' c'e' ANNULLA).
    let totale = app.note_cestino_visibili().len();
    if totale == 0 {
        nota_attenuata(ui, t, app.tr.t("cestino_vuoto"));
        return true;
    }
    // Ricerca, conteggio, schede, suggerimento e Svuota cestino con lo spazio
    // standard tra loro, come nella sezione Cestino del computer.
    super::ricerca::campo_cestino(ui, app, t);
    let ricerca = app.interfaccia.ricerca_cestino.clone();
    let note: Vec<String> =
        app.note_cestino_visibili().into_iter().filter(|n| n.corrisponde(&ricerca)).map(|n| n.id.clone()).collect();
    if ricerca.trim().is_empty() {
        nota_attenuata(ui, t, &app.tr.f("cestino_conteggio", &[("n", &totale.to_string())]));
    } else {
        let testo = app.tr.f("cestino_trovate", &[("n", &note.len().to_string()), ("totale", &totale.to_string())]);
        nota_attenuata(ui, t, &testo);
    }
    // In fondo restano il suggerimento e Svuota cestino.
    let altezza = (ui.available_height() - 100.0).max(120.0);
    let in_cima = super::schede::caselle_cestino(ui, app, t, &note, altezza);
    nota_attenuata(ui, t, app.tr.t("cestino_istruzioni"));
    let svuota = pulsante_largo_colorato(ui, t, app.tr.t("svuota_cestino"), t.pericolo);
    if svuota.on_hover_text(app.tr.t("aiuto_svuota_cestino")).clicked() {
        app.avviso = Some(Avviso::ConfermaSvuotaCestino);
    }
    in_cima
}
