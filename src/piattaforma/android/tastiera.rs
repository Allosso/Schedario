//! Ponte tra la tastiera di Android e i campi di testo di egui.
//!
//! Con GameActivity la tastiera tiene una copia del testo del campo e la
//! modifica lei (accenti, suggerimenti, scrittura a scorrimento). winit 0.30
//! non inoltra questi cambiamenti, quindi:
//!
//! 1. quando un campo prende il fuoco, gli si da' il suo testo e il cursore;
//! 2. prima di ogni fotogramma si confronta il testo della tastiera con
//!    l'ultimo noto e la differenza diventa un evento normale di egui. Per
//!    esempio, se "citta" diventa "città " si seleziona la "a" finale e si
//!    scrive "à ": il campo la tratta come se fosse stata digitata, quindi
//!    valgono annulla, liste, scorrimento fino al cursore;
//! 3. se invece il testo cambia da egui (Taglia, una voce di lista nuova,
//!    un tocco che sposta il cursore) il nuovo stato va alla tastiera.
//!
//! Vale per qualunque campo, purche' chiami `interfaccia::tastiera::campo`
//! dopo essersi disegnato.

use std::time::{Duration, Instant};

use egui::text::{CCursor, CCursorRange};
use egui::text_edit::TextEditState;
use egui::{Event, Id, Key, Modifiers, RawInput};
use winit::platform::android::activity::input::{ImeOptions, InputType, TextInputAction, TextInputState, TextSpan};

use super::APP;
use crate::interfaccia::tastiera::{CampoAttivo, a_utf16, da_utf16, differenza};

/// Testo e posizioni in caratteri (come egui).
#[derive(Clone, Debug, PartialEq, Eq)]
struct Stato {
    testo: String,
    selezione: (usize, usize),
}

/// Quanto aspettare che la tastiera accetti uno stato mandato da qui.
const ATTESA_MASSIMA: Duration = Duration::from_millis(600);

#[derive(Default)]
pub struct Ponte {
    /// Campo collegato: id, va a capo, riservato (codice).
    campo: Option<(Id, bool, bool)>,
    /// Ultimo stato su cui tastiera e campo sono d'accordo.
    noto: Option<Stato>,
    /// Stato appena mandato alla tastiera, finche' non lo conferma.
    mandato: Option<(Stato, Instant)>,
    /// In questo fotogramma il testo e' cambiato dalla tastiera.
    da_tastiera: bool,
    /// La finestra e' tornata attiva: se c'e' un campo con il fuoco la
    /// tastiera va riaperta (Android ignora la richiesta fatta mentre la
    /// finestra non era attiva, per esempio aprendo "Nuova nota" dall'icona).
    riapri: bool,
}

impl Ponte {
    /// Prima del fotogramma: traduce in eventi egui cio' che la tastiera ha
    /// cambiato e toglie gli eventi doppi (GameActivity manda anche i tasti
    /// cancella e Invio, gia' compresi nel testo della tastiera).
    pub fn prima(&mut self, ctx: &egui::Context, input: &mut RawInput) {
        self.da_tastiera = false;
        for evento in &input.events {
            if let Event::WindowFocused(attiva) = evento {
                self.riapri = *attiva;
            }
        }
        let Some((id, multiriga, _)) = self.campo else { return };
        input.events.retain(|evento| {
            !matches!(
                evento,
                Event::Text(_) | Event::Ime(_) | Event::Key { key: Key::Backspace | Key::Delete | Key::Enter, .. }
            )
        });
        let Some(app) = APP.get() else { return };
        let stato = converti(app.text_input_state());
        if let Some((mandato, quando)) = &self.mandato {
            if *mandato == stato || quando.elapsed() > ATTESA_MASSIMA {
                self.mandato = None;
            } else {
                // La tastiera non ha ancora ricevuto l'ultimo stato: si aspetta.
                ctx.request_repaint_after(Duration::from_millis(30));
                return;
            }
        }
        let Some(noto) = self.noto.clone() else { return };
        if noto == stato {
            return;
        }
        self.noto = Some(stato.clone());
        self.da_tastiera = true;
        if noto.testo == stato.testo {
            // Solo il cursore (per esempio spostato dalla tastiera stessa).
            let mut memoria = TextEditState::load(ctx, id).unwrap_or_default();
            let (a, b) = stato.selezione;
            memoria.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(a), CCursor::new(b))));
            memoria.store(ctx, id);
            return;
        }
        let (da, a, inserito) = differenza(&noto.testo, &stato.testo);
        // Si seleziona la parte cambiata, poi la si sostituisce come se fosse
        // stata scritta.
        let mut memoria = TextEditState::load(ctx, id).unwrap_or_default();
        memoria.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(da), CCursor::new(a))));
        memoria.store(ctx, id);
        let tasto = |key| Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE };
        if inserito.is_empty() {
            input.events.push(tasto(Key::Backspace));
        }
        for (i, pezzo) in inserito.split('\n').enumerate() {
            if i > 0 {
                // A capo nel testo; in un campo di una riga vale come conferma.
                input.events.push(tasto(Key::Enter));
                if !multiriga {
                    break;
                }
            }
            if !pezzo.is_empty() {
                input.events.push(Event::Text(pezzo.to_owned()));
            }
        }
    }

    /// Dopo il fotogramma: il campo con il fuoco e com'e' adesso.
    pub fn dopo(&mut self, ctx: &egui::Context, attivo: Option<CampoAttivo>) {
        let Some(campo) = attivo else {
            self.campo = None;
            self.noto = None;
            self.mandato = None;
            // Nessun campo in uso tornando nell'app: niente da riaprire.
            self.riapri = false;
            return;
        };
        if std::mem::take(&mut self.riapri) {
            if let Some(app) = APP.get() {
                app.show_soft_input(false);
            }
        }
        let attuale = Stato { testo: campo.testo, selezione: campo.selezione };
        let chiave = (campo.id, campo.multiriga, campo.riservato);
        if self.campo != Some(chiave) {
            self.campo = Some(chiave);
            imposta_tipo(campo.multiriga, campo.riservato);
            self.manda(attuale);
            return;
        }
        let Some(noto) = &self.noto else {
            self.manda(attuale);
            return;
        };
        if attuale == *noto {
            return;
        }
        if attuale.testo == noto.testo && self.da_tastiera {
            // Il testo e' quello della tastiera, il cursore no: vale il suo.
            let mut memoria = TextEditState::load(ctx, campo.id).unwrap_or_default();
            let (a, b) = noto.selezione;
            memoria.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(a), CCursor::new(b))));
            memoria.store(ctx, campo.id);
            ctx.request_repaint();
            return;
        }
        self.manda(attuale);
    }

    fn manda(&mut self, stato: Stato) {
        let Some(app) = APP.get() else { return };
        let testo = &stato.testo;
        app.set_text_input_state(TextInputState {
            text: testo.clone(),
            selection: TextSpan { start: a_utf16(testo, stato.selezione.0), end: a_utf16(testo, stato.selezione.1) },
            compose_region: None,
        });
        self.mandato = Some((stato.clone(), Instant::now()));
        self.noto = Some(stato);
    }
}

/// Dice alla tastiera che tipo di campo e': maiuscola a inizio frase e
/// suggerimenti, a capo nel testo, nessun suggerimento per i codici.
fn imposta_tipo(multiriga: bool, riservato: bool) {
    let Some(app) = APP.get() else { return };
    let mut tipo = InputType::TYPE_CLASS_TEXT;
    if riservato {
        tipo |= InputType::TYPE_TEXT_VARIATION_PASSWORD;
    } else {
        tipo |= InputType::TYPE_TEXT_FLAG_CAP_SENTENCES | InputType::TYPE_TEXT_FLAG_AUTO_CORRECT;
    }
    let azione = if multiriga {
        tipo |= InputType::TYPE_TEXT_FLAG_MULTI_LINE;
        TextInputAction::None
    } else {
        TextInputAction::Done
    };
    // In orizzontale molte tastiere passano a tutto schermo, con una casella
    // di testo loro che copre l'app: si chiede di non farlo.
    // (Il nome IMG_ con la G e' un refuso di android-activity.)
    app.set_ime_editor_info(tipo, azione, ImeOptions::IME_FLAG_NO_FULLSCREEN | ImeOptions::IMG_FLAG_NO_EXTRACT_UI);
}

fn converti(stato: TextInputState) -> Stato {
    let testo = stato.text;
    let (a, b) = (da_utf16(&testo, stato.selection.start), da_utf16(&testo, stato.selection.end));
    Stato { selezione: (a.min(b), a.max(b)), testo }
}
