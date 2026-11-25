//! I file `.txt` delle note: uno per nota, con nome stabile `Nota-<id>.txt`.
//!
//! Formato del file: titolo sulla prima riga, una riga vuota, poi il testo.
//! Cosi' le note restano leggibili e modificabili anche senza il programma.

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::{Risultato, errore_io, scrittura_atomica};
use crate::modello::{Nota, nota::id_valido};

pub const CARTELLA_NOTE: &str = "Note";
pub const CARTELLA_CESTINO: &str = "Cestino";

pub struct FileTesto {
    radice: PathBuf,
}

impl FileTesto {
    pub fn new(radice: &Path) -> Self {
        Self { radice: radice.to_path_buf() }
    }

    pub fn cartella_note(&self) -> PathBuf {
        self.radice.join(CARTELLA_NOTE)
    }

    pub fn cartella_cestino(&self) -> PathBuf {
        self.radice.join(CARTELLA_CESTINO)
    }

    /// Percorso dove la nota deve stare, in base al cestino.
    pub fn percorso(&self, id: &str, nel_cestino: bool) -> PathBuf {
        let cartella = if nel_cestino { self.cartella_cestino() } else { self.cartella_note() };
        cartella.join(nome_file(id))
    }

    /// Scrive il file nella cartella giusta e rimuove l'eventuale copia
    /// rimasta nell'altra (per esempio dopo il ripristino dal cestino).
    /// Restituisce l'impronta del contenuto scritto.
    pub fn salva(&self, nota: &Nota) -> Risultato<String> {
        let contenuto = componi(&nota.titolo, &nota.testo);
        scrittura_atomica::scrivi(&self.percorso(&nota.id, nota.nel_cestino), contenuto.as_bytes())?;
        let vecchio = self.percorso(&nota.id, !nota.nel_cestino);
        if vecchio.exists() {
            fs::remove_file(&vecchio).map_err(|e| errore_io("Impossibile spostare", &vecchio, e))?;
        }
        Ok(impronta(&contenuto))
    }

    /// Elimina il file della nota da entrambe le cartelle (eliminazione definitiva).
    pub fn rimuovi(&self, id: &str) -> Risultato<()> {
        for nel_cestino in [false, true] {
            let percorso = self.percorso(id, nel_cestino);
            if percorso.exists() {
                fs::remove_file(&percorso).map_err(|e| errore_io("Impossibile eliminare", &percorso, e))?;
            }
        }
        Ok(())
    }

    /// Legge il file di una nota, cercandolo in entrambe le cartelle.
    /// Restituisce titolo, testo, impronta e se si trovava nel cestino.
    pub fn leggi(&self, id: &str) -> Option<FileLetto> {
        [false, true].into_iter().find_map(|nel_cestino| {
            let testo = fs::read(self.percorso(id, nel_cestino)).ok()?;
            let testo = String::from_utf8_lossy(&testo).into_owned();
            let (titolo, corpo) = scomponi(&testo);
            Some(FileLetto { titolo, testo: corpo, impronta: impronta(&testo), nel_cestino })
        })
    }

    /// Identificativi di tutti i file gestiti (`Nota-<id>.txt`) presenti.
    pub fn id_presenti(&self) -> Vec<String> {
        let mut id = Vec::new();
        for cartella in [self.cartella_note(), self.cartella_cestino()] {
            let Ok(voci) = fs::read_dir(&cartella) else { continue };
            for voce in voci.flatten() {
                let nome = voce.file_name().to_string_lossy().into_owned();
                if let Some(trovato) = id_da_nome(&nome) {
                    if !id.contains(&trovato) {
                        id.push(trovato);
                    }
                }
            }
        }
        id
    }
}

pub struct FileLetto {
    pub titolo: String,
    pub testo: String,
    pub impronta: String,
    pub nel_cestino: bool,
}

pub fn nome_file(id: &str) -> String {
    format!("Nota-{id}.txt")
}

fn id_da_nome(nome: &str) -> Option<String> {
    let id = nome.strip_prefix("Nota-")?.strip_suffix(".txt")?;
    id_valido(id).then(|| id.to_owned())
}

pub fn componi(titolo: &str, testo: &str) -> String {
    // Il titolo e' una sola riga: eventuali a capo diventano spazi.
    let titolo = titolo.replace(['\r', '\n'], " ");
    format!("{titolo}\n\n{testo}")
}

pub fn scomponi(contenuto: &str) -> (String, String) {
    let contenuto = contenuto.strip_prefix('\u{feff}').unwrap_or(contenuto);
    let contenuto = contenuto.replace("\r\n", "\n");
    match contenuto.split_once('\n') {
        None => (contenuto.trim_end().to_owned(), String::new()),
        Some((titolo, resto)) => {
            let corpo = resto.strip_prefix('\n').unwrap_or(resto);
            (titolo.trim_end().to_owned(), corpo.to_owned())
        }
    }
}

/// Impronta SHA-256 abbreviata: rivela le modifiche fatte da altri editor.
pub fn impronta(contenuto: &str) -> String {
    let hash = Sha256::digest(contenuto.as_bytes());
    hash.iter().take(16).map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn componi_e_scomponi_sono_inversi() {
        let testo = componi("Spesa", "latte\npane\n");
        assert_eq!(scomponi(&testo), ("Spesa".into(), "latte\npane\n".into()));
        assert_eq!(scomponi("solo titolo"), ("solo titolo".into(), String::new()));
        assert_eq!(scomponi("T\r\n\r\ncorpo"), ("T".into(), "corpo".into()));
    }

    #[test]
    fn riconosce_solo_i_nomi_gestiti() {
        let id = crate::modello::nota::nuovo_id();
        assert_eq!(id_da_nome(&nome_file(&id)), Some(id));
        assert_eq!(id_da_nome("appunti.txt"), None);
        assert_eq!(id_da_nome("Nota-xyz.txt"), None);
    }
}
