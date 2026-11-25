// In release su Windows non apre la console nera accanto alla finestra.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(errore) = schedario::avvia() {
        eprintln!("Schedario: {errore}");
        std::process::exit(1);
    }
}
