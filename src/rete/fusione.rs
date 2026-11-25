//! Regole di fusione: decidono quali note scambiare e quale versione tenere.
//!
//! Ogni nota porta un vettore di versioni (un contatore per computer):
//! - se una versione **segue** l'altra, la piu' recente sostituisce la vecchia;
//! - se due versioni sono **concorrenti** (modificate in parallelo su due
//!   computer), vince quella salvata per ultima, ma l'altra non va persa:
//!   diventa una nuova nota "copia in conflitto".
//!
//! Tutte le scelte sono deterministiche: due computer che risolvono lo stesso
//! conflitto arrivano esattamente allo stesso risultato.

use std::collections::HashMap;

use super::protocollo::Timbro;
use crate::modello::nota::{Relazione, relazione, unisci_versioni};
use crate::modello::Nota;

pub fn timbri(note: &[Nota]) -> Vec<Timbro> {
    note.iter()
        .map(|n| Timbro {
            id: n.id.clone(),
            modificata: n.modificata,
            origine: n.origine.clone(),
            versioni: n.versioni.clone(),
        })
        .collect()
}

fn relazione_nota_timbro(n: &Nota, t: &Timbro) -> Relazione {
    relazione((&n.versioni, n.modificata, &n.origine), (&t.versioni, t.modificata, &t.origine))
}

/// Confronta le note locali con i timbri dell'altro dispositivo.
/// Restituisce (note da inviare, identificativi da chiedere).
/// Le versioni concorrenti vengono sia inviate sia chieste: ciascun lato
/// risolve poi il conflitto da se', arrivando allo stesso risultato.
pub fn confronta(locali: &[Nota], remoti: &[Timbro]) -> (Vec<Nota>, Vec<String>) {
    let remoti_per_id: HashMap<&str, &Timbro> = remoti.iter().map(|t| (t.id.as_str(), t)).collect();
    let locali_per_id: HashMap<&str, &Nota> = locali.iter().map(|n| (n.id.as_str(), n)).collect();

    let da_inviare = locali
        .iter()
        .filter(|n| match remoti_per_id.get(n.id.as_str()) {
            None => true,
            Some(t) => matches!(relazione_nota_timbro(n, t), Relazione::Segue | Relazione::Concorrente),
        })
        .cloned()
        .collect();

    let da_chiedere = remoti
        .iter()
        .filter(|t| match locali_per_id.get(t.id.as_str()) {
            None => true,
            Some(n) => matches!(relazione_nota_timbro(n, t), Relazione::Precede | Relazione::Concorrente),
        })
        .map(|t| t.id.clone())
        .collect();

    (da_inviare, da_chiedere)
}

/// Esito dell'arrivo di una nota da un altro computer.
#[derive(Debug, PartialEq)]
pub enum Fusione {
    /// La versione locale e' gia' uguale o piu' recente.
    Ignora,
    /// La nota arrivata sostituisce (o si aggiunge a) quella locale.
    Accetta(Nota),
    /// Modifiche in parallelo: `unita` prende il posto della nota; `copia`,
    /// se presente, conserva il testo della versione che ha perso.
    Conflitto { unita: Nota, copia: Option<Nota> },
}

pub fn fondi(locale: Option<&Nota>, arrivata: &Nota) -> Fusione {
    if !crate::modello::nota::id_valido(&arrivata.id) {
        return Fusione::Ignora;
    }
    let Some(locale) = locale else {
        return Fusione::Accetta(arrivata.clone());
    };
    match arrivata.relazione_con(locale) {
        Relazione::Segue => Fusione::Accetta(arrivata.clone()),
        Relazione::Precede | Relazione::Uguale => Fusione::Ignora,
        Relazione::Concorrente => {
            let (vincitrice, perdente) =
                if arrivata.piu_recente_di(locale) { (arrivata, locale) } else { (locale, arrivata) };
            let mut unita = vincitrice.clone();
            unita.versioni = unisci_versioni(&locale.versioni, &arrivata.versioni);
            // Si conserva la perdente solo se contiene testo diverso e reale.
            let da_conservare = !perdente.eliminata
                && !perdente.stesso_testo(vincitrice)
                && !(perdente.titolo.trim().is_empty() && perdente.testo.trim().is_empty());
            let copia = da_conservare.then(|| perdente.copia_in_conflitto());
            Fusione::Conflitto { unita, copia }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn nota(id: &str, testo: &str) -> Nota {
        let mut n = Nota::nuova("base");
        n.id = id.repeat(32);
        n.testo = testo.into();
        n
    }

    #[test]
    fn scambia_solo_cio_che_serve() {
        let base_a = nota("a", "");
        let base_b = nota("b", "");
        let mut b_piu_nuova = base_b.clone();
        b_piu_nuova.segna_modificata("remoto");
        let mut c_locale = nota("c", "");
        let c_remota = c_locale.clone();
        c_locale.segna_modificata("locale");

        let locali = vec![base_a.clone(), base_b, c_locale];
        let remoti = timbri(&[base_a, b_piu_nuova, c_remota, nota("d", "")]);
        let (inviare, chiedere) = confronta(&locali, &remoti);
        let inviare: Vec<_> = inviare.iter().map(|n| &n.id[..1]).collect();
        let chiedere: Vec<_> = chiedere.iter().map(|id| &id[..1]).collect();
        assert_eq!(inviare, ["c"]);
        assert_eq!(chiedere, ["b", "d"]);
    }

    #[test]
    fn una_versione_successiva_viene_accettata_quella_vecchia_ignorata() {
        let vecchia = nota("a", "uno");
        let mut nuova = vecchia.clone();
        nuova.testo = "due".into();
        nuova.segna_modificata("remoto");
        assert_eq!(fondi(Some(&vecchia), &nuova), Fusione::Accetta(nuova.clone()));
        assert_eq!(fondi(Some(&nuova), &vecchia), Fusione::Ignora);
        assert_eq!(fondi(None, &nuova), Fusione::Accetta(nuova));
    }

    #[test]
    fn un_conflitto_conserva_entrambi_i_testi_ed_e_simmetrico() {
        let base = nota("a", "base");
        let mut su_a = base.clone();
        su_a.testo = "scritto su A".into();
        su_a.segna_modificata("pc-a");
        let mut su_b = base.clone();
        su_b.testo = "scritto su B".into();
        su_b.segna_modificata("pc-b");

        // Ciascun computer riceve la versione dell'altro.
        let su_a_riceve = fondi(Some(&su_a), &su_b);
        let su_b_riceve = fondi(Some(&su_b), &su_a);
        assert_eq!(su_a_riceve, su_b_riceve, "entrambi devono arrivare allo stesso risultato");

        let Fusione::Conflitto { unita, copia: Some(copia) } = su_a_riceve else { panic!("atteso conflitto") };
        let testi = [unita.testo.as_str(), copia.testo.as_str()];
        assert!(testi.contains(&"scritto su A") && testi.contains(&"scritto su B"));
        assert_eq!(unita.relazione_con(&su_a), crate::modello::nota::Relazione::Segue);
        assert_eq!(unita.relazione_con(&su_b), crate::modello::nota::Relazione::Segue);
    }

    #[test]
    fn un_conflitto_con_lo_stesso_testo_non_crea_copie() {
        let base = nota("a", "base");
        let mut su_a = base.clone();
        su_a.nel_cestino = true;
        su_a.segna_modificata("pc-a");
        let mut su_b = base.clone();
        su_b.segna_modificata("pc-b");
        assert!(matches!(fondi(Some(&su_a), &su_b), Fusione::Conflitto { copia: None, .. }));
    }

    #[test]
    fn una_eliminazione_contro_una_modifica_salva_la_modifica() {
        let base = nota("a", "base");
        let mut eliminata = base.clone();
        eliminata.elimina("pc-a");
        let mut modificata = base.clone();
        modificata.testo = "aggiunta importante".into();
        modificata.segna_modificata("pc-b");
        // L'eliminazione e' piu' recente: vince, ma la modifica resta come copia.
        eliminata.modificata = modificata.modificata + 10;
        match fondi(Some(&modificata), &eliminata) {
            Fusione::Conflitto { unita, copia: Some(copia) } => {
                assert!(unita.eliminata);
                assert_eq!(copia.testo, "aggiunta importante");
            }
            altro => panic!("inatteso: {altro:?}"),
        }
    }
}
