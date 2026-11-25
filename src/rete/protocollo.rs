//! Formato dei messaggi scambiati in rete.
//!
//! TCP: ogni messaggio e' `lunghezza (u32, big endian) || dati cifrati`,
//! dove i dati in chiaro sono JSON.
//!
//! UDP (annunci): `MAGICO || etichetta gruppo (16 byte ASCII) || dati cifrati`.

use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

use super::cifratura::Chiave;
use crate::modello::Nota;
use crate::modello::nota::Versioni;

pub const VERSIONE_PROTOCOLLO: u32 = 2;
pub const PORTA_SCOPERTA: u16 = 47810;
pub const PORTA_SINCRONIZZAZIONE: u16 = 47811;
const MAGICO: &[u8; 8] = b"SCHEDAR2";
const DIMENSIONE_MASSIMA: usize = 64 * 1024 * 1024;

/// Contenuto (cifrato) di un annuncio UDP.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Annuncio {
    pub versione: u32,
    pub id_dispositivo: String,
    pub nome: String,
    /// Porta TCP su cui il dispositivo accetta le sincronizzazioni.
    pub porta: u16,
}

/// Versione di una nota senza il contenuto: basta per capire chi e' indietro.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timbro {
    pub id: String,
    pub modificata: i64,
    pub origine: String,
    #[serde(default)]
    pub versioni: Versioni,
}

/// Sequenza di una sincronizzazione: Offerta -> Risposta -> Invio.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Messaggio {
    /// Chi chiama presenta le versioni che possiede.
    Offerta { versione: u32, id_dispositivo: String, nome: String, porta: u16, timbri: Vec<Timbro> },
    /// Chi risponde invia le note piu' recenti (o concorrenti) e chiede le altre.
    Risposta { id_dispositivo: String, nome: String, note: Vec<Nota>, richieste: Vec<String> },
    /// Chi chiama invia le note richieste.
    Invio { note: Vec<Nota> },
    /// "Mandami il tuo ordine delle schede" (bottone "Prendi l'ordine delle
    /// schede da…"). Una versione precedente di Schedario non lo conosce e
    /// chiude il collegamento: chi chiede lo dice all'utente.
    RichiestaOrdine { id_dispositivo: String },
    /// Risposta: gli identificativi delle schede nell'ordine mostrato.
    Ordine { ordine: Vec<String> },
}

pub fn scrivi_messaggio(flusso: &mut impl Write, chiave: &Chiave, messaggio: &Messaggio) -> io::Result<()> {
    let json = serde_json::to_vec(messaggio).map_err(io::Error::other)?;
    let dati = chiave.sigilla(&json);
    flusso.write_all(&(dati.len() as u32).to_be_bytes())?;
    flusso.write_all(&dati)?;
    flusso.flush()
}

pub fn leggi_messaggio(flusso: &mut impl Read, chiave: &Chiave) -> Result<Messaggio, String> {
    let mut lunghezza = [0u8; 4];
    flusso.read_exact(&mut lunghezza).map_err(|e| e.to_string())?;
    let lunghezza = u32::from_be_bytes(lunghezza) as usize;
    if lunghezza > DIMENSIONE_MASSIMA {
        return Err("messaggio troppo grande".into());
    }
    let mut dati = vec![0u8; lunghezza];
    flusso.read_exact(&mut dati).map_err(|e| e.to_string())?;
    let json = chiave.apri(&dati)?;
    serde_json::from_slice(&json).map_err(|e| format!("messaggio non valido: {e}"))
}

pub fn codifica_annuncio(chiave: &Chiave, annuncio: &Annuncio) -> Vec<u8> {
    let json = serde_json::to_vec(annuncio).expect("serializzazione di un annuncio");
    let mut pacchetto = Vec::with_capacity(64 + json.len());
    pacchetto.extend_from_slice(MAGICO);
    pacchetto.extend_from_slice(chiave.etichetta().as_bytes());
    pacchetto.extend_from_slice(&chiave.sigilla(&json));
    pacchetto
}

/// Restituisce l'annuncio solo se appartiene al nostro gruppo ed e' autentico.
pub fn decodifica_annuncio(chiave: &Chiave, pacchetto: &[u8]) -> Option<Annuncio> {
    let resto = pacchetto.strip_prefix(MAGICO.as_slice())?;
    let etichetta = chiave.etichetta().as_bytes();
    let cifrato = resto.strip_prefix(etichetta)?;
    let json = chiave.apri(cifrato).ok()?;
    let annuncio: Annuncio = serde_json::from_slice(&json).ok()?;
    (annuncio.versione == VERSIONE_PROTOCOLLO).then_some(annuncio)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn annuncio_e_messaggio_andata_e_ritorno() {
        let chiave = Chiave::da_codice("gruppo-prova");
        let annuncio = Annuncio {
            versione: VERSIONE_PROTOCOLLO,
            id_dispositivo: "x".into(),
            nome: "PC".into(),
            porta: 1234,
        };
        let pacchetto = codifica_annuncio(&chiave, &annuncio);
        assert_eq!(decodifica_annuncio(&chiave, &pacchetto), Some(annuncio));
        assert_eq!(decodifica_annuncio(&Chiave::da_codice("altro-gruppo"), &pacchetto), None);

        let mut buffer = Vec::new();
        scrivi_messaggio(&mut buffer, &chiave, &Messaggio::Invio { note: vec![Nota::nuova("a")] }).unwrap();
        match leggi_messaggio(&mut buffer.as_slice(), &chiave).unwrap() {
            Messaggio::Invio { note } => assert_eq!(note.len(), 1),
            altro => panic!("inatteso: {altro:?}"),
        }
    }
}
