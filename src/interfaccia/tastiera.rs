//! Collegamento dei campi di testo con la tastiera del telefono.
//!
//! Ogni campo, dopo essersi disegnato, chiama `campo`: se ha il fuoco, su
//! Android il ponte (`piattaforma::android::tastiera`) lo collega alla
//! tastiera. Sul computer non fa nulla.

use egui::text_edit::TextEditOutput;
use egui::{Id, Ui};

/// Il campo con il fuoco in questo fotogramma.
#[derive(Clone, Debug)]
pub struct CampoAttivo {
    pub id: Id,
    pub testo: String,
    /// Selezione in caratteri (inizio, fine); vuota = cursore.
    pub selezione: (usize, usize),
    /// Va a capo (testo della nota) o e' di una riga sola.
    pub multiriga: bool,
    /// Codice o password: niente suggerimenti della tastiera.
    pub riservato: bool,
}

fn chiave() -> Id {
    Id::unique("campo_tastiera")
}

fn chiave_cursore() -> Id {
    Id::unique("tastiera_mostra_cursore")
}

/// Da chiamare dopo aver disegnato un campo di testo.
pub fn campo(ui: &Ui, uscita: &TextEditOutput, testo: &str, multiriga: bool, riservato: bool) {
    if !cfg!(target_os = "android") || !uscita.response.has_focus() {
        return;
    }
    let ctx = ui.ctx();
    // Con il dito egui, quando un campo non ancora in uso viene toccato, gli
    // da' il fuoco ma mette il cursore in fondo al testo. Il primo tocco deve
    // invece metterlo sotto il dito, come negli altri campi del telefono.
    let appena_toccato = uscita.response.clicked() && campo_precedente(ctx).is_none_or(|c| c.id != uscita.response.id);
    if super::schermo::TOCCO && appena_toccato {
        if let Some(punto) = uscita.response.interact_pointer_pos() {
            let cursore = uscita.galley.cursor_from_pos(punto - uscita.galley_pos);
            let mut stato = uscita.state.clone();
            stato.cursor.set_char_range(Some(egui::text::CCursorRange::one(cursore)));
            stato.store(ctx, uscita.response.id);
            let i = cursore.index.0;
            let attivo = CampoAttivo { id: uscita.response.id, testo: testo.to_owned(), selezione: (i, i), multiriga, riservato };
            ctx.data_mut(|d| d.insert_temp(chiave(), Some(attivo)));
            ctx.request_repaint();
            return;
        }
    }
    let selezione = match uscita.cursor_range {
        Some(r) => {
            let (a, b) = (r.secondary.index.0, r.primary.index.0);
            (a.min(b), a.max(b))
        }
        None => {
            let fine = testo.chars().count();
            (fine, fine)
        }
    };
    let attivo = CampoAttivo { id: uscita.response.id, testo: testo.to_owned(), selezione, multiriga, riservato };
    ctx.data_mut(|d| d.insert_temp(chiave(), Some(attivo)));
    // La tastiera si e' appena aperta: la riga del cursore resta in vista.
    if ctx.data_mut(|d| d.remove_temp::<bool>(chiave_cursore())).unwrap_or(false) {
        if let Some(r) = uscita.cursor_range {
            let rettangolo = uscita.galley.pos_from_cursor(r.primary).translate(uscita.galley_pos.to_vec2()).expand(12.0);
            ui.scroll_to_rect(rettangolo, None);
        }
    }
}

/// Il campo con il fuoco nell'ultimo fotogramma (una volta sola). Ne resta
/// una copia per `campo_precedente`.
pub fn prendi(ctx: &egui::Context) -> Option<CampoAttivo> {
    let campo = ctx.data_mut(|d| d.remove_temp::<Option<CampoAttivo>>(chiave())).flatten();
    ctx.data_mut(|d| d.insert_temp(chiave_precedente(), campo.clone()));
    campo
}

/// Il campo che aveva il fuoco alla fine del fotogramma precedente (per la
/// barra sopra la tastiera, che si disegna prima dei campi).
pub fn campo_precedente(ctx: &egui::Context) -> Option<CampoAttivo> {
    ctx.data(|d| d.get_temp::<Option<CampoAttivo>>(chiave_precedente())).flatten()
}

fn chiave_precedente() -> Id {
    Id::unique("campo_tastiera_precedente")
}

/// Azioni sul campo con il fuoco, chieste da un pulsante (barra sopra la
/// tastiera, menu tenendo premuto). Arrivano al campo come i tasti
/// corrispondenti, cosi' valgono le stesse regole: Annulla = Ctrl+Z,
/// Ripeti = Ctrl+Y, Voce di lista = Ctrl+L.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AzioneTesto {
    Annulla,
    Ripeti,
    VoceLista,
}

fn chiave_azioni() -> Id {
    Id::unique("azioni_sul_testo")
}

/// Chiede l'azione per il prossimo fotogramma.
pub fn richiedi(ctx: &egui::Context, azione: AzioneTesto) {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Vec<AzioneTesto>>(chiave_azioni()).push(azione));
    ctx.request_repaint();
}

/// All'inizio del fotogramma: le azioni chieste diventano tasti.
pub fn applica_richieste(ctx: &egui::Context) {
    let azioni: Vec<AzioneTesto> = ctx.data_mut(|d| d.remove_temp(chiave_azioni())).unwrap_or_default();
    if azioni.is_empty() {
        return;
    }
    ctx.input_mut(|i| {
        for azione in azioni {
            let key = match azione {
                AzioneTesto::Annulla => egui::Key::Z,
                AzioneTesto::Ripeti => egui::Key::Y,
                AzioneTesto::VoceLista => egui::Key::L,
            };
            i.events.push(egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::COMMAND });
        }
    });
}

/// C'e' qualcosa da annullare (o da ripetere) nel campo `id` con testo `testo`?
pub fn si_puo(ctx: &egui::Context, id: Id, testo: &str, ripeti: bool) -> bool {
    let Some(stato) = egui::text_edit::TextEditState::load(ctx, id) else { return false };
    let attuale = (stato.cursor.char_range().unwrap_or_default(), testo.to_owned());
    let annullatore = stato.undoer();
    if ripeti { annullatore.has_redo(&attuale) } else { annullatore.has_undo(&attuale) }
}

/// Chiede al campo con il fuoco di portare in vista il cursore.
pub fn mostra_cursore(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(chiave_cursore(), true));
}

// Funzioni pure usate dal ponte di Android (qui per poterle provare anche sul computer).

/// Cosa e' cambiato da `prima` a `dopo`: i caratteri `da..a` di `prima` sono
/// stati sostituiti dal testo restituito.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) fn differenza(prima: &str, dopo: &str) -> (usize, usize, String) {
    let vecchi: Vec<char> = prima.chars().collect();
    let nuovi: Vec<char> = dopo.chars().collect();
    let inizio = vecchi.iter().zip(&nuovi).take_while(|(a, b)| a == b).count();
    let massimo = vecchi.len().min(nuovi.len()) - inizio;
    let fine = vecchi.iter().rev().zip(nuovi.iter().rev()).take(massimo).take_while(|(a, b)| a == b).count();
    let inserito = nuovi[inizio..nuovi.len() - fine].iter().collect();
    (inizio, vecchi.len() - fine, inserito)
}

/// Da posizione UTF-16 (Java) a indice di carattere (egui). Per l'italiano
/// coincidono; un'emoji come 😀 vale 2 in UTF-16 e 1 in egui.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) fn da_utf16(testo: &str, posizione: usize) -> usize {
    let mut unita = 0;
    for (indice, carattere) in testo.chars().enumerate() {
        if unita >= posizione {
            return indice;
        }
        unita += carattere.len_utf16();
    }
    testo.chars().count()
}

/// Da indice di carattere (egui) a posizione UTF-16 (Java).
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub(crate) fn a_utf16(testo: &str, indice: usize) -> usize {
    testo.chars().take(indice).map(char::len_utf16).sum()
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn differenze_tipiche_della_tastiera() {
        // Una lettera in fondo.
        assert_eq!(differenza("cia", "ciao"), (3, 3, "o".to_owned()));
        // Suggerimento: "citta" diventa "città ".
        assert_eq!(differenza("la citta", "la città "), (7, 8, "à ".to_owned()));
        // Cancella.
        assert_eq!(differenza("ciao", "cia"), (3, 4, String::new()));
        // Lettere uguali vicine: "aa" -> "aaa" aggiunge in fondo.
        assert_eq!(differenza("aa", "aaa"), (2, 2, "a".to_owned()));
        // In mezzo.
        assert_eq!(differenza("Ciao è", "Ciao Xè"), (5, 5, "X".to_owned()));
    }

    #[test]
    fn posizioni_utf16() {
        let testo = "è😀a";
        assert_eq!(a_utf16(testo, 2), 3);
        assert_eq!(da_utf16(testo, 3), 2);
        assert_eq!(da_utf16(testo, 99), 3);
    }
}
