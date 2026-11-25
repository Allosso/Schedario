//! Avvio automatico all'accesso dell'utente, con il meccanismo standard di
//! ciascun sistema. Nessun privilegio di amministratore richiesto.
//!
//! - Windows: valore `Schedario` nella chiave del registro
//!   `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`;
//! - Linux:   file `~/.config/autostart/schedario.desktop` (standard XDG);
//! - macOS:   file `~/Library/LaunchAgents/io.github.allosso.schedario.plist`.

use std::path::PathBuf;

/// Percorso dell'eseguibile in uso, da registrare per l'avvio.
fn eseguibile() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("percorso del programma non disponibile: {e}"))
}

pub fn attivo() -> bool {
    sistema::attivo()
}

pub fn imposta(attivo: bool) -> Result<(), String> {
    if attivo { sistema::attiva(&eseguibile()?) } else { sistema::disattiva() }
}

#[cfg(target_os = "windows")]
mod sistema {
    use std::os::windows::process::CommandExt;
    use std::path::Path;
    use std::process::Command;

    const CHIAVE: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const NOME: &str = "Schedario";
    /// Evita che compaia una finestra nera per un istante.
    const SENZA_FINESTRA: u32 = 0x0800_0000;

    fn reg(argomenti: &[&str]) -> Result<bool, String> {
        Command::new("reg")
            .args(argomenti)
            .creation_flags(SENZA_FINESTRA)
            .output()
            .map(|uscita| uscita.status.success())
            .map_err(|e| format!("impossibile usare il registro di sistema: {e}"))
    }

    pub fn attivo_in(chiave: &str, nome: &str) -> bool {
        reg(&["query", chiave, "/v", nome]).unwrap_or(false)
    }

    pub fn scrivi_in(chiave: &str, nome: &str, valore: &str) -> Result<(), String> {
        match reg(&["add", chiave, "/v", nome, "/t", "REG_SZ", "/d", valore, "/f"])? {
            true => Ok(()),
            false => Err("il registro di sistema non ha accettato la modifica".into()),
        }
    }

    pub fn cancella_in(chiave: &str, nome: &str) -> Result<(), String> {
        if attivo_in(chiave, nome) {
            reg(&["delete", chiave, "/v", nome, "/f"])?;
        }
        Ok(())
    }

    pub fn attivo() -> bool {
        attivo_in(CHIAVE, NOME)
    }

    pub fn attiva(eseguibile: &Path) -> Result<(), String> {
        scrivi_in(CHIAVE, NOME, &format!("\"{}\"", eseguibile.display()))
    }

    pub fn disattiva() -> Result<(), String> {
        cancella_in(CHIAVE, NOME)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        /// Usa una chiave di prova: il vero avvio automatico non viene toccato.
        #[test]
        fn scrive_legge_e_cancella_nel_registro() {
            let chiave = r"HKCU\Software\SchedarioProva";
            let nome = "Prova";
            cancella_in(chiave, nome).unwrap();
            assert!(!attivo_in(chiave, nome));
            scrivi_in(chiave, nome, r#""C:\percorso con spazi\schedario.exe""#).unwrap();
            assert!(attivo_in(chiave, nome));
            cancella_in(chiave, nome).unwrap();
            assert!(!attivo_in(chiave, nome));
            let _ = reg(&["delete", chiave, "/f"]);
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod sistema {
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::piattaforma::percorsi;

    fn file_avvio() -> Option<PathBuf> {
        let home = percorsi::cartella_utente()?;
        if cfg!(target_os = "macos") {
            Some(home.join("Library/LaunchAgents/io.github.allosso.schedario.plist"))
        } else {
            let configurazione = std::env::var_os("XDG_CONFIG_HOME")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            Some(configurazione.join("autostart/schedario.desktop"))
        }
    }

    fn contenuto(eseguibile: &Path) -> String {
        let percorso = eseguibile.display().to_string();
        if cfg!(target_os = "macos") {
            let percorso = percorso.replace('&', "&amp;").replace('<', "&lt;");
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                 <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
                 <plist version=\"1.0\"><dict>\n\
                 <key>Label</key><string>io.github.allosso.schedario</string>\n\
                 <key>ProgramArguments</key><array><string>{percorso}</string></array>\n\
                 <key>RunAtLoad</key><true/>\n\
                 </dict></plist>\n"
            )
        } else {
            let percorso = percorso.replace('\\', "\\\\").replace('"', "\\\"");
            format!(
                "[Desktop Entry]\nType=Application\nName=Schedario\nExec=\"{percorso}\"\n\
                 X-GNOME-Autostart-enabled=true\nTerminal=false\n"
            )
        }
    }

    pub fn attivo() -> bool {
        file_avvio().is_some_and(|f| f.exists())
    }

    pub fn attiva(eseguibile: &Path) -> Result<(), String> {
        let file = file_avvio().ok_or("cartella personale non trovata")?;
        crate::archivio::scrittura_atomica::scrivi(&file, contenuto(eseguibile).as_bytes())
    }

    pub fn disattiva() -> Result<(), String> {
        match file_avvio() {
            Some(file) if file.exists() => {
                fs::remove_file(&file).map_err(|e| format!("impossibile rimuovere {}: {e}", file.display()))
            }
            _ => Ok(()),
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
mod sistema {
    pub fn attivo() -> bool {
        false
    }
    pub fn attiva(_: &std::path::Path) -> Result<(), String> {
        Err("avvio automatico non disponibile su questo sistema".into())
    }
    pub fn disattiva() -> Result<(), String> {
        Ok(())
    }
}
