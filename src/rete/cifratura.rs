//! Chiave di gruppo e cifratura autenticata dei messaggi.
//!
//! La chiave a 256 bit si ricava dal codice di gruppo con **Argon2id**
//! (32 MiB di memoria, 3 passaggi): ogni tentativo di indovinare il codice
//! costa tempo e memoria, anche a chi registra il traffico e prova offline.
//! Ogni messaggio porta un nonce casuale di 12 byte; Poly1305 garantisce che
//! un messaggio alterato o cifrato con un codice diverso venga scartato.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use sha2::{Digest, Sha256};

/// Il sale e' fisso perche' tutti i computer del gruppo devono ricavare la
/// stessa chiave dallo stesso codice, senza scambiarsi nulla prima.
const SALE: &[u8] = b"schedario-rete-v2-sale-di-gruppo";
const MEMORIA_KIB: u32 = 32 * 1024;
const PASSAGGI: u32 = 3;
const LUNGHEZZA_NONCE: usize = 12;

/// Simboli dei codici generati: niente 0/o, 1/l/i, per non confonderli.
const ALFABETO: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";

#[derive(Clone)]
pub struct Chiave {
    cifrario: ChaCha20Poly1305,
    etichetta: String,
}

impl Chiave {
    /// Ricava la chiave dal codice. Richiede qualche decimo di secondo:
    /// va chiamata fuori dal thread dell'interfaccia.
    pub fn da_codice(codice: &str) -> Self {
        let parametri = Params::new(MEMORIA_KIB, PASSAGGI, 1, Some(32)).expect("parametri Argon2 validi");
        let mut chiave = [0u8; 32];
        Argon2::new(Algorithm::Argon2id, Version::V0x13, parametri)
            .hash_password_into(normalizza(codice).as_bytes(), SALE, &mut chiave)
            .expect("derivazione Argon2");
        let cifrario = ChaCha20Poly1305::new_from_slice(&chiave).expect("la chiave ha 32 byte");
        let mut etichetta = Sha256::new();
        etichetta.update(b"etichetta-gruppo");
        etichetta.update(chiave);
        let etichetta = etichetta.finalize().iter().take(8).map(|b| format!("{b:02x}")).collect();
        Self { cifrario, etichetta }
    }

    /// Etichetta pubblica del gruppo (16 cifre esadecimali). Permette di
    /// ignorare subito gli annunci di altri gruppi senza rivelare il codice.
    pub fn etichetta(&self) -> &str {
        &self.etichetta
    }

    /// Cifra: restituisce `nonce || testo cifrato || etichetta Poly1305`.
    pub fn sigilla(&self, dati: &[u8]) -> Vec<u8> {
        let mut nonce = [0u8; LUNGHEZZA_NONCE];
        getrandom::fill(&mut nonce).expect("generatore casuale del sistema non disponibile");
        let cifrato = self
            .cifrario
            .encrypt(&Nonce::from(nonce), dati)
            .expect("la cifratura in memoria non fallisce");
        let mut uscita = Vec::with_capacity(LUNGHEZZA_NONCE + cifrato.len());
        uscita.extend_from_slice(&nonce);
        uscita.extend_from_slice(&cifrato);
        uscita
    }

    /// Decifra e verifica. Fallisce se il codice e' diverso o i dati alterati.
    pub fn apri(&self, dati: &[u8]) -> Result<Vec<u8>, String> {
        if dati.len() < LUNGHEZZA_NONCE + 16 {
            return Err("messaggio troppo corto".into());
        }
        let (nonce, cifrato) = dati.split_at(LUNGHEZZA_NONCE);
        let nonce: [u8; LUNGHEZZA_NONCE] = nonce.try_into().expect("lunghezza controllata");
        self.cifrario
            .decrypt(&Nonce::from(nonce), cifrato)
            .map_err(|_| "messaggio non autentico o codice di gruppo diverso".to_owned())
    }
}

/// Spazi iniziali/finali e maiuscole non contano: "ABCD-efgh" = "abcd-efgh ".
pub fn normalizza(codice: &str) -> String {
    codice.trim().to_lowercase()
}

/// Codice casuale come `k7m2-q9xd-4tpa-zr8c` (circa 79 bit di casualita').
pub fn genera_codice() -> String {
    let mut simboli = String::new();
    while simboli.chars().filter(|c| *c != '-').count() < 16 {
        let mut byte = [0u8; 1];
        getrandom::fill(&mut byte).expect("generatore casuale del sistema non disponibile");
        // Scarta i valori che renderebbero alcuni simboli piu' probabili di altri.
        let limite = 256 - 256 % ALFABETO.len();
        if usize::from(byte[0]) >= limite {
            continue;
        }
        let conteggio = simboli.chars().filter(|c| *c != '-').count();
        if conteggio > 0 && conteggio % 4 == 0 {
            simboli.push('-');
        }
        simboli.push(char::from(ALFABETO[usize::from(byte[0]) % ALFABETO.len()]));
    }
    simboli
}

/// Valutazione approssimativa del codice scelto dall'utente.
pub fn codice_robusto(codice: &str) -> bool {
    let codice = normalizza(codice);
    let distinti = codice.chars().collect::<std::collections::HashSet<_>>().len();
    codice.chars().count() >= 12 && distinti >= 6
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn andata_e_ritorno() {
        let chiave = Chiave::da_codice("segreto-di-prova");
        let sigillato = chiave.sigilla(b"ciao");
        assert_ne!(&sigillato[12..], b"ciao");
        assert_eq!(chiave.apri(&sigillato).unwrap(), b"ciao");
    }

    #[test]
    fn un_codice_diverso_non_apre_e_le_maiuscole_non_contano() {
        let a = Chiave::da_codice("codice-uno");
        let b = Chiave::da_codice("codice-due");
        let a_maiuscolo = Chiave::da_codice("  CODICE-UNO ");
        assert_ne!(a.etichetta(), b.etichetta());
        assert_eq!(a.etichetta(), a_maiuscolo.etichetta());
        assert!(b.apri(&a.sigilla(b"ciao")).is_err());
    }

    #[test]
    fn i_codici_generati_hanno_il_formato_atteso() {
        let codice = genera_codice();
        assert_eq!(codice.len(), 19, "{codice}");
        assert_eq!(codice.matches('-').count(), 3);
        assert!(codice_robusto(&codice));
        assert_ne!(codice, genera_codice());
        assert!(!codice_robusto("aaaaaa"));
    }
}
