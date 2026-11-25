//! Crea l'icona da 512 × 512 per la scheda del Play Store, dallo stesso
//! disegno dell'app (src/interfaccia/icona_forme.rs).
//!
//!   cargo run --example icona_store -- dist\play-store\icona-512.png
//!
//! Il PNG e' scritto senza librerie: i dati sono in blocchi "non compressi"
//! di zlib, quindi il file e' grande (circa 1 MB) ma valido; il Play Store
//! accetta fino a 1 MB, percio' i pixel sono salvati senza canale alfa
//! (l'icona del Play Store non ha trasparenze) e il file resta sotto il limite.

use std::path::PathBuf;

const LATO: u32 = 512;

fn main() {
    let destinazione = std::env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("icona-512.png"));
    let rgba = schedario::interfaccia::icona_forme::disegna_a_tutto_campo(LATO);

    // Righe del PNG: un byte di filtro (0, nessuno) e poi i pixel RGB.
    let mut grezzo = Vec::with_capacity(((LATO * 3 + 1) * LATO) as usize);
    for riga in rgba.chunks((LATO * 4) as usize) {
        grezzo.push(0);
        for pixel in riga.chunks(4) {
            grezzo.extend_from_slice(&pixel[..3]);
        }
    }

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut intestazione = Vec::new();
    intestazione.extend_from_slice(&LATO.to_be_bytes());
    intestazione.extend_from_slice(&LATO.to_be_bytes());
    // 8 bit per canale, colore RGB, compressione/filtro/interlacciamento standard.
    intestazione.extend_from_slice(&[8, 2, 0, 0, 0]);
    blocco(&mut png, b"IHDR", &intestazione);
    blocco(&mut png, b"IDAT", &zlib_senza_compressione(&grezzo));
    blocco(&mut png, b"IEND", &[]);

    if let Some(cartella) = destinazione.parent().filter(|c| !c.as_os_str().is_empty()) {
        std::fs::create_dir_all(cartella).expect("creazione della cartella");
    }
    std::fs::write(&destinazione, &png).expect("scrittura del PNG");
    println!("{} ({} KB)", destinazione.display(), png.len() / 1024);
}

fn blocco(png: &mut Vec<u8>, tipo: &[u8; 4], dati: &[u8]) {
    png.extend_from_slice(&(dati.len() as u32).to_be_bytes());
    let inizio = png.len();
    png.extend_from_slice(tipo);
    png.extend_from_slice(dati);
    let crc = crc32(&png[inizio..]);
    png.extend_from_slice(&crc.to_be_bytes());
}

fn zlib_senza_compressione(dati: &[u8]) -> Vec<u8> {
    let mut uscita = vec![0x78, 0x01];
    let pezzi: Vec<&[u8]> = dati.chunks(65_535).collect();
    for (i, pezzo) in pezzi.iter().enumerate() {
        uscita.push(u8::from(i + 1 == pezzi.len()));
        let lunghezza = pezzo.len() as u16;
        uscita.extend_from_slice(&lunghezza.to_le_bytes());
        uscita.extend_from_slice(&(!lunghezza).to_le_bytes());
        uscita.extend_from_slice(pezzo);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in dati {
        a = (a + byte as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    uscita.extend_from_slice(&((b << 16) | a).to_be_bytes());
    uscita
}

fn crc32(dati: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in dati {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}
