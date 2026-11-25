//! Scorciatoia da tastiera globale per la nota rapida, da qualsiasi programma
//! e anche a Schedario nascosto. La combinazione si sceglie nelle impostazioni
//! (predefinita Ctrl+Alt+N, Ctrl+Option+N su macOS).
//!
//! Usa `global-hotkey` (Apache-2.0 OR MIT). Funziona su Windows, macOS e
//! Linux con X11. Su Wayland i programmi non possono registrare tasti globali:
//! li' la creazione fallisce e il programma lo dice nelle impostazioni (si
//! puo' associare la scorciatoia a Schedario dalle impostazioni del desktop).
//!
//! Va creata sul thread della finestra (lo richiedono Windows e macOS).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, Once};

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

use super::area_notifica::Ridisegna;
use crate::archivio::preferenze::Combinazione;

// Il gestore degli eventi della libreria si installa una sola volta per tutto
// il programma: comunica con l'interfaccia attraverso questi valori.
static PREMUTA: AtomicBool = AtomicBool::new(false);
static ID_TASTO: AtomicU32 = AtomicU32::new(0);
static RIDISEGNA: Mutex<Option<Ridisegna>> = Mutex::new(None);
static INSTALLA: Once = Once::new();

pub struct ScorciatoiaGlobale {
    gestore: GlobalHotKeyManager,
    tasto: HotKey,
}

impl ScorciatoiaGlobale {
    pub fn crea(ridisegna: Ridisegna, combinazione: &Combinazione) -> Result<Self, String> {
        let tasto = tasto_di(combinazione).ok_or_else(|| format!("tasto non supportato: {}", combinazione.tasto))?;
        let gestore = GlobalHotKeyManager::new().map_err(|e| e.to_string())?;
        gestore.register(tasto).map_err(|e| e.to_string())?;
        ID_TASTO.store(tasto.id(), Ordering::SeqCst);
        if let Ok(mut r) = RIDISEGNA.lock() {
            *r = Some(ridisegna);
        }
        INSTALLA.call_once(|| {
            GlobalHotKeyEvent::set_event_handler(Some(|evento: GlobalHotKeyEvent| {
                if evento.state == HotKeyState::Pressed && evento.id == ID_TASTO.load(Ordering::SeqCst) {
                    PREMUTA.store(true, Ordering::SeqCst);
                    if let Ok(r) = RIDISEGNA.lock() {
                        if let Some(r) = r.as_ref() {
                            r();
                        }
                    }
                }
            }));
        });
        Ok(Self { gestore, tasto })
    }

    /// `true` una volta per ogni pressione della scorciatoia.
    pub fn premuta(&self) -> bool {
        PREMUTA.swap(false, Ordering::SeqCst)
    }
}

impl Drop for ScorciatoiaGlobale {
    fn drop(&mut self) {
        let _ = self.gestore.unregister(self.tasto);
        PREMUTA.store(false, Ordering::SeqCst);
    }
}

/// `true` se il tasto (nome egui) si puo' usare in una scorciatoia globale:
/// lettere, cifre, F1–F12 e spazio.
pub fn tasto_supportato(tasto: &str) -> bool {
    codice(tasto).is_some()
}

fn tasto_di(c: &Combinazione) -> Option<HotKey> {
    let mut modificatori = Modifiers::empty();
    for (attivo, m) in [(c.ctrl, Modifiers::CONTROL), (c.alt, Modifiers::ALT), (c.maiuscolo, Modifiers::SHIFT), (c.comando, Modifiers::META)] {
        if attivo {
            modificatori |= m;
        }
    }
    Some(HotKey::new(Some(modificatori), codice(&c.tasto)?))
}

fn codice(tasto: &str) -> Option<Code> {
    use Code::*;
    const LETTERE: [Code; 26] = [
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR,
        KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    ];
    const CIFRE: [Code; 10] = [Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9];
    const FUNZIONE: [Code; 12] = [F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12];
    let mut caratteri = tasto.chars();
    match (caratteri.next(), caratteri.next()) {
        (Some(c @ 'A'..='Z'), None) => return Some(LETTERE[(c as u8 - b'A') as usize]),
        (Some(c @ '0'..='9'), None) => return Some(CIFRE[(c as u8 - b'0') as usize]),
        _ => {}
    }
    if tasto == "Space" {
        return Some(Space);
    }
    let numero: usize = tasto.strip_prefix('F')?.parse().ok()?;
    FUNZIONE.get(numero.checked_sub(1)?).copied()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn riconosce_i_tasti_ammessi() {
        assert_eq!(codice("N"), Some(Code::KeyN));
        assert_eq!(codice("7"), Some(Code::Digit7));
        assert_eq!(codice("F12"), Some(Code::F12));
        assert_eq!(codice("Space"), Some(Code::Space));
        assert_eq!(codice("F13"), None);
        assert_eq!(codice("F0"), None);
        assert_eq!(codice("Enter"), None);
        assert_eq!(codice("n"), None);
    }

    #[test]
    fn la_combinazione_predefinita_e_ctrl_alt_n() {
        let c = Combinazione::default();
        assert!(c.valida());
        let attesa = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyN);
        assert_eq!(tasto_di(&c), Some(attesa));
        let solo_shift = Combinazione { ctrl: false, alt: false, maiuscolo: true, ..c };
        assert!(!solo_shift.valida());
    }
}
