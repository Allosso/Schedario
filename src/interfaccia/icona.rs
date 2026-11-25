//! Icona del programma generata in codice: nessun file immagine da
//! distribuire e nessuna licenza grafica da rispettare. Il disegno e' in
//! `icona_forme.rs`; qui diventa l'icona della finestra e dell'area di
//! notifica. Lo stesso disegno e' il logo dentro il programma
//! (`componenti::disegna_logo`) e l'icona dell'eseguibile (`build.rs`).

use std::sync::Arc;

use egui::IconData;

pub use super::icona_forme::{FONDO, FORME, GRIGLIA};

/// Abbastanza grande per la barra delle applicazioni con lo schermo
/// ingrandito; il sistema la riduce dove serve.
const LATO: u32 = 128;

pub fn icona() -> Arc<IconData> {
    Arc::new(IconData { rgba: super::icona_forme::disegna(LATO), width: LATO, height: LATO })
}

pub fn colore_egui(colore: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(colore[0], colore[1], colore[2])
}

#[cfg(test)]
mod test {
    use super::super::icona_forme::{CARTA, disegna, file_ico};

    #[test]
    fn l_icona_ha_angoli_trasparenti_e_centro_pieno() {
        let rgba = disegna(64);
        let alfa = |x: u32, y: u32| rgba[((y * 64 + x) * 4 + 3) as usize];
        assert_eq!(alfa(0, 0), 0);
        assert_eq!(alfa(63, 63), 0);
        assert_eq!(alfa(32, 32), 255);
        // La maniglia e' color carta, al centro del cassetto.
        let i = ((48 * 64 + 32) * 4) as usize;
        assert_eq!(&rgba[i..i + 3], &CARTA);
    }

    #[test]
    fn il_file_ico_elenca_tutte_le_grandezze() {
        let ico = file_ico(&[16, 32, 256]);
        assert_eq!(&ico[0..6], &[0, 0, 1, 0, 3, 0]);
        // Voce della 256: larghezza e altezza scritte come 0.
        assert_eq!(&ico[6 + 32..6 + 34], &[0, 0]);
        // L'ultima immagine finisce esattamente alla fine del file.
        let voce = 6 + 32;
        let lunghezza = u32::from_le_bytes(ico[voce + 8..voce + 12].try_into().unwrap()) as usize;
        let inizio = u32::from_le_bytes(ico[voce + 12..voce + 16].try_into().unwrap()) as usize;
        assert_eq!(inizio + lunghezza, ico.len());
    }
}
