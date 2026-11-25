//! Istanti espressi in millisecondi dall'epoca Unix (UTC).
//! Un numero intero e' semplice da salvare, confrontare e inviare in rete.

use std::time::{SystemTime, UNIX_EPOCH};

/// Millisecondi trascorsi dal 1 gennaio 1970 (UTC).
pub fn adesso_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|durata| durata.as_millis() as i64)
        .unwrap_or(0)
}

/// Restituisce un istante sempre successivo a `precedente`, anche se
/// l'orologio del computer e' stato spostato indietro.
pub fn successivo(precedente: i64) -> i64 {
    adesso_ms().max(precedente + 1)
}

/// Data e ora leggibili (UTC), per esempio `2026-11-21 14:05`.
pub fn formatta(ms: i64) -> String {
    let secondi = ms.div_euclid(1000);
    let giorni = secondi.div_euclid(86_400);
    let resto = secondi.rem_euclid(86_400);
    let (anno, mese, giorno) = data_civile(giorni);
    format!(
        "{anno:04}-{mese:02}-{giorno:02} {:02}:{:02} UTC",
        resto / 3600,
        (resto % 3600) / 60
    )
}

/// Converte i giorni dall'epoca in (anno, mese, giorno) del calendario gregoriano.
/// Algoritmo di Howard Hinnant, di pubblico dominio.
fn data_civile(giorni: i64) -> (i64, u32, u32) {
    let z = giorni + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let giorno = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mese = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let anno = yoe + era * 400 + i64::from(mese <= 2);
    (anno, mese, giorno)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn formatta_una_data_nota() {
        // 2026-11-21 00:00:00 UTC
        assert_eq!(formatta(1_795_219_200_000), "2026-11-21 00:00 UTC");
    }

    #[test]
    fn successivo_e_sempre_maggiore() {
        let futuro = adesso_ms() + 1_000_000;
        assert_eq!(successivo(futuro), futuro + 1);
    }
}
