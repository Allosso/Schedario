//! Ordine delle schede scelto dall'utente, trascinandole.
//!
//! L'ordine e' un elenco di identificativi salvato per cartella su questo
//! computer (non viaggia con le note). Le regole:
//! - le note fissate stanno sempre prima delle altre; l'ordine vale dentro
//!   ciascuno dei due gruppi;
//! - le note che non compaiono nell'elenco (per esempio quelle nuove) vanno in
//!   fondo al loro gruppo, in ordine di creazione;
//! - gli identificativi di note che non esistono piu' vengono ignorati.

use std::collections::HashMap;

use super::Nota;

/// Ordina le note: fissate prima, poi secondo `ordine`, poi per creazione.
pub fn ordina<'a>(mut note: Vec<&'a Nota>, ordine: &[String]) -> Vec<&'a Nota> {
    let posizione: HashMap<&str, usize> = ordine.iter().enumerate().map(|(i, id)| (id.as_str(), i)).collect();
    note.sort_by(|a, b| {
        let chiave = |n: &Nota| (!n.fissata, posizione.get(n.id.as_str()).copied().unwrap_or(usize::MAX), n.creata);
        chiave(a).cmp(&chiave(b)).then_with(|| a.id.cmp(&b.id))
    });
    note
}

/// Quante delle schede (gia' ordinate) sono fissate: sono sempre le prime.
pub fn fissate_in_testa(note: &[&Nota]) -> usize {
    note.iter().take_while(|n| n.fissata).count()
}

/// Dove puo' finire una scheda trascinata dalla posizione `da`: dentro il
/// suo gruppo (fissate o no). Restituisce la posizione `a` limitata.
pub fn limita_al_gruppo(da: usize, a: usize, fissate: usize, totale: usize) -> usize {
    if da < fissate {
        a.min(fissate.saturating_sub(1))
    } else {
        a.clamp(fissate, totale.saturating_sub(1))
    }
}

/// Sposta l'elemento in posizione `da` alla posizione `a` e restituisce il
/// nuovo ordine completo.
pub fn sposta(ids: &[String], da: usize, a: usize) -> Vec<String> {
    let mut nuovo = ids.to_vec();
    if da < nuovo.len() {
        let id = nuovo.remove(da);
        nuovo.insert(a.min(nuovo.len()), id);
    }
    nuovo
}

/// Ordine preso da un altro computer: prima le note che esistono anche qui,
/// nell'ordine dell'altro computer; poi quelle che esistono solo qui,
/// nell'ordine attuale. Le note che esistono solo sull'altro sono ignorate.
pub fn adotta(attuale: &[String], remoto: &[String]) -> Vec<String> {
    let qui: std::collections::HashSet<&str> = attuale.iter().map(String::as_str).collect();
    let mut visti = std::collections::HashSet::new();
    let mut nuovo: Vec<String> =
        remoto.iter().filter(|id| qui.contains(id.as_str()) && visti.insert(id.as_str())).cloned().collect();
    nuovo.extend(attuale.iter().filter(|id| !visti.contains(id.as_str())).cloned());
    nuovo
}

#[cfg(test)]
mod test {
    use super::*;

    fn nota(id: &str, creata: i64, fissata: bool) -> Nota {
        let mut n = Nota::nuova("a");
        n.id = id.to_owned();
        n.creata = creata;
        n.fissata = fissata;
        n
    }

    fn ids(note: &[&Nota]) -> Vec<String> {
        note.iter().map(|n| n.id.clone()).collect()
    }

    #[test]
    fn senza_ordine_vale_la_creazione_e_le_fissate_prima() {
        let (a, b, c) = (nota("a", 1, false), nota("b", 2, true), nota("c", 3, false));
        assert_eq!(ids(&ordina(vec![&c, &a, &b], &[])), ["b", "a", "c"]);
    }

    #[test]
    fn l_ordine_scelto_vale_dentro_ogni_gruppo() {
        let (a, b, c, d) = (nota("a", 1, false), nota("b", 2, false), nota("c", 3, true), nota("d", 4, false));
        let ordine: Vec<String> = ["d", "a", "c", "b"].map(String::from).to_vec();
        // c e' fissata: resta prima anche se nell'elenco viene dopo.
        assert_eq!(ids(&ordina(vec![&a, &b, &c, &d], &ordine)), ["c", "d", "a", "b"]);
        // Una nota nuova, non nell'elenco, va in fondo.
        let e = nota("e", 0, false);
        assert_eq!(ids(&ordina(vec![&a, &b, &c, &d, &e], &ordine)), ["c", "d", "a", "b", "e"]);
    }

    #[test]
    fn spostare_e_limitare_al_gruppo() {
        let elenco: Vec<String> = ["f1", "f2", "a", "b", "c"].map(String::from).to_vec();
        assert_eq!(sposta(&elenco, 4, 2), ["f1", "f2", "c", "a", "b"]);
        assert_eq!(sposta(&elenco, 2, 4), ["f1", "f2", "b", "c", "a"]);
        // Una nota normale non entra tra le fissate, e viceversa.
        assert_eq!(limita_al_gruppo(3, 0, 2, 5), 2);
        assert_eq!(limita_al_gruppo(0, 4, 2, 5), 1);
        assert_eq!(limita_al_gruppo(3, 4, 2, 5), 4);
    }

    #[test]
    fn adottare_l_ordine_di_un_altro_computer() {
        let attuale: Vec<String> = ["a", "b", "c", "solo_qui"].map(String::from).to_vec();
        let remoto: Vec<String> = ["c", "solo_la", "a", "b"].map(String::from).to_vec();
        assert_eq!(adotta(&attuale, &remoto), ["c", "a", "b", "solo_qui"]);
        assert_eq!(adotta(&attuale, &[]), attuale);
    }
}
