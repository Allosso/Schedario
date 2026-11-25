//! Esportazione e importazione di backup JSON e di singole note in `.txt`.
//!
//! L'importazione aggiunge copie con nuovi identificativi e non sovrascrive
//! mai le note esistenti. Accetta anche i backup del prototipo "Notte"
//! (campi `title`, `body`, `deleted`).

use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use super::{Risultato, errore_io, file_testo, scrittura_atomica};
use crate::modello::{Nota, tempo};

#[derive(Serialize)]
struct Backup<'a> {
    formato: &'static str,
    versione: u32,
    esportato: i64,
    note: Vec<&'a Nota>,
}

pub fn esporta(percorso: &Path, note: Vec<&Nota>) -> Risultato<()> {
    let note = note.into_iter().filter(|n| n.visibile()).collect();
    let backup = Backup { formato: "schedario", versione: 1, esportato: tempo::adesso_ms(), note };
    scrittura_atomica::scrivi_json(percorso, &backup)
}

pub fn importa(percorso: &Path, dispositivo: &str) -> Risultato<Vec<Nota>> {
    let testo = std::fs::read_to_string(percorso).map_err(|e| errore_io("Impossibile leggere", percorso, e))?;
    let valore: Value =
        serde_json::from_str(super::scrittura_atomica::senza_bom(&testo)).map_err(|e| format!("Il file non e' un backup valido: {e}"))?;
    let elenco = valore
        .get("note")
        .or_else(|| valore.get("notes"))
        .and_then(Value::as_array)
        .ok_or("Il file non contiene note.")?;

    let testo_di = |voce: &Value, chiavi: [&str; 2]| -> String {
        chiavi
            .iter()
            .find_map(|k| voce.get(*k).and_then(Value::as_str))
            .unwrap_or_default()
            .to_owned()
    };

    let mut importate = Vec::new();
    for (posizione, voce) in elenco.iter().enumerate() {
        if !voce.is_object() || voce.get("eliminata").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let mut nota = Nota::nuova(dispositivo);
        nota.titolo = testo_di(voce, ["titolo", "title"]);
        nota.testo = testo_di(voce, ["testo", "body"]);
        nota.nel_cestino = ["nel_cestino", "deleted"]
            .iter()
            .find_map(|k| voce.get(*k).and_then(Value::as_bool))
            .unwrap_or(false);
        let colore = testo_di(voce, ["colore", "color"]);
        if crate::modello::nota::colore_valido(&colore) {
            nota.colore = colore;
        }
        // Mantiene l'ordine originale delle schede.
        nota.creata += posizione as i64;
        importate.push(nota);
    }
    Ok(importate)
}

pub fn esporta_testo(percorso: &Path, nota: &Nota) -> Risultato<()> {
    scrittura_atomica::scrivi(percorso, file_testo::componi(&nota.titolo, &nota.testo).as_bytes())
}

/// Nome di file sicuro ricavato dal titolo, per i dialoghi di salvataggio.
pub fn nome_sicuro(titolo: &str, predefinito: &str) -> String {
    let pulito: String = titolo
        .chars()
        .map(|c| if c.is_alphanumeric() || " -_".contains(c) { c } else { '_' })
        .collect::<String>()
        .trim()
        .chars()
        .take(60)
        .collect();
    if pulito.is_empty() { predefinito.to_owned() } else { pulito }
}
