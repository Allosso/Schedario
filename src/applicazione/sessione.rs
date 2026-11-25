//! Widget aperti e stato dell'interfaccia da ricordare tra un avvio e l'altro.

use super::stato::{Applicazione, WidgetAperto};
use crate::archivio::{Sessione, StatoWidget};

/// Dimensioni minime e iniziali di un widget, in punti.
pub const LARGHEZZA_MINIMA_WIDGET: f32 = 260.0;
pub const ALTEZZA_MINIMA_WIDGET: f32 = 200.0;
pub const ALTEZZA_WIDGET_RIDOTTO: f32 = 46.0;

impl Applicazione {
    pub fn apri_widget(&mut self, id: &str) {
        let Some(nota) = self.nota(id) else { return };
        if nota.nel_cestino || self.widget.iter().any(|w| w.stato.id_nota == id) {
            return;
        }
        // Ogni nuovo widget appare leggermente spostato rispetto al precedente.
        let scarto = 28.0 * (self.widget.len() % 8) as f32;
        let stato = StatoWidget {
            id_nota: id.to_owned(),
            x: 90.0 + scarto,
            y: 90.0 + scarto,
            larghezza: 340.0,
            altezza: 420.0,
            in_primo_piano: true,
            ridotto: false,
        };
        self.aggiungi_widget(stato);
    }

    fn aggiungi_widget(&mut self, mut stato: StatoWidget) {
        stato.larghezza = stato.larghezza.max(LARGHEZZA_MINIMA_WIDGET);
        stato.altezza = stato.altezza.max(ALTEZZA_MINIMA_WIDGET);
        // Una finestra fuori schermo (monitor scollegato) torna visibile.
        stato.x = stato.x.clamp(-stato.larghezza + 80.0, 4000.0);
        stato.y = stato.y.clamp(0.0, 3000.0);
        let altezza_iniziale = if stato.ridotto { ALTEZZA_WIDGET_RIDOTTO } else { stato.altezza };
        self.widget.push(WidgetAperto {
            altezza_espansa: stato.altezza,
            geometria_iniziale: [stato.x, stato.y, stato.larghezza, altezza_iniziale],
            stato,
            fuoco_testo: 0,
        });
    }

    /// Chiude il widget (la nota resta). Se era l'ultimo e la finestra
    /// principale era nascosta, la finestra ricompare.
    pub fn chiudi_widget(&mut self, id: &str) {
        self.widget.retain(|w| w.stato.id_nota != id);
        if self.widget.is_empty() {
            self.finestra_visibile = true;
        }
    }

    pub fn mostra_finestra(&mut self) {
        self.finestra_visibile = true;
    }

    /// Apre la finestra principale sulla nota indicata.
    pub fn mostra_nota(&mut self, id: &str) {
        self.interfaccia.ricerca.clear();
        self.interfaccia.selezionata = Some(id.to_owned());
        self.mostra_finestra();
    }

    pub fn salva_sessione(&mut self) {
        let Some(cartella) = &self.cartella else { return };
        let sessione = Sessione {
            nota_selezionata: self.interfaccia.selezionata.clone(),
            widget: self
                .widget
                .iter()
                .map(|w| {
                    let mut stato = w.stato.clone();
                    if stato.ridotto {
                        stato.altezza = w.altezza_espansa;
                    }
                    stato
                })
                .collect(),
            sezioni: self.interfaccia.sezioni.clone(),
            finestra_visibile: self.finestra_visibile,
            ordine_schede: self.interfaccia.ordine_schede.clone(),
        };
        if let Err(errore) = cartella.salva_sessione(&sessione) {
            self.avviso = Some(super::Avviso::Errore(errore));
        }
    }

    pub(super) fn ripristina_sessione(&mut self) {
        let Some(cartella) = &self.cartella else { return };
        let sessione = cartella.carica_sessione();
        let valida = |id: &String| cartella.note.get(id).is_some_and(|n| !n.nel_cestino);
        self.interfaccia.selezionata = sessione.nota_selezionata.filter(valida);
        self.interfaccia.sezioni = sessione.sezioni;
        self.interfaccia.ordine_schede = sessione.ordine_schede;
        let widget: Vec<StatoWidget> = sessione.widget.into_iter().filter(|w| valida(&w.id_nota)).collect();
        for stato in widget {
            self.aggiungi_widget(stato);
        }
        if self.interfaccia.selezionata.is_none() {
            self.interfaccia.selezionata = self.note_attive_id().into_iter().next();
        }
        self.finestra_visibile = sessione.finestra_visibile || self.widget.is_empty();
    }
}
