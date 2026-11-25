//! Cronologia delle versioni: `.schedario/storico/<id>.json`.
//!
//! Ogni file contiene le versioni precedenti di una nota, dalla piu' vecchia.
//! Una versione si aggiunge:
//! - mentre si scrive, al massimo una ogni `INTERVALLO_SCRITTURA`;
//! - sempre, prima che un'altra versione sostituisca il testo (modifica
//!   arrivata dalla rete, ripristino di una versione precedente).
//!
//! Si tengono le ultime `MASSIMO_VERSIONI` versioni e, tra queste, si buttano
//! quelle piu' vecchie di `DURATA_MASSIMA_MS`, salvo le ultime `MINIMO_VERSIONI`.
//! La cronologia resta su questo computer: non viene sincronizzata.
//! Un'eliminazione definitiva cancella anche la cronologia della nota.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{Risultato, errore_io, scrittura_atomica};
use crate::modello::Nota;
use crate::modello::nota::id_valido;

pub const INTERVALLO_SCRITTURA_MS: i64 = 5 * 60 * 1000;
pub const MASSIMO_VERSIONI: usize = 50;
pub const MINIMO_VERSIONI: usize = 10;
pub const DURATA_MASSIMA_MS: i64 = 30 * 24 * 60 * 60 * 1000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Versione {
    /// Istante della versione (la data di modifica della nota in quel momento).
    pub quando: i64,
    pub titolo: String,
    pub testo: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct FileStorico {
    versioni: Vec<Versione>,
}

pub struct Storico {
    cartella: PathBuf,
}

impl Storico {
    pub fn new(cartella_interna: &Path) -> Self {
        Self { cartella: cartella_interna.join("storico") }
    }

    fn percorso(&self, id: &str) -> Option<PathBuf> {
        id_valido(id).then(|| self.cartella.join(format!("{id}.json")))
    }

    /// Versioni salvate della nota, dalla piu' vecchia alla piu' recente.
    pub fn leggi(&self, id: &str) -> Vec<Versione> {
        let Some(percorso) = self.percorso(id) else { return Vec::new() };
        scrittura_atomica::leggi_json::<FileStorico>(&percorso)
            .ok()
            .flatten()
            .map(|f| f.versioni)
            .unwrap_or_default()
    }

    /// Aggiunge lo stato attuale della nota, se diverso dall'ultima versione e
    /// (con `intervallo_ms` > 0) se l'ultima e' piu' vecchia dell'intervallo.
    /// `adesso` e' l'istante attuale in millisecondi.
    ///
    /// Restituisce l'istante dell'ultima versione salvata dopo la chiamata (o
    /// `None` se non ce ne sono): il chiamante lo ricorda per non rileggere il
    /// file a ogni salvataggio.
    pub fn aggiungi(&self, nota: &Nota, intervallo_ms: i64, adesso: i64) -> Risultato<Option<i64>> {
        let Some(percorso) = self.percorso(&nota.id) else { return Ok(None) };
        let mut versioni = self.leggi(&nota.id);
        let ultima = versioni.last().map(|v| v.quando);
        if nota.eliminata || (nota.titolo.trim().is_empty() && nota.testo.trim().is_empty()) {
            return Ok(ultima);
        }
        if let Some(precedente) = versioni.last() {
            let uguale = precedente.titolo == nota.titolo && precedente.testo == nota.testo;
            if uguale || (intervallo_ms > 0 && nota.modificata - precedente.quando < intervallo_ms) {
                return Ok(ultima);
            }
        }
        versioni.push(Versione { quando: nota.modificata, titolo: nota.titolo.clone(), testo: nota.testo.clone() });
        sfoltisci(&mut versioni, adesso);
        scrittura_atomica::scrivi_json(&percorso, &FileStorico { versioni })?;
        Ok(Some(nota.modificata))
    }

    pub fn rimuovi(&self, id: &str) -> Risultato<()> {
        let Some(percorso) = self.percorso(id) else { return Ok(()) };
        match fs::remove_file(&percorso) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(errore_io("Impossibile eliminare", &percorso, e)),
        }
    }
}

/// Tiene le ultime `MASSIMO_VERSIONI` e toglie quelle troppo vecchie, ma
/// lascia sempre almeno le ultime `MINIMO_VERSIONI`.
fn sfoltisci(versioni: &mut Vec<Versione>, adesso: i64) {
    if versioni.len() > MASSIMO_VERSIONI {
        versioni.drain(..versioni.len() - MASSIMO_VERSIONI);
    }
    let limite = adesso - DURATA_MASSIMA_MS;
    while versioni.len() > MINIMO_VERSIONI && versioni.first().is_some_and(|v| v.quando < limite) {
        versioni.remove(0);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn cartella() -> PathBuf {
        std::env::temp_dir().join(format!("schedario-storico-{}", crate::modello::nota::nuovo_id()))
    }

    #[test]
    fn aggiunge_solo_versioni_nuove_e_rispetta_l_intervallo() {
        let radice = cartella();
        let storico = Storico::new(&radice);
        let mut nota = Nota::nuova("a");
        let quante = |s: &Storico, id: &str| s.leggi(id).len();
        assert_eq!(storico.aggiungi(&nota, 0, nota.modificata).unwrap(), None, "nota vuota");
        nota.testo = "uno".into();
        let prima = nota.modificata;
        assert_eq!(storico.aggiungi(&nota, INTERVALLO_SCRITTURA_MS, prima).unwrap(), Some(prima));
        storico.aggiungi(&nota, 0, prima).unwrap();
        assert_eq!(quante(&storico, &nota.id), 1, "stesso testo: nessuna versione nuova");

        nota.testo = "due".into();
        nota.modificata += 1000;
        assert_eq!(storico.aggiungi(&nota, INTERVALLO_SCRITTURA_MS, nota.modificata).unwrap(), Some(prima));
        assert_eq!(quante(&storico, &nota.id), 1, "troppo presto");
        assert_eq!(storico.aggiungi(&nota, 0, nota.modificata).unwrap(), Some(nota.modificata), "forzata");
        let versioni = storico.leggi(&nota.id);
        assert_eq!(versioni.iter().map(|v| v.testo.as_str()).collect::<Vec<_>>(), ["uno", "due"]);

        storico.rimuovi(&nota.id).unwrap();
        assert!(storico.leggi(&nota.id).is_empty());
        let _ = fs::remove_dir_all(&radice);
    }

    #[test]
    fn sfoltisce_le_versioni_vecchie() {
        let adesso = 100 * DURATA_MASSIMA_MS;
        let versione = |quando| Versione { quando, titolo: String::new(), testo: quando.to_string() };
        let mut versioni: Vec<_> = (0..60).map(|i| versione(i)).collect();
        sfoltisci(&mut versioni, adesso);
        assert_eq!(versioni.len(), MINIMO_VERSIONI, "tutte vecchie: restano le ultime");
        assert_eq!(versioni.last().unwrap().quando, 59);

        let mut recenti: Vec<_> = (0..60).map(|i| versione(adesso - i)).rev().collect();
        sfoltisci(&mut recenti, adesso);
        assert_eq!(recenti.len(), MASSIMO_VERSIONI);
    }
}
