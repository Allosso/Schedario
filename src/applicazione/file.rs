//! Backup, importazione ed esportazione. I percorsi arrivano gia' scelti
//! dall'interfaccia (finestre di dialogo native).

use std::path::Path;

use super::stato::{Applicazione, Avviso};
use crate::archivio::backup;

impl Applicazione {
    fn percorso_protetto(&self, percorso: &Path) -> bool {
        let percorso = std::path::absolute(percorso).unwrap_or_else(|_| percorso.to_path_buf());
        let preferenze = std::path::absolute(self.preferenze.percorso()).unwrap_or_default();
        percorso == preferenze
            || self
                .cartella
                .as_ref()
                .is_some_and(|c| c.percorsi_protetti().contains(&percorso))
    }

    pub fn esporta_backup(&mut self, percorso: &Path) {
        let Some(cartella) = &self.cartella else { return };
        if self.percorso_protetto(percorso) {
            self.avviso = Some(Avviso::Errore(self.tr.t("percorso_protetto").to_owned()));
            return;
        }
        let note = cartella.note.values().collect();
        self.avviso = Some(match backup::esporta(percorso, note) {
            Ok(()) => Avviso::Info(self.tr.t("backup_fatto").to_owned()),
            Err(errore) => Avviso::Errore(errore),
        });
    }

    pub fn importa_backup(&mut self, percorso: &Path) {
        let importate = match backup::importa(percorso, &self.preferenze.id_dispositivo) {
            Ok(note) => note,
            Err(errore) => {
                self.avviso = Some(Avviso::Errore(errore));
                return;
            }
        };
        let Some(cartella) = self.cartella.as_mut() else { return };
        let quante = importate.len();
        for nota in importate {
            self.in_sospeso.insert(nota.id.clone());
            cartella.note.insert(nota.id.clone(), nota);
        }
        self.modifiche_da_condividere = true;
        self.salva_ora();
        self.avviso = Some(Avviso::Info(self.tr.f("importate", &[("n", &quante.to_string())])));
    }

    pub fn esporta_nota(&mut self, id: &str, percorso: &Path) {
        if self.percorso_protetto(percorso) {
            self.avviso = Some(Avviso::Errore(self.tr.t("percorso_protetto").to_owned()));
            return;
        }
        let Some(nota) = self.nota(id) else { return };
        self.avviso = Some(match backup::esporta_testo(percorso, nota) {
            Ok(()) => Avviso::Info(self.tr.t("nota_esportata").to_owned()),
            Err(errore) => Avviso::Errore(errore),
        });
    }
}
