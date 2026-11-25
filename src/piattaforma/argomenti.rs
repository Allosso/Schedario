//! Argomenti facoltativi della riga di comando.

use std::path::PathBuf;

pub const TESTO_AIUTO: &str = "\
Schedario - note con schede, widget fluttuanti e rete locale

Uso: schedario [opzioni]

  --notes-dir <cartella>   Apre direttamente questa cartella di note
  --data-dir <cartella>    Usa questa cartella per le preferenze del programma
  --library                Mostra sempre la finestra principale all'avvio
  --help                   Mostra questo aiuto";

#[derive(Debug, Default, Clone)]
pub struct Argomenti {
    pub cartella_note: Option<PathBuf>,
    pub cartella_dati: Option<PathBuf>,
    pub mostra_finestra: bool,
    pub aiuto: bool,
}

impl Argomenti {
    pub fn da_riga_di_comando() -> Result<Self, String> {
        Self::da_elenco(std::env::args_os().skip(1).map(|a| a.to_string_lossy().into_owned()))
    }

    pub fn da_elenco(elenco: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut risultato = Self::default();
        let mut elenco = elenco.into_iter();
        while let Some(argomento) = elenco.next() {
            match argomento.as_str() {
                "--notes-dir" => {
                    risultato.cartella_note = Some(valore(&mut elenco, &argomento)?.into());
                }
                "--data-dir" => {
                    risultato.cartella_dati = Some(valore(&mut elenco, &argomento)?.into());
                }
                "--library" => risultato.mostra_finestra = true,
                "--help" | "-h" => risultato.aiuto = true,
                altro => return Err(format!("argomento sconosciuto: {altro}\n\n{TESTO_AIUTO}")),
            }
        }
        Ok(risultato)
    }
}

fn valore(elenco: &mut impl Iterator<Item = String>, nome: &str) -> Result<String, String> {
    elenco
        .next()
        .ok_or_else(|| format!("{nome} richiede una cartella"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn legge_le_opzioni() {
        let a = Argomenti::da_elenco(
            ["--notes-dir", "x", "--library"].map(String::from),
        )
        .unwrap();
        assert_eq!(a.cartella_note, Some(PathBuf::from("x")));
        assert!(a.mostra_finestra);
        assert!(Argomenti::da_elenco(["--boh".to_owned()]).is_err());
    }
}
