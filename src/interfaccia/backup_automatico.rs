//! Telefono: backup automatico di tutte le note in un file scelto
//! dall'utente, fuori dall'app (Download, scheda SD, la cartella di un
//! servizio cloud...). Le note dell'app stanno nella sua memoria privata, che
//! Android cancella disinstallandola: il file invece resta, e con "Importa
//! backup" le note tornano (anche su un altro telefono o in un'app
//! reinstallata con un'altra firma).
//!
//! Il file si sceglie una volta sola (Impostazioni → Cartella e backup, o
//! l'invito alla prima apertura); Android da' un permesso permanente per
//! riscriverlo (`MainActivity.scegliBackupAutomatico`). Dopo ogni modifica
//! salvata si aspetta `ATTESA` (per non riscriverlo a ogni parola) e lo si
//! riscrive; passando a un'altra app lo si riscrive subito, perche' Android
//! potrebbe chiudere l'app senza avvisare.

use std::time::{Duration, Instant};

use crate::applicazione::Applicazione;
use crate::archivio::preferenze::BackupAutomatico;
use crate::piattaforma::android;

/// Nome proposto per il file.
pub const NOME_FILE: &str = "Schedario-backup-automatico.json";
/// Pausa dopo l'ultima modifica salvata prima di riscrivere il file.
const ATTESA: Duration = Duration::from_secs(20);

/// Quando riscrivere il file.
#[derive(Default)]
pub struct Pianificazione {
    /// `Applicazione::modifiche_salvate` all'ultimo backup (riuscito o no).
    fatto: Option<u64>,
    scadenza: Option<Instant>,
}

impl Pianificazione {
    /// A ogni fotogramma: se ci sono modifiche nuove, il backup tra `ATTESA`.
    pub fn aggiorna(&mut self, app: &mut Applicazione, ctx: &egui::Context) {
        if app.preferenze.backup_automatico.is_none() || app.cartella.is_none() {
            self.scadenza = None;
            return;
        }
        // All'apertura il file e' gia' aggiornato, a meno che l'ultimo backup
        // non sia mai riuscito.
        let fatto = *self.fatto.get_or_insert_with(|| {
            let mai_riuscito = app.preferenze.backup_automatico.as_ref().is_some_and(|b| b.ultimo.is_none());
            if mai_riuscito { u64::MAX } else { app.modifiche_salvate }
        });
        if fatto == app.modifiche_salvate {
            return;
        }
        let scadenza = *self.scadenza.get_or_insert_with(|| Instant::now() + ATTESA);
        let manca = scadenza.saturating_duration_since(Instant::now());
        if manca.is_zero() {
            self.esegui(app);
        } else {
            ctx.request_repaint_after(manca);
        }
    }

    /// Passando a un'altra app: subito, se c'e' qualcosa da salvare.
    pub fn subito_se_serve(&mut self, app: &mut Applicazione) {
        if app.preferenze.backup_automatico.is_some() && self.fatto != Some(app.modifiche_salvate) {
            self.esegui(app);
        }
    }

    /// L'utente ha scelto il file: lo si ricorda e lo si scrive subito.
    pub fn scelto(&mut self, app: &mut Applicazione) {
        let Some((indirizzo, nome)) = android::backup_automatico_scelto() else { return };
        if let Some(prima) = &app.preferenze.backup_automatico {
            if prima.indirizzo != indirizzo {
                android::rilascia_backup_automatico(&prima.indirizzo);
            }
        }
        app.preferenze.backup_automatico = Some(BackupAutomatico { indirizzo, nome, ultimo: None });
        app.salva_preferenze();
        self.esegui(app);
        if app.errore_backup_automatico.is_none() {
            app.avviso = Some(crate::applicazione::Avviso::Info(app.tr.t("backup_automatico_attivato").to_owned()));
        }
    }

    fn esegui(&mut self, app: &mut Applicazione) {
        self.scadenza = None;
        self.fatto = Some(app.modifiche_salvate);
        let Some(indirizzo) = app.preferenze.backup_automatico.as_ref().map(|b| b.indirizzo.clone()) else { return };
        let percorso = android::file_in_uscita("backup-automatico.json");
        if let Some(cartella) = percorso.parent() {
            let _ = std::fs::create_dir_all(cartella);
        }
        let esito = app.scrivi_backup(&percorso).and_then(|()| {
            if android::scrivi_backup_automatico(&indirizzo, &percorso) {
                Ok(())
            } else {
                Err(app.tr.t("backup_automatico_non_riuscito").to_owned())
            }
        });
        let _ = std::fs::remove_file(&percorso);
        match esito {
            Ok(()) => {
                app.errore_backup_automatico = None;
                if let Some(backup) = app.preferenze.backup_automatico.as_mut() {
                    backup.ultimo = Some(crate::modello::tempo::adesso_ms());
                }
                app.salva_preferenze_dopo();
            }
            Err(errore) => {
                log::warn!("backup automatico non riuscito: {errore}");
                app.errore_backup_automatico = Some(errore);
            }
        }
    }
}

/// Fa scegliere a Android il file del backup automatico.
pub fn attiva() {
    android::scegli_backup_automatico(NOME_FILE);
}

/// Spegne il backup automatico. Il file gia' scritto resta dov'e'.
pub fn disattiva(app: &mut Applicazione) {
    if let Some(backup) = app.preferenze.backup_automatico.take() {
        android::rilascia_backup_automatico(&backup.indirizzo);
    }
    app.errore_backup_automatico = None;
    app.salva_preferenze();
}
