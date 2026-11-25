//! La cartella di lavoro: un archivio di note indipendente scelto dall'utente.
//!
//! All'apertura riconcilia indice e file di testo:
//! - un file modificato con un altro editor (impronta diversa) viene riletto;
//! - un file mancante viene ricreato dalla copia nell'indice;
//! - un file `Nota-<id>.txt` sconosciuto viene adottato come nuova nota.
//!
//! Tiene anche la cronologia delle versioni (vedi `storico`).

use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File, TryLockError};
use std::path::{Path, PathBuf};

use super::file_testo::FileTesto;
use super::indice::{Indice, VERSIONE_INDICE};
use super::sessione::{ImpostazioniRete, Sessione};
use super::storico::{self, Storico, Versione};
use super::{Risultato, errore_io, scrittura_atomica};
use crate::modello::nota::nuovo_id;
use crate::modello::{Nota, tempo};

/// Messaggio restituito quando un'altra istanza sta gia' usando la cartella.
pub const ERRORE_IN_USO: &str = "cartella-in-uso";

const CARTELLA_INTERNA: &str = ".schedario";
const FILE_BLOCCO: &str = ".schedario.lock";

pub struct CartellaLavoro {
    radice: PathBuf,
    file: FileTesto,
    pub note: BTreeMap<String, Nota>,
    impronte: BTreeMap<String, String>,
    id_cartella: String,
    storico: Storico,
    /// Istante dell'ultima versione in cronologia di ciascuna nota, per non
    /// rileggere il file della cronologia a ogni salvataggio.
    ultime_versioni: HashMap<String, Option<i64>>,
    /// Tenuto aperto per tutta la durata: il blocco si rilascia alla chiusura.
    _blocco: File,
}

impl CartellaLavoro {
    pub fn apri(radice: &Path, dispositivo: &str) -> Risultato<Self> {
        fs::create_dir_all(radice).map_err(|e| errore_io("Impossibile creare", radice, e))?;
        let radice = std::path::absolute(radice).unwrap_or_else(|_| radice.to_path_buf());
        let blocco = blocca(&radice)?;
        let file = FileTesto::new(&radice);
        for cartella in [file.cartella_note(), radice.join(CARTELLA_INTERNA)] {
            fs::create_dir_all(&cartella).map_err(|e| errore_io("Impossibile creare", &cartella, e))?;
        }

        let indice: Indice = scrittura_atomica::leggi_json(&radice.join(CARTELLA_INTERNA).join("indice.json"))?
            .unwrap_or_else(|| Indice { versione: VERSIONE_INDICE, id_cartella: nuovo_id(), ..Default::default() });

        let mut cartella = Self {
            storico: Storico::new(&radice.join(CARTELLA_INTERNA)),
            ultime_versioni: HashMap::new(),
            radice,
            file,
            note: BTreeMap::new(),
            impronte: indice.impronte,
            id_cartella: indice.id_cartella,
            _blocco: blocco,
        };
        let da_salvare = cartella.riconcilia(indice.note, dispositivo);
        if !da_salvare.is_empty() {
            cartella.salva(&da_salvare)?;
        }
        Ok(cartella)
    }

    fn riconcilia(&mut self, note_indice: Vec<Nota>, dispositivo: &str) -> Vec<String> {
        let mut da_salvare = Vec::new();
        for mut nota in note_indice {
            if !crate::modello::nota::id_valido(&nota.id) {
                continue;
            }
            match self.file.leggi(&nota.id) {
                // Un file ricomparso per una nota eliminata (es. rimesso a mano
                // da un backup): l'utente lo rivuole, la nota torna in vita.
                Some(letto) if nota.eliminata => {
                    nota.titolo = letto.titolo;
                    nota.testo = letto.testo;
                    nota.nel_cestino = letto.nel_cestino;
                    nota.eliminata = false;
                    nota.segna_modificata(dispositivo);
                    da_salvare.push(nota.id.clone());
                }
                None if nota.eliminata => {}
                Some(letto) => {
                    let modificato_fuori = self.impronte.get(&nota.id) != Some(&letto.impronta);
                    if modificato_fuori && (letto.titolo != nota.titolo || letto.testo != nota.testo) {
                        // La versione di prima resta nella cronologia.
                        let _ = self.storico.aggiungi(&nota, 0, tempo::adesso_ms());
                        nota.titolo = letto.titolo;
                        nota.testo = letto.testo;
                        nota.segna_modificata(dispositivo);
                        da_salvare.push(nota.id.clone());
                    } else if letto.nel_cestino != nota.nel_cestino {
                        // Il file e' nella cartella sbagliata: lo rimettiamo a posto.
                        da_salvare.push(nota.id.clone());
                    }
                }
                None => da_salvare.push(nota.id.clone()),
            }
            self.note.insert(nota.id.clone(), nota);
        }
        for id in self.file.id_presenti() {
            if self.note.contains_key(&id) {
                continue;
            }
            if let Some(letto) = self.file.leggi(&id) {
                let ora = tempo::adesso_ms();
                let mut nota = Nota::nuova(dispositivo);
                nota.id = id.clone();
                nota.titolo = letto.titolo;
                nota.testo = letto.testo;
                nota.creata = ora;
                nota.nel_cestino = letto.nel_cestino;
                self.note.insert(id.clone(), nota);
                da_salvare.push(id);
            }
        }
        da_salvare
    }

    /// Scrive i file di testo delle note indicate e poi l'indice completo.
    /// Aggiorna anche la cronologia (al massimo una versione ogni pochi minuti).
    pub fn salva(&mut self, id: &[String]) -> Risultato<()> {
        for id in id {
            let Some(nota) = self.note.get(id) else { continue };
            if nota.eliminata {
                self.file.rimuovi(id)?;
                self.impronte.remove(id);
                // Eliminata definitivamente: sparisce anche la cronologia.
                let _ = self.storico.rimuovi(id);
                self.ultime_versioni.remove(id);
            } else {
                let impronta = self.file.salva(nota)?;
                self.impronte.insert(id.clone(), impronta);
                self.registra_versione(id, false);
            }
        }
        self.salva_indice()
    }

    /// Mette lo stato attuale della nota nella cronologia. Con `forzata` lo fa
    /// subito (prima che un'altra versione sostituisca il testo); altrimenti
    /// solo se l'ultima versione e' piu' vecchia di qualche minuto.
    /// Un errore della cronologia non blocca mai il salvataggio della nota.
    pub fn registra_versione(&mut self, id: &str, forzata: bool) {
        let Some(nota) = self.note.get(id).filter(|n| !n.eliminata) else { return };
        let intervallo = if forzata { 0 } else { storico::INTERVALLO_SCRITTURA_MS };
        if !forzata {
            if let Some(Some(ultima)) = self.ultime_versioni.get(id) {
                if nota.modificata - ultima < intervallo {
                    return;
                }
            }
        }
        if let Ok(ultima) = self.storico.aggiungi(nota, intervallo, tempo::adesso_ms()) {
            self.ultime_versioni.insert(id.to_owned(), ultima);
        }
    }

    /// Versioni precedenti della nota, dalla piu' vecchia alla piu' recente.
    pub fn cronologia(&self, id: &str) -> Vec<Versione> {
        self.storico.leggi(id)
    }

    fn salva_indice(&self) -> Risultato<()> {
        let indice = Indice {
            versione: VERSIONE_INDICE,
            id_cartella: self.id_cartella.clone(),
            note: self.note.values().cloned().collect(),
            impronte: self.impronte.clone(),
        };
        scrittura_atomica::scrivi_json(&self.percorso_interno("indice.json"), &indice)
    }

    pub fn radice(&self) -> &Path {
        &self.radice
    }

    pub fn cartella_note(&self) -> PathBuf {
        self.file.cartella_note()
    }

    fn percorso_interno(&self, nome: &str) -> PathBuf {
        self.radice.join(CARTELLA_INTERNA).join(nome)
    }

    /// Note attive nell'ordine delle schede: prima le fissate, poi le altre,
    /// ciascun gruppo per data di creazione.
    pub fn note_attive(&self) -> Vec<&Nota> {
        let mut note: Vec<_> = self.note.values().filter(|n| n.visibile() && !n.nel_cestino).collect();
        note.sort_by(|a, b| (!a.fissata, a.creata, &a.id).cmp(&(!b.fissata, b.creata, &b.id)));
        note
    }

    /// Note nel cestino, dalla piu' recente.
    pub fn note_cestino(&self) -> Vec<&Nota> {
        let mut note: Vec<_> = self.note.values().filter(|n| n.visibile() && n.nel_cestino).collect();
        note.sort_by(|a, b| b.modificata.cmp(&a.modificata));
        note
    }

    pub fn carica_sessione(&self) -> Sessione {
        scrittura_atomica::leggi_json(&self.percorso_interno("sessione.json"))
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    pub fn salva_sessione(&self, sessione: &Sessione) -> Risultato<()> {
        scrittura_atomica::scrivi_json(&self.percorso_interno("sessione.json"), sessione)
    }

    pub fn carica_rete(&self) -> ImpostazioniRete {
        scrittura_atomica::leggi_json(&self.percorso_interno("rete.json"))
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    pub fn salva_rete(&self, rete: &ImpostazioniRete) -> Risultato<()> {
        scrittura_atomica::scrivi_json(&self.percorso_interno("rete.json"), rete)
    }

    /// File interni che un'esportazione non deve mai sovrascrivere.
    pub fn percorsi_protetti(&self) -> Vec<PathBuf> {
        ["indice.json", "sessione.json", "rete.json"]
            .iter()
            .map(|n| self.percorso_interno(n))
            .chain([self.radice.join(FILE_BLOCCO)])
            .collect()
    }
}

fn blocca(radice: &Path) -> Risultato<File> {
    let percorso = radice.join(FILE_BLOCCO);
    let file = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&percorso)
        .map_err(|e| errore_io("Impossibile aprire", &percorso, e))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(ERRORE_IN_USO.to_owned()),
        Err(TryLockError::Error(e)) => Err(errore_io("Impossibile bloccare", &percorso, e)),
    }
}
