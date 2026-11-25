//! Su Windows mette l'icona dentro l'eseguibile, cosi' Esplora file, il menu
//! Start e i collegamenti mostrano lo schedario invece dell'icona generica.
//!
//! Il file `.ico` si ricava dal disegno in `src/interfaccia/icona_forme.rs`
//! (nessuna immagine nel progetto); `windres`, che fa parte di MinGW come il
//! linker, lo trasforma in una risorsa da collegare all'eseguibile. Senza
//! `windres` la compilazione continua lo stesso, con un avviso.

use std::path::PathBuf;
use std::process::Command;

#[allow(dead_code)]
#[path = "src/interfaccia/icona_forme.rs"]
mod icona_forme;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/interfaccia/icona_forme.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let uscita = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let ico = uscita.join("schedario.ico");
    std::fs::write(&ico, icona_forme::file_ico(&[16, 20, 24, 32, 40, 48, 64, 256])).expect("scrittura dell'icona");
    let rc = uscita.join("icona.rc");
    std::fs::write(&rc, format!("1 ICON \"{}\"\n", ico.display().to_string().replace('\\', "/")))
        .expect("scrittura di icona.rc");
    let risorsa = uscita.join("icona.o");

    let windres = std::env::var("WINDRES").unwrap_or_else(|_| "windres".to_owned());
    match Command::new(&windres).arg(&rc).args(["-O", "coff", "-o"]).arg(&risorsa).status() {
        // Solo per l'eseguibile: la libreria e i test non ne hanno bisogno.
        Ok(stato) if stato.success() => println!("cargo:rustc-link-arg-bins={}", risorsa.display()),
        Ok(stato) => println!("cargo:warning=windres non riuscito ({stato}): eseguibile senza icona"),
        Err(errore) => println!("cargo:warning=windres non trovato ({errore}): eseguibile senza icona"),
    }
}
