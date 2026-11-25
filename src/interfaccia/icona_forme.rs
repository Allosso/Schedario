//! Il disegno dell'icona: forme e rasterizzazione, senza dipendenze.
//!
//! Uno schedario aperto: tre schede in fila con le linguette sfalsate
//! (quella davanti arancione e con tre righe di testo) dentro un cassetto con
//! la maniglia. Questo file non usa nient'altro del programma perche' lo
//! include anche `build.rs`, che ne ricava il file `.ico` dentro
//! l'eseguibile per Windows. Le stesse forme, copiate a mano, sono l'icona di
//! Android (`android/progetto/app/src/main/res/drawable/icona_schede.xml` e
//! `icona_monocromatica.xml`): se si cambia qui, aggiornare anche quei file.

/// Le forme sono descritte su una griglia di 64 × 64.
pub const GRIGLIA: f32 = 64.0;

/// Un rettangolo arrotondato: angolo in alto a sinistra, angolo in basso a
/// destra, raggio degli angoli e colore (RGB).
pub struct Forma {
    pub da: (f32, f32),
    pub a: (f32, f32),
    pub raggio: f32,
    pub colore: [u8; 3],
}

const ACCENTO: [u8; 3] = [0x2F, 0x6B, 0x7E];
const CASSETTO: [u8; 3] = [0x24, 0x52, 0x5F];
pub const CARTA: [u8; 3] = [0xFA, 0xF8, 0xF4];
const SCHEDA_MEDIA: [u8; 3] = [0xE4, 0xEC, 0xEE];
const SCHEDA_FONDO: [u8; 3] = [0xB9, 0xCF, 0xD5];
const ARANCIO: [u8; 3] = [0xE0, 0x8A, 0x3C];

const fn forma(x0: f32, y0: f32, x1: f32, y1: f32, raggio: f32, colore: [u8; 3]) -> Forma {
    Forma { da: (x0, y0), a: (x1, y1), raggio, colore }
}

/// Il fondo: un quadrato arrotondato nel colore principale. Su Android il
/// fondo lo disegna il telefono (nella forma che preferisce), quindi e'
/// separato dal resto.
pub const FONDO: Forma = forma(4.0, 4.0, 60.0, 60.0, 12.0, ACCENTO);

/// Il disegno sopra il fondo, dal piu' lontano al piu' vicino.
pub const FORME: [Forma; 11] = [
    // Scheda in fondo, con la linguetta a destra.
    forma(37.0, 9.0, 48.0, 16.0, 2.0, SCHEDA_FONDO),
    forma(14.0, 13.0, 50.0, 43.0, 3.0, SCHEDA_FONDO),
    // Scheda in mezzo, con la linguetta al centro.
    forma(25.0, 13.0, 36.0, 20.0, 2.0, SCHEDA_MEDIA),
    forma(14.0, 17.0, 50.0, 43.0, 3.0, SCHEDA_MEDIA),
    // Scheda davanti: linguetta arancione a sinistra e tre righe.
    forma(14.0, 17.0, 25.0, 24.0, 2.0, ARANCIO),
    forma(14.0, 21.0, 50.0, 47.0, 3.0, CARTA),
    forma(19.0, 26.5, 45.0, 29.0, 1.25, ACCENTO),
    forma(19.0, 31.5, 45.0, 34.0, 1.25, ACCENTO),
    forma(19.0, 36.5, 37.0, 39.0, 1.25, ACCENTO),
    // Il cassetto e la sua maniglia.
    forma(9.0, 41.0, 55.0, 55.0, 4.0, CASSETTO),
    forma(25.0, 46.0, 39.0, 50.5, 2.25, CARTA),
];

/// Punti di prova per lato di ogni pixel: 4 × 4 = 16, per bordi morbidi.
const CAMPIONI: u32 = 4;

/// L'icona intera (fondo compreso) come pixel RGBA, `lato` × `lato`, riga
/// per riga dall'alto. Gli angoli fuori dal fondo sono trasparenti.
pub fn disegna(lato: u32) -> Vec<u8> {
    disegna_con(lato, false)
}

/// Come `disegna`, ma con il fondo che riempie tutto il quadrato, senza
/// angoli trasparenti: e' l'icona da 512 px che chiede il Play Store, che gli
/// angoli li arrotonda da se'.
pub fn disegna_a_tutto_campo(lato: u32) -> Vec<u8> {
    disegna_con(lato, true)
}

fn disegna_con(lato: u32, fondo_pieno: bool) -> Vec<u8> {
    let scala = GRIGLIA / lato as f32;
    let mut rgba = vec![0u8; (lato * lato * 4) as usize];
    for y in 0..lato {
        for x in 0..lato {
            let mut somma = [0u32; 3];
            let mut coperti = 0u32;
            for sy in 0..CAMPIONI {
                for sx in 0..CAMPIONI {
                    let px = (x as f32 + (sx as f32 + 0.5) / CAMPIONI as f32) * scala;
                    let py = (y as f32 + (sy as f32 + 0.5) / CAMPIONI as f32) * scala;
                    let colore = colore_in(px, py).or(fondo_pieno.then_some(FONDO.colore));
                    if let Some(colore) = colore {
                        coperti += 1;
                        for c in 0..3 {
                            somma[c] += colore[c] as u32;
                        }
                    }
                }
            }
            let i = ((y * lato + x) * 4) as usize;
            if coperti > 0 {
                for c in 0..3 {
                    rgba[i + c] = (somma[c] / coperti) as u8;
                }
                rgba[i + 3] = (coperti * 255 / (CAMPIONI * CAMPIONI)) as u8;
            }
        }
    }
    rgba
}

/// Colore della forma piu' vicina che copre il punto, se ce n'e' una.
fn colore_in(x: f32, y: f32) -> Option<[u8; 3]> {
    std::iter::once(&FONDO).chain(FORME.iter()).rev().find(|f| dentro(f, x, y)).map(|f| f.colore)
}

fn dentro(f: &Forma, x: f32, y: f32) -> bool {
    let ((x0, y0), (x1, y1), r) = (f.da, f.a, f.raggio);
    if x < x0 || x > x1 || y < y0 || y > y1 {
        return false;
    }
    let cx = x.clamp(x0 + r, x1 - r);
    let cy = y.clamp(y0 + r, y1 - r);
    (x - cx).powi(2) + (y - cy).powi(2) <= r * r
}

/// Un file `.ico` di Windows con l'icona a piu' grandezze. Ogni immagine e'
/// salvata come bitmap a 32 bit (BGRA, dal basso verso l'alto) seguita dalla
/// maschera a 1 bit che i vecchi programmi si aspettano: niente compressione,
/// quindi niente librerie.
pub fn file_ico(lati: &[u32]) -> Vec<u8> {
    let immagini: Vec<Vec<u8>> = lati.iter().map(|&lato| bitmap_ico(lato)).collect();
    let mut ico = Vec::new();
    ico.extend_from_slice(&[0, 0, 1, 0]);
    ico.extend_from_slice(&(lati.len() as u16).to_le_bytes());
    let mut posizione = 6 + 16 * lati.len() as u32;
    for (lato, immagine) in lati.iter().zip(&immagini) {
        // 256 si scrive 0: il campo e' di un solo byte.
        let misura = if *lato >= 256 { 0 } else { *lato as u8 };
        ico.extend_from_slice(&[misura, misura, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(immagine.len() as u32).to_le_bytes());
        ico.extend_from_slice(&posizione.to_le_bytes());
        posizione += immagine.len() as u32;
    }
    for immagine in immagini {
        ico.extend_from_slice(&immagine);
    }
    ico
}

fn bitmap_ico(lato: u32) -> Vec<u8> {
    let rgba = disegna(lato);
    let riga_maschera = lato.div_ceil(32) * 4;
    let mut b = Vec::new();
    // BITMAPINFOHEADER: l'altezza conta due volte (colori + maschera).
    b.extend_from_slice(&40u32.to_le_bytes());
    b.extend_from_slice(&(lato as i32).to_le_bytes());
    b.extend_from_slice(&((lato * 2) as i32).to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&32u16.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&(lato * lato * 4 + riga_maschera * lato).to_le_bytes());
    b.extend_from_slice(&[0; 16]);
    for y in (0..lato).rev() {
        for x in 0..lato {
            let i = ((y * lato + x) * 4) as usize;
            b.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
        }
    }
    for y in (0..lato).rev() {
        let mut riga = vec![0u8; riga_maschera as usize];
        for x in 0..lato {
            if rgba[((y * lato + x) * 4 + 3) as usize] == 0 {
                riga[(x / 8) as usize] |= 0x80 >> (x % 8);
            }
        }
        b.extend_from_slice(&riga);
    }
    b
}
