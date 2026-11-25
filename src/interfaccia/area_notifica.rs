//! Icona nell'area di notifica (vicino all'orologio) con un piccolo menu:
//! Mostra Schedario, Nuova nota, Esci. Un clic sull'icona mostra la finestra.
//!
//! Serve soprattutto quando sono aperti solo i widget: la finestra principale
//! e' nascosta e i widget non compaiono nella barra delle applicazioni.
//!
//! - Windows e macOS: libreria `tray-icon`;
//! - Linux: libreria `ksni` (protocollo StatusNotifierItem). Su GNOME serve
//!   l'estensione AppIndicator, gia' presente in Ubuntu.
//!
//! Se il sistema non offre un'area di notifica, `crea` restituisce `None` e il
//! programma funziona come prima.

use std::sync::{Arc, mpsc};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Comando {
    Mostra,
    NuovaNota,
    Esci,
}

pub struct Testi {
    pub mostra: String,
    pub nuova: String,
    pub esci: String,
    pub suggerimento: String,
}

pub type Ridisegna = Arc<dyn Fn() + Send + Sync>;

pub struct AreaNotifica {
    comandi: mpsc::Receiver<Comando>,
    _icona: sistema::Icona,
}

impl AreaNotifica {
    pub fn crea(testi: Testi, ridisegna: Ridisegna) -> Option<Self> {
        let (mittente, comandi) = mpsc::channel();
        let invia: Arc<dyn Fn(Comando) + Send + Sync> = Arc::new(move |comando| {
            let _ = mittente.send(comando);
            ridisegna();
        });
        let icona = super::icona::icona();
        let _icona = sistema::crea(&testi, &icona.rgba, icona.width, icona.height, invia)?;
        Some(Self { comandi, _icona })
    }

    /// Comandi scelti dall'utente dall'ultima chiamata.
    pub fn comandi(&self) -> Vec<Comando> {
        self.comandi.try_iter().collect()
    }
}

type Invia = Arc<dyn Fn(Comando) + Send + Sync>;

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod sistema {
    use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

    use super::{Comando, Invia, Testi};

    pub struct Icona {
        _icona: TrayIcon,
    }

    pub fn crea(testi: &Testi, rgba: &[u8], larghezza: u32, altezza: u32, invia: Invia) -> Option<Icona> {
        let mostra = MenuItem::new(&testi.mostra, true, None);
        let nuova = MenuItem::new(&testi.nuova, true, None);
        let esci = MenuItem::new(&testi.esci, true, None);
        let menu = Menu::new();
        menu.append(&mostra).ok()?;
        menu.append(&nuova).ok()?;
        menu.append(&PredefinedMenuItem::separator()).ok()?;
        menu.append(&esci).ok()?;

        let id = (mostra.id().clone(), nuova.id().clone(), esci.id().clone());
        let invia_menu = invia.clone();
        MenuEvent::set_event_handler(Some(move |evento: MenuEvent| {
            let comando = if evento.id == id.0 {
                Comando::Mostra
            } else if evento.id == id.1 {
                Comando::NuovaNota
            } else if evento.id == id.2 {
                Comando::Esci
            } else {
                return;
            };
            invia_menu(comando);
        }));
        TrayIconEvent::set_event_handler(Some(move |evento: TrayIconEvent| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = evento {
                invia(Comando::Mostra);
            }
        }));

        let immagine = tray_icon::Icon::from_rgba(rgba.to_vec(), larghezza, altezza).ok()?;
        let icona = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip(&testi.suggerimento)
            .with_icon(immagine)
            .build()
            .ok()?;
        Some(Icona { _icona: icona })
    }
}

#[cfg(target_os = "linux")]
mod sistema {
    use ksni::blocking::{Handle, TrayMethods};
    use ksni::menu::StandardItem;

    use super::{Comando, Invia, Testi};

    struct Vassoio {
        mostra: String,
        nuova: String,
        esci: String,
        suggerimento: String,
        immagine: ksni::Icon,
        invia: Invia,
    }

    impl ksni::Tray for Vassoio {
        fn id(&self) -> String {
            "schedario".into()
        }
        fn title(&self) -> String {
            crate::NOME_APP.into()
        }
        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            vec![ksni::Icon {
                width: self.immagine.width,
                height: self.immagine.height,
                data: self.immagine.data.clone(),
            }]
        }
        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip { title: self.suggerimento.clone(), ..Default::default() }
        }
        fn activate(&mut self, _x: i32, _y: i32) {
            (self.invia)(Comando::Mostra);
        }
        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            let voce = |etichetta: &str, comando: Comando| -> ksni::MenuItem<Self> {
                StandardItem {
                    label: etichetta.to_owned(),
                    activate: Box::new(move |vassoio: &mut Self| (vassoio.invia)(comando)),
                    ..Default::default()
                }
                .into()
            };
            vec![
                voce(&self.mostra, Comando::Mostra),
                voce(&self.nuova, Comando::NuovaNota),
                ksni::MenuItem::Separator,
                voce(&self.esci, Comando::Esci),
            ]
        }
    }

    pub struct Icona {
        gestore: Handle<Vassoio>,
    }

    impl Drop for Icona {
        fn drop(&mut self) {
            self.gestore.shutdown().wait();
        }
    }

    pub fn crea(testi: &Testi, rgba: &[u8], larghezza: u32, altezza: u32, invia: Invia) -> Option<Icona> {
        // ksni vuole i pixel in formato ARGB (ordine di rete).
        let data = rgba.chunks_exact(4).flat_map(|p| [p[3], p[0], p[1], p[2]]).collect();
        let vassoio = Vassoio {
            mostra: testi.mostra.clone(),
            nuova: testi.nuova.clone(),
            esci: testi.esci.clone(),
            suggerimento: testi.suggerimento.clone(),
            immagine: ksni::Icon { width: larghezza as i32, height: altezza as i32, data },
            invia,
        };
        vassoio.spawn().ok().map(|gestore| Icona { gestore })
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod sistema {
    use super::{Invia, Testi};

    pub struct Icona;

    pub fn crea(_: &Testi, _: &[u8], _: u32, _: u32, _: Invia) -> Option<Icona> {
        None
    }
}
