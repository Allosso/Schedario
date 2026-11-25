//! Testi dell'interfaccia. I file JSON stanno in `assets/lingue/` e vengono
//! incorporati nell'eseguibile: non serve distribuirli a parte.
//!
//! Per aggiungere una lingua: crea `assets/lingue/<codice>.json` con le stesse
//! chiavi di `it.json` e aggiungila a `LINGUE` e a `testo_lingua`.

use std::collections::HashMap;

/// Lingue disponibili: (codice, nome mostrato).
pub const LINGUE: [(&str, &str); 2] = [("it", "Italiano"), ("en", "English")];

fn testo_lingua(codice: &str) -> &'static str {
    match codice {
        "en" => include_str!("../../assets/lingue/en.json"),
        _ => include_str!("../../assets/lingue/it.json"),
    }
}

pub struct Traduttore {
    codice: String,
    testi: HashMap<String, String>,
    /// L'italiano fa da riserva per le chiavi mancanti in altre lingue.
    riserva: HashMap<String, String>,
}

/// Le chiavi con questa fine sono la versione per il telefono di un testo:
/// "rete_nome@telefono" dice "Nome di questo dispositivo" dove il computer
/// dice "Nome di questo computer". Su Android prendono il posto del testo
/// normale; sul computer non si usano.
const PER_TELEFONO: &str = "@telefono";

fn carica(codice: &str) -> HashMap<String, String> {
    let mut testi: HashMap<String, String> = serde_json::from_str(testo_lingua(codice)).unwrap_or_default();
    if cfg!(target_os = "android") {
        let varianti: Vec<(String, String)> = testi
            .iter()
            .filter_map(|(chiave, testo)| chiave.strip_suffix(PER_TELEFONO).map(|base| (base.to_owned(), testo.clone())))
            .collect();
        testi.extend(varianti);
    }
    testi
}

impl Traduttore {
    pub fn new(codice: &str) -> Self {
        let codice = if LINGUE.iter().any(|(c, _)| *c == codice) { codice } else { "it" };
        Self { codice: codice.to_owned(), testi: carica(codice), riserva: carica("it") }
    }

    pub fn codice(&self) -> &str {
        &self.codice
    }

    /// Testo per la chiave; se manca, restituisce la chiave stessa.
    pub fn t<'a>(&'a self, chiave: &'a str) -> &'a str {
        self.testi
            .get(chiave)
            .or_else(|| self.riserva.get(chiave))
            .map(String::as_str)
            .unwrap_or(chiave)
    }

    /// Testo con segnaposto: `f("parole", &[("n", "3")])` sostituisce `{n}`.
    pub fn f(&self, chiave: &str, valori: &[(&str, &str)]) -> String {
        let mut testo = self.t(chiave).to_owned();
        for (nome, valore) in valori {
            testo = testo.replace(&format!("{{{nome}}}"), valore);
        }
        testo
    }

    /// "1 parola", "3 parole".
    pub fn parole(&self, n: usize) -> String {
        if n == 1 { self.t("una_parola").to_owned() } else { self.f("parole", &[("n", &n.to_string())]) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn le_lingue_hanno_le_stesse_chiavi() {
        let it: HashMap<String, String> = serde_json::from_str(testo_lingua("it")).unwrap();
        let en: HashMap<String, String> = serde_json::from_str(testo_lingua("en")).unwrap();
        let mut mancanti: Vec<_> = it.keys().filter(|k| !en.contains_key(*k)).collect();
        mancanti.extend(en.keys().filter(|k| !it.contains_key(*k)));
        assert!(mancanti.is_empty(), "chiavi non allineate: {mancanti:?}");
    }

    #[test]
    fn le_versioni_per_il_telefono_hanno_il_testo_normale() {
        // Ogni "chiave@telefono" sostituisce una chiave che esiste: un errore
        // di battitura lascerebbe il testo del computer senza accorgersene.
        for codice in ["it", "en"] {
            let testi: HashMap<String, String> = serde_json::from_str(testo_lingua(codice)).unwrap();
            let orfane: Vec<_> = testi
                .keys()
                .filter_map(|k| k.strip_suffix(PER_TELEFONO))
                .filter(|base| !testi.contains_key(*base))
                .collect();
            assert!(orfane.is_empty(), "{codice}: versioni per il telefono senza testo normale: {orfane:?}");
        }
    }

    #[test]
    fn sostituisce_i_segnaposto() {
        let tr = Traduttore::new("it");
        assert_eq!(tr.f("parole", &[("n", "3")]), "3 parole");
    }
}
