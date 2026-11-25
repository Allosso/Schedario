//! Scrittura "tutto o niente": prima un file temporaneo, poi la sostituzione.
//! Se il programma si interrompe a meta', il file originale resta intatto.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use super::{Risultato, errore_io};

pub fn scrivi(percorso: &Path, contenuto: &[u8]) -> Risultato<()> {
    let cartella = percorso
        .parent()
        .ok_or_else(|| format!("percorso non valido: {}", percorso.display()))?;
    fs::create_dir_all(cartella).map_err(|e| errore_io("Impossibile creare", cartella, e))?;

    let nome = percorso
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temporaneo = cartella.join(format!(".{nome}.{}.tmp", crate::modello::nota::nuovo_id()));

    let esito = (|| -> std::io::Result<()> {
        let mut file = File::create(&temporaneo)?;
        file.write_all(contenuto)?;
        file.sync_all()?;
        drop(file);
        // Su Windows `rename` sostituisce il file esistente (MoveFileEx).
        fs::rename(&temporaneo, percorso)
    })();

    if let Err(errore) = esito {
        let _ = fs::remove_file(&temporaneo);
        return Err(errore_io("Impossibile scrivere", percorso, errore));
    }
    Ok(())
}

/// Serializza in JSON leggibile e scrive in modo atomico.
pub fn scrivi_json<T: serde::Serialize>(percorso: &Path, valore: &T) -> Risultato<()> {
    let testo = serde_json::to_string_pretty(valore)
        .map_err(|e| format!("Impossibile preparare {}: {e}", percorso.display()))?;
    scrivi(percorso, testo.as_bytes())
}

/// Legge un JSON; restituisce `None` se il file non esiste. Accetta il BOM
/// UTF-8 che alcuni editor (e PowerShell) aggiungono all'inizio del file.
pub fn leggi_json<T: serde::de::DeserializeOwned>(percorso: &Path) -> Risultato<Option<T>> {
    match fs::read_to_string(percorso) {
        Ok(testo) => serde_json::from_str(senza_bom(&testo))
            .map(Some)
            .map_err(|e| format!("File danneggiato {}: {e}", percorso.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(errore_io("Impossibile leggere", percorso, e)),
    }
}

pub fn senza_bom(testo: &str) -> &str {
    testo.strip_prefix('\u{feff}').unwrap_or(testo)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn legge_un_json_con_bom() {
        let cartella = std::env::temp_dir().join(format!("schedario-bom-{}", crate::modello::nota::nuovo_id()));
        let percorso = cartella.join("prova.json");
        scrivi(&percorso, "\u{feff}{\"a\": 1}".as_bytes()).unwrap();
        let valore: Option<std::collections::HashMap<String, i32>> = leggi_json(&percorso).unwrap();
        assert_eq!(valore.unwrap()["a"], 1);
        let _ = fs::remove_dir_all(&cartella);
    }
}
