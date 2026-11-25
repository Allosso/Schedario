//! Date e ore nell'ora locale del computer.
//!
//! Il fuso orario si legge dal sistema operativo (con `chrono`): le
//! impostazioni di Windows, `/etc/localtime` o `TZ` su Linux e macOS. L'ora
//! legale e' calcolata per la data mostrata, quindi una versione di luglio
//! appare con l'ora estiva anche se la si guarda a dicembre.
//!
//! Negli archivi e in rete gli istanti restano in millisecondi UTC
//! (`modello::tempo`): la conversione serve solo a mostrarli.

use chrono::{DateTime, Local, TimeZone};

fn locale(ms: i64) -> Option<DateTime<Local>> {
    Local.timestamp_millis_opt(ms).single()
}

/// Data e ora: `22/11/2026 14:05` in italiano, `2026-11-22 14:05` in inglese.
/// Se il sistema non fornisce il fuso orario si usa l'ora UTC, indicata come tale.
pub fn data_ora(ms: i64, lingua: &str) -> String {
    let formato = if lingua == "en" { "%Y-%m-%d %H:%M" } else { "%d/%m/%Y %H:%M" };
    locale(ms)
        .map(|d| d.format(formato).to_string())
        .unwrap_or_else(|| crate::modello::tempo::formatta(ms))
}

/// Un istante descritto rispetto a oggi: "oggi alle 10:42", "ieri alle
/// 18:03" oppure "il 12/11/2026 alle 09:15".
pub struct Quando {
    /// Giorni di calendario fa: 0 oggi, 1 ieri. Negativo se l'orologio di un
    /// altro computer era avanti.
    pub giorni_fa: i64,
    pub data: String,
    pub ora: String,
}

pub fn quando(ms: i64, lingua: &str) -> Quando {
    quando_rispetto_a(ms, Local::now(), lingua)
}

fn quando_rispetto_a(ms: i64, adesso: DateTime<Local>, lingua: &str) -> Quando {
    let formato = if lingua == "en" { "%Y-%m-%d" } else { "%d/%m/%Y" };
    match locale(ms) {
        Some(d) => Quando {
            giorni_fa: (adesso.date_naive() - d.date_naive()).num_days(),
            data: d.format(formato).to_string(),
            ora: d.format("%H:%M").to_string(),
        },
        None => {
            let utc = crate::modello::tempo::formatta(ms);
            Quando { giorni_fa: -1, data: utc[..10].to_owned(), ora: utc[11..].to_owned() }
        }
    }
}

/// Solo l'ora, per esempio `14:05`.
pub fn ora(ms: i64) -> String {
    locale(ms)
        .map(|d| d.format("%H:%M").to_string())
        .unwrap_or_else(|| crate::modello::tempo::formatta(ms)[11..16].to_owned())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn usa_il_fuso_del_sistema() {
        let istante = 1_795_219_200_000; // 2026-11-21 00:00 UTC
        let atteso = Local.timestamp_millis_opt(istante).unwrap();
        assert_eq!(ora(istante), atteso.format("%H:%M").to_string());
        assert_eq!(data_ora(istante, "it"), atteso.format("%d/%m/%Y %H:%M").to_string());
        assert_eq!(data_ora(istante, "en"), atteso.format("%Y-%m-%d %H:%M").to_string());
        assert!(!data_ora(istante, "it").contains("UTC"));
    }

    #[test]
    fn oggi_ieri_e_data() {
        let adesso = Local.with_ymd_and_hms(2026, 11, 22, 10, 30, 0).unwrap();
        let ms = |d: DateTime<Local>| d.timestamp_millis();
        let stamattina = Local.with_ymd_and_hms(2026, 11, 22, 0, 5, 0).unwrap();
        let q = quando_rispetto_a(ms(stamattina), adesso, "it");
        assert_eq!((q.giorni_fa, q.ora.as_str()), (0, "00:05"));
        let ieri_sera = Local.with_ymd_and_hms(2026, 11, 21, 23, 55, 0).unwrap();
        assert_eq!(quando_rispetto_a(ms(ieri_sera), adesso, "it").giorni_fa, 1);
        let prima = Local.with_ymd_and_hms(2026, 11, 3, 9, 12, 0).unwrap();
        let q = quando_rispetto_a(ms(prima), adesso, "it");
        assert_eq!((q.giorni_fa, q.data.as_str()), (19, "03/11/2026"));
        assert_eq!(quando_rispetto_a(ms(prima), adesso, "en").data, "2026-11-03");
    }
}
