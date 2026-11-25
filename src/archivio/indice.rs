//! L'indice interno `.schedario/indice.json`.
//!
//! Contiene per ogni nota i metadati (date, cestino, dispositivo) e una copia
//! completa del contenuto, usata per ricreare un file `.txt` cancellato per
//! errore. Le impronte permettono di accorgersi se un file e' stato modificato
//! con un altro editor mentre il programma era chiuso.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::modello::Nota;

pub const VERSIONE_INDICE: u32 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Indice {
    pub versione: u32,
    /// Identificativo di questa cartella di lavoro.
    pub id_cartella: String,
    pub note: Vec<Nota>,
    /// Impronta dell'ultimo contenuto scritto per ciascun file.
    #[serde(default)]
    pub impronte: BTreeMap<String, String>,
}
