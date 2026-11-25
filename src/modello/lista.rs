//! Liste di controllo dentro il testo di una nota.
//!
//! Una riga che inizia con `[ ]` e' una voce da fare, con `[x]` una voce
//! fatta. Prima del segno possono esserci spazi (per i rientri) e un trattino
//! `- ` o un asterisco `* ` (lo stile Markdown). Il file `.txt` resta testo
//! normale, leggibile con qualsiasi editor:
//!
//! ```text
//! [ ] latte
//! [x] pane
//! ```
//!
//! Tutte le posizioni sono indici di **carattere** (non di byte), come quelle
//! del cursore dell'editor.

/// Una voce di lista trovata nel testo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Voce {
    /// Primo carattere della riga.
    pub inizio_riga: usize,
    /// Posizione della `[` del segno: il segno occupa tre caratteri.
    pub segno: usize,
    /// Primo carattere dopo il segno e lo spazio che lo segue.
    pub inizio_testo: usize,
    /// Fine della riga (posizione del `\n` o fine del testo).
    pub fine_riga: usize,
    pub fatta: bool,
}

/// Lunghezza del segno `[ ]` in caratteri.
pub const LUNGHEZZA_SEGNO: usize = 3;
const NUOVA_VOCE: &str = "[ ] ";

/// Tutte le voci di lista del testo, in ordine.
pub fn voci(testo: &str) -> Vec<Voce> {
    let mut trovate = Vec::new();
    let mut inizio = 0;
    for riga in testo.split('\n') {
        let lunghezza = riga.chars().count();
        if let Some((scarto, fatta, dopo)) = segno_in(riga) {
            trovate.push(Voce {
                inizio_riga: inizio,
                segno: inizio + scarto,
                inizio_testo: inizio + dopo,
                fine_riga: inizio + lunghezza,
                fatta,
            });
        }
        inizio += lunghezza + 1;
    }
    trovate
}

/// Cerca il segno all'inizio della riga: (posizione, fatta, inizio del testo).
fn segno_in(riga: &str) -> Option<(usize, bool, usize)> {
    let senza_rientro = riga.trim_start_matches([' ', '\t']);
    let rientro = riga.len() - senza_rientro.len(); // spazi e tab: un byte ciascuno
    let (resto, punto) = match senza_rientro.get(..2) {
        Some("- " | "* ") => (&senza_rientro[2..], 2),
        _ => (senza_rientro, 0),
    };
    let fatta = match resto.get(..3) {
        Some("[ ]") => false,
        Some("[x]" | "[X]") => true,
        _ => return None,
    };
    // Dopo il segno: fine riga oppure uno spazio ("[x]latte" non e' una voce).
    let dopo = &resto[3..];
    let spazio = match dopo.chars().next() {
        None | Some('\r') => 0,
        Some(' ') => 1,
        Some(_) => return None,
    };
    let segno = rientro + punto;
    Some((segno, fatta, segno + LUNGHEZZA_SEGNO + spazio))
}

/// Spunta o toglie la spunta della voce il cui segno inizia in `segno`.
/// Restituisce `true` se il testo e' cambiato.
pub fn alterna(testo: &mut String, segno: usize) -> bool {
    let Some(voce) = voci(testo).into_iter().find(|v| v.segno == segno) else { return false };
    let posizione = byte(testo, segno + 1);
    let nuovo = if voce.fatta { " " } else { "x" };
    testo.replace_range(posizione..posizione + 1, nuovo);
    true
}

/// Trasforma la riga del cursore in una voce di lista, o la riporta a testo
/// normale se lo era gia'. Restituisce la nuova posizione del cursore.
pub fn alterna_riga(testo: &mut String, cursore: usize) -> usize {
    let inizio_riga = inizio_riga(testo, cursore);
    match voci(testo).into_iter().find(|v| v.inizio_riga == inizio_riga) {
        Some(voce) => {
            let da = byte(testo, voce.segno);
            let a = byte(testo, voce.inizio_testo);
            testo.replace_range(da..a, "");
            let tolti = voce.inizio_testo - voce.segno;
            if cursore >= voce.inizio_testo { cursore - tolti } else { cursore.min(voce.segno) }
        }
        None => {
            let riga: String = testo.chars().skip(inizio_riga).take_while(|c| *c != '\n').collect();
            let rientro = riga.chars().take_while(|c| *c == ' ' || *c == '\t').count();
            let dove = inizio_riga + rientro;
            let posizione = byte(testo, dove);
            testo.insert_str(posizione, NUOVA_VOCE);
            let aggiunti = NUOVA_VOCE.chars().count();
            if cursore >= dove { cursore + aggiunti } else { dove + aggiunti }
        }
    }
}

/// Da chiamare subito dopo che l'utente ha premuto Invio, con il cursore
/// all'inizio della riga nuova. Se la riga precedente era una voce:
/// - con del testo, la riga nuova diventa una voce vuota (con lo stesso rientro);
/// - vuota, la lista finisce: il segno viene tolto e la riga nuova eliminata.
///
/// Restituisce la nuova posizione del cursore, o `None` se non c'era una lista.
pub fn continua_dopo_invio(testo: &mut String, cursore: usize) -> Option<usize> {
    if cursore == 0 || testo.chars().nth(cursore - 1) != Some('\n') {
        return None;
    }
    let precedente = inizio_riga(testo, cursore - 1);
    let voce = voci(testo).into_iter().find(|v| v.inizio_riga == precedente)?;
    let contenuto: String = testo.chars().skip(voce.inizio_testo).take(voce.fine_riga - voce.inizio_testo).collect();
    if contenuto.trim().is_empty() {
        // Voce vuota: si esce dalla lista.
        let da = byte(testo, voce.segno);
        let a = byte(testo, cursore);
        testo.replace_range(da..a, "");
        return Some(voce.segno);
    }
    let prefisso: String = testo.chars().skip(voce.inizio_riga).take(voce.segno - voce.inizio_riga).collect();
    let inserito = format!("{prefisso}{NUOVA_VOCE}");
    let posizione = byte(testo, cursore);
    testo.insert_str(posizione, &inserito);
    Some(cursore + inserito.chars().count())
}

/// Cancellando il carattere in posizione `cancellato` (con Backspace): se fa
/// parte del segno di una voce (`[ ] ` o `[x] `, spazio compreso), si toglie
/// tutto il segno, cosi' non restano parentesi a meta'. "[ ] latte" diventa
/// "latte". Restituisce la nuova posizione del cursore, o `None` se il
/// carattere non era nel segno (allora la cancellazione e' quella normale).
pub fn cancella_segno(testo: &mut String, cancellato: usize) -> Option<usize> {
    let voce = voci(testo).into_iter().find(|v| (v.segno..v.inizio_testo).contains(&cancellato))?;
    let da = byte(testo, voce.segno);
    let a = byte(testo, voce.inizio_testo);
    testo.replace_range(da..a, "");
    Some(voce.segno)
}

/// Indice del primo carattere della riga che contiene `cursore`.
fn inizio_riga(testo: &str, cursore: usize) -> usize {
    let prima: Vec<char> = testo.chars().take(cursore).collect();
    prima.iter().rposition(|c| *c == '\n').map(|p| p + 1).unwrap_or(0)
}

/// Converte un indice di carattere in indice di byte.
fn byte(testo: &str, carattere: usize) -> usize {
    testo.char_indices().nth(carattere).map(|(b, _)| b).unwrap_or(testo.len())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn riconosce_le_voci_e_ignora_il_resto() {
        let testo = "Spesa\n[ ] latte\n[x] pane\n  - [X] uova\n[x]senza spazio\n[ ]\nnota [ ] in mezzo";
        let trovate = voci(testo);
        assert_eq!(trovate.len(), 4);
        assert_eq!((trovate[0].segno, trovate[0].fatta), (6, false));
        assert!(trovate[1].fatta);
        assert_eq!(trovate[2].segno - trovate[2].inizio_riga, 4, "rientro e trattino");
        assert_eq!(trovate[3].inizio_testo, trovate[3].fine_riga, "voce vuota a fine riga");
    }

    #[test]
    fn funziona_con_caratteri_accentati() {
        let mut testo = "caffè\n[ ] tè".to_owned();
        let voce = voci(&testo)[0];
        assert_eq!(voce.segno, 6);
        assert!(alterna(&mut testo, voce.segno));
        assert_eq!(testo, "caffè\n[x] tè");
        assert!(alterna(&mut testo, voce.segno));
        assert_eq!(testo, "caffè\n[ ] tè");
        assert!(!alterna(&mut testo, 0), "nessuna voce in quella posizione");
    }

    #[test]
    fn trasforma_una_riga_e_torna_indietro() {
        let mut testo = "uno\n  due".to_owned();
        let cursore = alterna_riga(&mut testo, 7); // dentro "due"
        assert_eq!(testo, "uno\n  [ ] due");
        assert_eq!(cursore, 11);
        let cursore = alterna_riga(&mut testo, cursore);
        assert_eq!(testo, "uno\n  due");
        assert_eq!(cursore, 7);
    }

    #[test]
    fn invio_continua_la_lista_e_una_voce_vuota_la_chiude() {
        let mut testo = "  [ ] latte\n".to_owned();
        let cursore = continua_dopo_invio(&mut testo, 12).unwrap();
        assert_eq!(testo, "  [ ] latte\n  [ ] ");
        assert_eq!(cursore, 18);

        testo.push('\n'); // Invio sulla voce vuota
        let cursore = continua_dopo_invio(&mut testo, 19).unwrap();
        assert_eq!(testo, "  [ ] latte\n  ");
        assert_eq!(cursore, 14);

        let mut normale = "testo\n".to_owned();
        assert_eq!(continua_dopo_invio(&mut normale, 6), None);
        assert_eq!(normale, "testo\n");
    }

    #[test]
    fn backspace_sul_segno_lo_toglie_tutto() {
        // Cursore dopo "[ ] ": Backspace cancella lo spazio, cioe' il carattere 3.
        let mut testo = "Spesa\n[ ] latte".to_owned();
        assert_eq!(cancella_segno(&mut testo, 9), Some(6));
        assert_eq!(testo, "Spesa\nlatte");
        // Anche una voce fatta, e anche cancellando la parentesi.
        let mut testo = "  [x] pane".to_owned();
        assert_eq!(cancella_segno(&mut testo, 4), Some(2));
        assert_eq!(testo, "  pane");
        // Voce vuota a fine riga.
        let mut testo = "[ ]".to_owned();
        assert_eq!(cancella_segno(&mut testo, 2), Some(0));
        assert_eq!(testo, "");
        // Fuori dal segno: cancellazione normale.
        let mut testo = "[ ] latte".to_owned();
        assert_eq!(cancella_segno(&mut testo, 5), None);
        assert_eq!(testo, "[ ] latte");
    }

    #[test]
    fn invio_a_meta_voce_la_divide_in_due() {
        let mut testo = "[ ] latte\npane".to_owned(); // Invio tra "latte" e "pane"
        let cursore = continua_dopo_invio(&mut testo, 10).unwrap();
        assert_eq!(testo, "[ ] latte\n[ ] pane");
        assert_eq!(cursore, 14);
    }
}
