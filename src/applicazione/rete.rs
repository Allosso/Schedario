//! Collegamento tra il coordinatore e il servizio di rete.

use std::sync::Arc;

use super::stato::Applicazione;
use crate::archivio::ImpostazioniRete;
use crate::modello::Nota;
use crate::modello::nota::Relazione;
use crate::rete::fusione::{self, Fusione};
use crate::rete::{self, ConfigurazioneRete, Dispositivo, Evento, ServizioRete};

/// Stato della sincronizzazione mostrato nella barra laterale.
#[derive(Default)]
pub struct StatoRete {
    /// Impostazioni salvate nella cartella di lavoro.
    pub impostazioni: ImpostazioniRete,
    pub servizio: Option<ServizioRete>,
    /// Valori in modifica nel pannello, salvati quando si lascia il campo.
    pub bozza_nome: String,
    pub bozza_codice: String,
    pub bozza_indirizzo: String,
    pub mostra_codice: bool,
    /// Ultimo messaggio informativo (es. "Ricevute 3 note da PC-studio").
    pub messaggio: Option<String>,
    /// Quando e' arrivato `messaggio`: sul computer si vede per qualche secondo.
    pub messaggio_da: Option<std::time::Instant>,
    pub errore: Option<String>,
    /// Funzione per chiedere all'interfaccia di ridisegnarsi.
    pub ridisegna: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Ultimo ordine delle schede consegnato al servizio (per non ripeterlo).
    ordine_consegnato: Vec<String>,
    /// Nome del computer a cui si e' chiesto l'ordine delle schede, finche'
    /// non arriva la risposta.
    pub ordine_in_attesa: Option<String>,
}

impl StatoRete {
    pub fn ferma(&mut self) {
        self.servizio = None;
    }

    pub fn dispositivi(&self) -> Vec<Dispositivo> {
        self.servizio.as_ref().map(ServizioRete::dispositivi).unwrap_or_default()
    }
}

impl Applicazione {
    pub(super) fn carica_impostazioni_rete(&mut self) {
        let Some(cartella) = &self.cartella else { return };
        self.rete.impostazioni = cartella.carica_rete();
        self.rete.bozza_codice = self.rete.impostazioni.codice_gruppo.clone();
        self.rete.bozza_nome = self.preferenze.nome_dispositivo.clone();
        self.rete.messaggio = None;
        self.rete.errore = None;
        self.riavvia_rete();
    }

    /// Avvia (o riavvia) il servizio con le impostazioni correnti.
    pub fn riavvia_rete(&mut self) {
        self.rete.ferma();
        let impostazioni = &self.rete.impostazioni;
        let Some(cartella) = &self.cartella else { return };
        if !impostazioni.attiva || impostazioni.codice_gruppo.trim().chars().count() < rete::LUNGHEZZA_MINIMA_CODICE {
            return;
        }
        let mut configurazione = ConfigurazioneRete::new(
            &self.preferenze.id_dispositivo,
            &self.preferenze.nome_dispositivo,
            &impostazioni.codice_gruppo,
        );
        configurazione.indirizzi_manuali = impostazioni.indirizzi_manuali.clone();
        let note = cartella.note.values().cloned().collect();
        let ridisegna = self.rete.ridisegna.clone();
        match ServizioRete::avvia(configurazione, note, move || {
            if let Some(r) = &ridisegna {
                r();
            }
        }) {
            Ok(servizio) => {
                self.rete.servizio = Some(servizio);
                self.rete.errore = None;
            }
            Err(errore) => {
                self.rete.errore = Some(self.tr.f("rete_errore", &[("errore", &errore)]));
            }
        }
    }

    /// Salva nome e codice scritti nel pannello quando l'utente lascia il campo
    /// (o preme Invio, o Genera). Non serve un pulsante "Applica": se la
    /// sincronizzazione e' attiva riparte subito con i nuovi valori. Un codice
    /// troppo corto non spegne la sincronizzazione in corso: resta quella con il
    /// codice precedente e compare l'errore.
    pub fn conferma_bozze_rete(&mut self) {
        let codice = self.rete.bozza_codice.trim().to_owned();
        let nome = self.rete.bozza_nome.trim().to_owned();
        let codice_cambiato = codice != self.rete.impostazioni.codice_gruppo;
        let nome_cambiato = !nome.is_empty() && nome != self.preferenze.nome_dispositivo;
        if !codice_cambiato && !nome_cambiato {
            return;
        }
        if codice_cambiato && self.rete.impostazioni.attiva && codice.chars().count() < rete::LUNGHEZZA_MINIMA_CODICE {
            self.rete.errore = Some(self.tr.t("rete_codice_corto").to_owned());
            return;
        }
        self.applica_impostazioni_rete(self.rete.impostazioni.attiva);
    }

    /// Accende o spegne la sincronizzazione e salva nome e codice del pannello.
    pub fn applica_impostazioni_rete(&mut self, attiva: bool) {
        let codice = self.rete.bozza_codice.trim().to_owned();
        if attiva && codice.chars().count() < rete::LUNGHEZZA_MINIMA_CODICE {
            self.rete.errore = Some(self.tr.t("rete_codice_corto").to_owned());
            self.rete.impostazioni.attiva = false;
            self.rete.ferma();
            return;
        }
        self.rete.errore = None;
        let nome = self.rete.bozza_nome.trim();
        if !nome.is_empty() && nome != self.preferenze.nome_dispositivo {
            self.preferenze.nome_dispositivo = nome.to_owned();
            self.salva_preferenze();
        }
        self.rete.impostazioni.attiva = attiva;
        self.rete.impostazioni.codice_gruppo = codice;
        self.salva_impostazioni_rete();
        self.riavvia_rete();
    }

    pub fn aggiungi_indirizzo_manuale(&mut self) {
        let indirizzo = self.rete.bozza_indirizzo.trim().to_owned();
        if indirizzo.is_empty() {
            return;
        }
        if rete::servizio::risolvi_indirizzo(&indirizzo).is_none() {
            self.rete.errore = Some(self.tr.f("rete_errore", &[("errore", &indirizzo)]));
            return;
        }
        if !self.rete.impostazioni.indirizzi_manuali.contains(&indirizzo) {
            self.rete.impostazioni.indirizzi_manuali.push(indirizzo);
        }
        self.rete.bozza_indirizzo.clear();
        self.salva_impostazioni_rete();
        self.riavvia_rete();
    }

    pub fn rimuovi_indirizzo_manuale(&mut self, indirizzo: &str) {
        self.rete.impostazioni.indirizzi_manuali.retain(|a| a != indirizzo);
        self.salva_impostazioni_rete();
        self.riavvia_rete();
    }

    fn salva_impostazioni_rete(&mut self) {
        if let Some(cartella) = &self.cartella {
            if let Err(errore) = cartella.salva_rete(&self.rete.impostazioni) {
                self.rete.errore = Some(errore);
            }
        }
    }

    pub fn sincronizza_ora(&mut self) {
        if let Some(servizio) = &self.rete.servizio {
            servizio.sincronizza_ora();
        }
    }

    /// Dopo un salvataggio: consegna le note aggiornate al servizio e, se
    /// le modifiche sono nate qui, chiede di avvisare gli altri computer.
    pub(super) fn condividi_modifiche(&mut self) {
        let (Some(servizio), Some(cartella)) = (&self.rete.servizio, &self.cartella) else {
            self.modifiche_da_condividere = false;
            return;
        };
        servizio.aggiorna_note(cartella.note.values().cloned().collect());
        if std::mem::take(&mut self.modifiche_da_condividere) {
            servizio.sincronizza_ora();
        }
    }

    pub(super) fn gestisci_eventi_rete(&mut self) {
        // Chi chiede l'ordine delle schede riceve sempre quello mostrato ora.
        let ordine = self.note_attive_id();
        let Some(servizio) = &self.rete.servizio else { return };
        if ordine != self.rete.ordine_consegnato {
            servizio.aggiorna_ordine(ordine.clone());
            self.rete.ordine_consegnato = ordine;
        }
        for evento in servizio.eventi() {
            match evento {
                Evento::NoteRicevute { note, da } => self.applica_note_remote(note, &da),
                Evento::Errore(errore) => {
                    self.rete.errore = Some(self.tr.f("rete_errore", &[("errore", &errore)]));
                }
                Evento::OrdineRicevuto { ordine, .. } => {
                    self.rete.ordine_in_attesa = None;
                    self.adotta_ordine(&ordine);
                }
                Evento::OrdineNonRicevuto { errore, da } => {
                    self.rete.ordine_in_attesa = None;
                    let chiave = match errore {
                        rete::sincronizzazione::ErroreOrdine::NonRaggiungibile => "ordine_non_raggiungibile",
                        rete::sincronizzazione::ErroreOrdine::VersionePrecedente => "ordine_versione_precedente",
                    };
                    self.avviso = Some(super::Avviso::Errore(self.tr.f(chiave, &[("da", &da)])));
                }
            }
        }
    }

    /// "Prendi l'ordine delle schede da…", dopo la conferma: la richiesta parte
    /// in background e la risposta arriva con gli eventi di rete.
    pub fn prendi_ordine_da(&mut self, id_dispositivo: &str) {
        let Some(servizio) = &self.rete.servizio else { return };
        let nome = servizio.dispositivi().into_iter().find(|d| d.id == id_dispositivo).map(|d| d.nome);
        servizio.chiedi_ordine(id_dispositivo);
        self.rete.ordine_in_attesa = Some(nome.unwrap_or_default());
    }

    /// Note arrivate dalla rete. Quelle che l'utente sta scrivendo proprio ora
    /// vengono rinviate, cosi' il testo non cambia sotto le dita.
    pub fn applica_note_remote(&mut self, note: Vec<Nota>, da: &str) {
        let (subito, rinviate): (Vec<Nota>, Vec<Nota>) =
            note.into_iter().partition(|n| !self.sta_scrivendo(&n.id));
        for nota in rinviate {
            // Della stessa nota basta tenere la versione piu' avanti.
            match self.note_rinviate.iter_mut().find(|(r, _)| r.id == nota.id) {
                Some(voce) if nota.relazione_con(&voce.0) == Relazione::Segue => *voce = (nota, da.to_owned()),
                Some(_) => {}
                None => self.note_rinviate.push((nota, da.to_owned())),
            }
        }
        self.fondi_note(subito, da);
    }

    pub(super) fn sta_scrivendo(&self, id: &str) -> bool {
        self.ultima_digitazione.get(id).is_some_and(|t| t.elapsed() < PAUSA_DIGITAZIONE)
    }

    pub(super) fn riprova_note_rinviate(&mut self) {
        self.ultima_digitazione.retain(|_, t| t.elapsed() < PAUSA_DIGITAZIONE);
        if self.note_rinviate.is_empty() {
            return;
        }
        let (pronte, ancora): (Vec<_>, Vec<_>) =
            std::mem::take(&mut self.note_rinviate).into_iter().partition(|(n, _)| !self.sta_scrivendo(&n.id));
        self.note_rinviate = ancora;
        for (nota, da) in pronte {
            self.fondi_note(vec![nota], &da);
        }
    }

    fn fondi_note(&mut self, note: Vec<Nota>, da: &str) {
        let Some(cartella) = self.cartella.as_mut() else { return };
        let (mut accettate, mut conflitti) = (0, 0);
        let mut da_chiudere = Vec::new();
        for nota in note {
            let (unita, copia) = match fusione::fondi(cartella.note.get(&nota.id), &nota) {
                Fusione::Ignora => continue,
                Fusione::Accetta(nota) => (nota, None),
                Fusione::Conflitto { unita, copia } => (unita, copia),
            };
            if unita.nel_cestino || unita.eliminata {
                da_chiudere.push(unita.id.clone());
            }
            // Il testo che c'era qui resta nella cronologia di questo computer.
            if cartella.note.get(&unita.id).is_some_and(|locale| !locale.stesso_testo(&unita)) {
                cartella.registra_versione(&unita.id, true);
            }
            self.in_sospeso.insert(unita.id.clone());
            cartella.note.insert(unita.id.clone(), unita);
            accettate += 1;
            // La versione che ha perso diventa una nuova nota (se non esiste gia',
            // perche' l'altro computer l'ha creata e inviata per primo).
            if let Some(copia) = copia.filter(|c| !cartella.note.contains_key(&c.id)) {
                self.in_sospeso.insert(copia.id.clone());
                cartella.note.insert(copia.id.clone(), copia);
                conflitti += 1;
            }
        }
        if accettate == 0 {
            return;
        }
        for id in da_chiudere {
            self.chiudi_widget(&id);
        }
        if self.interfaccia.selezionata.as_deref().is_some_and(|id| self.nota(id).is_none()) {
            self.interfaccia.selezionata = self.note_attive_id().into_iter().next();
        }
        // "1 nota ricevuta", "3 note ricevute": singolare e plurale.
        self.rete.messaggio = Some(match (conflitti, accettate) {
            (1, _) => self.tr.f("rete_conflitto", &[("da", da)]),
            (n, _) if n > 1 => self.tr.f("rete_conflitti", &[("n", &n.to_string()), ("da", da)]),
            (_, 1) => self.tr.f("rete_ricevuta", &[("da", da)]),
            (_, n) => self.tr.f("rete_ricevute", &[("n", &n.to_string()), ("da", da)]),
        });
        self.rete.messaggio_da = Some(std::time::Instant::now());
        // Le note arrivate (e le versioni unite) vanno inoltrate agli altri computer.
        self.modifiche_da_condividere = true;
        self.salva_ora();
    }
}

/// Dopo quanto tempo senza scrivere una nota puo' essere aggiornata dalla rete.
const PAUSA_DIGITAZIONE: std::time::Duration = std::time::Duration::from_secs(2);
