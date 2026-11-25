//! Stato dell'interfaccia legato a una cartella di lavoro: widget aperti,
//! nota selezionata, sezioni aperte della barra laterale; e impostazioni
//! della rete locale per questa cartella.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Sessione {
    pub nota_selezionata: Option<String>,
    pub widget: Vec<StatoWidget>,
    /// Sezioni della barra laterale: `true` = aperta.
    pub sezioni: BTreeMap<String, bool>,
    /// La finestra principale era visibile all'uscita?
    pub finestra_visibile: bool,
    /// Ordine delle schede scelto trascinandole (vedi `modello::ordine`).
    /// Vale solo su questo computer: non viaggia con le note.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ordine_schede: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatoWidget {
    pub id_nota: String,
    /// Posizione dell'angolo in alto a sinistra, in punti dello schermo.
    pub x: f32,
    pub y: f32,
    pub larghezza: f32,
    pub altezza: f32,
    #[serde(default = "vero")]
    pub in_primo_piano: bool,
    #[serde(default)]
    pub ridotto: bool,
}

fn vero() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ImpostazioniRete {
    /// La sincronizzazione parte solo se attivata esplicitamente.
    pub attiva: bool,
    /// Codice condiviso: solo i computer con lo stesso codice si vedono e
    /// si scambiano le note. Da esso deriva la chiave di cifratura.
    pub codice_gruppo: String,
    /// Indirizzi aggiunti a mano (`192.168.1.20` o `192.168.1.20:47811`),
    /// utili se la rete blocca la scoperta automatica.
    pub indirizzi_manuali: Vec<String>,
}

impl Default for ImpostazioniRete {
    fn default() -> Self {
        Self { attiva: false, codice_gruppo: String::new(), indirizzi_manuali: Vec::new() }
    }
}
