//! Cartelle standard di ciascun sistema operativo.
//!
//! - Windows: `%APPDATA%\Schedario`
//! - macOS:   `~/Library/Application Support/Schedario`
//! - Linux:   `$XDG_CONFIG_HOME/schedario` oppure `~/.config/schedario`

use std::env;
use std::path::PathBuf;

/// Cartella delle preferenze del programma (non delle note).
pub fn cartella_configurazione() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Some(appdata) = env::var_os("APPDATA") {
            return PathBuf::from(appdata).join(crate::NOME_APP);
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = cartella_utente() {
            return home
                .join("Library")
                .join("Application Support")
                .join(crate::NOME_APP);
        }
    } else {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
            return PathBuf::from(xdg).join("schedario");
        }
        if let Some(home) = cartella_utente() {
            return home.join(".config").join("schedario");
        }
    }
    // Ultima risorsa: accanto al programma.
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("Dati")))
        .unwrap_or_else(|| PathBuf::from("Dati"))
}

/// Cartella usata per le note al primo avvio: `Documenti/Schedario`.
pub fn cartella_note_predefinita() -> PathBuf {
    cartella_documenti().join(crate::NOME_APP)
}

/// La cartella Documenti dell'utente, anche quando il sistema l'ha spostata
/// (per esempio in OneDrive su Windows) o le ha dato un nome tradotto (Linux).
fn cartella_documenti() -> PathBuf {
    if let Some(cartella) = documenti_di_sistema().filter(|p| p.is_dir()) {
        return cartella;
    }
    let base = cartella_utente().unwrap_or_else(|| PathBuf::from("."));
    ["Documents", "Documenti"]
        .iter()
        .map(|nome| base.join(nome))
        .find(|p| p.is_dir())
        .unwrap_or(base)
}

/// Windows: valore `Personal` di `User Shell Folders` nel registro utente.
#[cfg(target_os = "windows")]
fn documenti_di_sistema() -> Option<PathBuf> {
    use std::os::windows::process::CommandExt;
    const SENZA_FINESTRA: u32 = 0x0800_0000;
    let uscita = std::process::Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders",
            "/v",
            "Personal",
        ])
        .creation_flags(SENZA_FINESTRA)
        .output()
        .ok()
        .filter(|u| u.status.success())?;
    let testo = String::from_utf8_lossy(&uscita.stdout);
    let riga = testo.lines().find(|r| r.trim_start().starts_with("Personal"))?;
    let (_, valore) = riga.split_once("REG_")?;
    let (_, valore) = valore.split_once(char::is_whitespace)?;
    Some(PathBuf::from(espandi_variabili(valore.trim())))
}

/// Linux: `XDG_DOCUMENTS_DIR` in `~/.config/user-dirs.dirs` (es. `~/Documenti`).
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn documenti_di_sistema() -> Option<PathBuf> {
    let home = cartella_utente()?;
    let config = env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    let testo = std::fs::read_to_string(config.join("user-dirs.dirs")).ok()?;
    let valore = testo
        .lines()
        .find_map(|r| r.trim().strip_prefix("XDG_DOCUMENTS_DIR="))?
        .trim()
        .trim_matches('"');
    let percorso = match valore.strip_prefix("$HOME") {
        Some(resto) => home.join(resto.trim_start_matches('/')),
        None => PathBuf::from(valore),
    };
    // "$HOME/" da solo vuol dire che la cartella Documenti e' disattivata.
    (percorso != home).then_some(percorso)
}

#[cfg(target_os = "macos")]
fn documenti_di_sistema() -> Option<PathBuf> {
    cartella_utente().map(|home| home.join("Documents"))
}

/// Sostituisce le variabili `%NOME%` con il loro valore (Windows).
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn espandi_variabili(testo: &str) -> String {
    let mut risultato = String::new();
    let mut resto = testo;
    while let Some(inizio) = resto.find('%') {
        risultato.push_str(&resto[..inizio]);
        let dopo = &resto[inizio + 1..];
        match dopo.find('%') {
            Some(fine) => {
                let nome = &dopo[..fine];
                match env::var(nome) {
                    Ok(valore) => risultato.push_str(&valore),
                    Err(_) => {
                        risultato.push('%');
                        risultato.push_str(nome);
                        risultato.push('%');
                    }
                }
                resto = &dopo[fine + 1..];
            }
            None => {
                risultato.push_str(&resto[inizio..]);
                resto = "";
            }
        }
    }
    risultato.push_str(resto);
    risultato
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn espande_le_variabili_note_e_lascia_le_altre() {
        let profilo = env::var("PATH").unwrap();
        assert_eq!(espandi_variabili("%PATH%\\Documenti"), format!("{profilo}\\Documenti"));
        assert_eq!(espandi_variabili("%NON_ESISTE_SCHEDARIO%\\x"), "%NON_ESISTE_SCHEDARIO%\\x");
        assert_eq!(espandi_variabili("senza variabili"), "senza variabili");
        assert_eq!(espandi_variabili("100%"), "100%");
    }

    #[test]
    fn la_cartella_predefinita_finisce_con_schedario() {
        assert!(cartella_note_predefinita().ends_with(crate::NOME_APP));
    }
}

/// Cartella personale dell'utente.
pub fn cartella_utente() -> Option<PathBuf> {
    let variabile = if cfg!(target_os = "windows") { "USERPROFILE" } else { "HOME" };
    env::var_os(variabile)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Nome del computer, usato come nome iniziale del dispositivo in rete.
pub fn nome_computer() -> String {
    // Android: il nome del telefono (per esempio "Galaxy S24").
    #[cfg(target_os = "android")]
    if let Some(nome) = super::android::nome_dispositivo() {
        return nome;
    }
    ["COMPUTERNAME", "HOSTNAME"]
        .iter()
        .find_map(|v| env::var(v).ok().filter(|s| !s.trim().is_empty()))
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "Computer".to_owned())
}
