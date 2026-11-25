//! Comandi sulle note e salvataggio differito.
//!
//! Ogni modifica aggiorna subito la nota in memoria (editor e widget la
//! vedono insieme) e programma il salvataggio dopo 450 ms di pausa.
//! Se la scrittura fallisce le modifiche restano in memoria e si riprova.

use std::time::{Duration, Instant};

use super::stato::{Applicazione, StatoSalvataggio};
use crate::modello::Nota;

const PAUSA_SALVATAGGIO: Duration = Duration::from_millis(450);
const NUOVO_TENTATIVO: Duration = Duration::from_secs(3);
/// Per quanto tempo resta l'avviso "Nota spostata nel cestino · Annulla".
pub const DURATA_ANNULLA: Duration = Duration::from_secs(10);

impl Applicazione {
    /// Nota visibile (esclusi i segnaposto delle note eliminate).
    pub fn nota(&self, id: &str) -> Option<&Nota> {
        self.cartella.as_ref()?.note.get(id).filter(|n| n.visibile())
    }

    pub fn nota_mut(&mut self, id: &str) -> Option<&mut Nota> {
        self.cartella.as_mut()?.note.get_mut(id)
    }

    pub fn nuova_nota(&mut self) {
        let Some(cartella) = self.cartella.as_mut() else { return };
        let nota = Nota::nuova(&self.preferenze.id_dispositivo);
        let id = nota.id.clone();
        cartella.note.insert(id.clone(), nota);
        self.interfaccia.ricerca.clear();
        self.interfaccia.selezionata = Some(id.clone());
        self.interfaccia.fuoco_titolo = true;
        self.mostra_finestra();
        self.registra_modifica(&id);
        self.salva_ora();
    }

    /// Da chiamare dopo aver cambiato titolo o testo di una nota.
    pub fn registra_modifica(&mut self, id: &str) {
        let dispositivo = self.preferenze.id_dispositivo.clone();
        let Some(nota) = self.nota_mut(id) else { return };
        nota.segna_modificata(&dispositivo);
        self.ultima_digitazione.insert(id.to_owned(), Instant::now());
        self.in_sospeso.insert(id.to_owned());
        self.modifiche_da_condividere = true;
        self.salvataggio = StatoSalvataggio::InCorso;
        self.scadenza_salvataggio = Some(Instant::now() + PAUSA_SALVATAGGIO);
    }

    pub fn rinomina(&mut self, id: &str, titolo: &str) {
        let titolo = titolo.replace(['\r', '\n'], " ");
        let Some(nota) = self.nota_mut(id) else { return };
        if nota.titolo != titolo {
            nota.titolo = titolo;
            self.registra_modifica(id);
        }
    }

    pub fn conferma_rinomina(&mut self) {
        if let Some((id, testo)) = self.interfaccia.rinomina.take() {
            self.rinomina(&id, &testo);
        }
    }

    pub fn sposta_nel_cestino(&mut self, id: &str) {
        let attive: Vec<String> = self.note_attive_id();
        let Some(nota) = self.nota_mut(id) else { return };
        if nota.nel_cestino {
            return;
        }
        nota.nel_cestino = true;
        self.registra_modifica(id);
        self.chiudi_widget(id);
        self.cestinata = Some((id.to_owned(), Instant::now()));
        // Seleziona la scheda vicina, come quando si chiude una scheda del browser.
        if self.interfaccia.selezionata.as_deref() == Some(id) {
            let posizione = attive.iter().position(|a| a == id).unwrap_or(0);
            let rimaste: Vec<&String> = attive.iter().filter(|a| *a != id).collect();
            self.interfaccia.selezionata = rimaste
                .get(posizione.min(rimaste.len().saturating_sub(1)))
                .map(|s| (*s).clone());
        }
        self.salva_ora();
    }

    /// La nota appena spostata nel cestino, se l'avviso "Annulla" e' ancora valido.
    pub fn cancellazione_annullabile(&self) -> Option<&str> {
        let (id, quando) = self.cestinata.as_ref()?;
        let ancora_nel_cestino = self.nota(id).is_some_and(|n| n.nel_cestino);
        (quando.elapsed() < DURATA_ANNULLA && ancora_nel_cestino).then_some(id.as_str())
    }

    /// Quanto resta (da 1 a 0) per annullare lo spostamento nel cestino.
    pub fn annulla_cestino_rimasto(&self) -> Option<f32> {
        self.cancellazione_annullabile()?;
        let (_, quando) = self.cestinata.as_ref()?;
        Some((1.0 - quando.elapsed().as_secs_f32() / DURATA_ANNULLA.as_secs_f32()).clamp(0.0, 1.0))
    }

    /// "Annulla" dopo Cancella: la nota torna al suo posto.
    pub fn annulla_cancellazione(&mut self) {
        if let Some(id) = self.cancellazione_annullabile().map(str::to_owned) {
            self.ripristina(&id);
        }
        self.cestinata = None;
    }

    pub fn alterna_fissata(&mut self, id: &str) {
        let Some(nota) = self.nota_mut(id).filter(|n| n.visibile() && !n.nel_cestino) else { return };
        nota.fissata = !nota.fissata;
        self.registra_modifica(id);
        self.salva_ora();
        // Una nota appena fissata va in fondo al gruppo delle fissate; una tolta
        // dalle fissate va all'inizio delle altre: in entrambi i casi al confine.
        let mut ordine = self.note_attive_id();
        ordine.retain(|a| a != id);
        let fissate = ordine.iter().filter(|a| self.nota(a).is_some_and(|n| n.fissata)).count();
        ordine.insert(fissate, id.to_owned());
        self.interfaccia.ordine_schede = ordine;
        self.salva_sessione();
    }

    /// Le schede nell'ordine mostrato: fissate prima, poi l'ordine scelto
    /// trascinandole, poi per creazione.
    pub fn note_ordinate(&self) -> Vec<&Nota> {
        self.cartella
            .as_ref()
            .map(|c| crate::modello::ordine::ordina(c.note_attive(), &self.interfaccia.ordine_schede))
            .unwrap_or_default()
    }

    /// La scheda accanto a `id` (`verso` -1 prima, +1 dopo) tra quelle mostrate,
    /// cioe' filtrate dalla ricerca in corso. `None` alle estremita'. Per una
    /// nota del cestino, la vicina tra quelle del cestino (nell'ordine della
    /// pagina Cestino, senza quelle appena eliminate).
    pub fn nota_vicina(&self, id: &str, verso: isize) -> Option<String> {
        let ricerca = self.ricerca_per(id);
        let nel_cestino = self.nota(id).is_some_and(|n| n.nel_cestino);
        let elenco = if nel_cestino { self.note_cestino_visibili() } else { self.note_ordinate() };
        let schede: Vec<&Nota> = elenco.into_iter().filter(|n| n.corrisponde(ricerca)).collect();
        let qui = schede.iter().position(|n| n.id == id)?;
        let la = qui.checked_add_signed(verso)?;
        schede.get(la).map(|n| n.id.clone())
    }

    /// La ricerca che vale per la nota `id`: quella del Cestino per una nota
    /// del cestino, altrimenti quella delle note. Filtra le note accanto ed
    /// evidenzia le parole nel testo.
    pub fn ricerca_per(&self, id: &str) -> &str {
        if self.nota(id).is_some_and(|n| n.nel_cestino) {
            &self.interfaccia.ricerca_cestino
        } else {
            &self.interfaccia.ricerca
        }
    }

    /// Sfogliando oltre l'ultima scheda si trova una pagina bianca (una nota
    /// nuova): vale per l'ultima nota attiva, senza una ricerca in corso.
    pub fn pagina_bianca_dopo(&self, id: &str) -> bool {
        self.interfaccia.ricerca.trim().is_empty()
            && self.nota(id).is_some_and(|n| !n.nel_cestino)
            && self.nota_vicina(id, 1).is_none()
    }

    /// Sposta la scheda `id` alla posizione `a` (tra tutte le schede), restando
    /// nel suo gruppo. L'ordine vale solo su questo computer: la nota non cambia.
    pub fn sposta_scheda(&mut self, id: &str, a: usize) {
        use crate::modello::ordine;
        let note = self.note_ordinate();
        let Some(da) = note.iter().position(|n| n.id == id) else { return };
        let fissate = ordine::fissate_in_testa(&note);
        let a = ordine::limita_al_gruppo(da, a, fissate, note.len());
        if a == da {
            return;
        }
        let ids: Vec<String> = note.iter().map(|n| n.id.clone()).collect();
        self.interfaccia.ordine_schede = ordine::sposta(&ids, da, a);
        self.salva_sessione();
    }

    /// Ctrl+Shift+frecce: sposta la scheda di un posto a sinistra (-1) o a destra (+1).
    pub fn sposta_scheda_di(&mut self, id: &str, passo: isize) {
        let Some(da) = self.note_attive_id().iter().position(|n| n == id) else { return };
        let a = da.saturating_add_signed(passo);
        self.sposta_scheda(id, a);
    }

    /// Usa l'ordine delle schede di un altro computer (vedi `modello::ordine::adotta`).
    /// L'ordine di prima resta per qualche secondo, per poter annullare.
    pub fn adotta_ordine(&mut self, remoto: &[String]) {
        let attuale = self.note_attive_id();
        self.ordine_precedente = Some((self.interfaccia.ordine_schede.clone(), Instant::now()));
        self.interfaccia.ordine_schede = crate::modello::ordine::adotta(&attuale, remoto);
        self.salva_sessione();
    }

    /// `true` finche' si puo' annullare l'ordine preso da un altro computer.
    pub fn ordine_annullabile(&self) -> bool {
        self.ordine_precedente.as_ref().is_some_and(|(_, quando)| quando.elapsed() < DURATA_ANNULLA)
    }

    pub fn annulla_ordine(&mut self) {
        if let Some((precedente, _)) = self.ordine_precedente.take() {
            self.interfaccia.ordine_schede = precedente;
            self.salva_sessione();
        }
    }

    /// Nota rapida (scorciatoia globale): una nota nuova aperta subito come
    /// widget, con il cursore nel testo, senza mostrare la finestra principale.
    pub fn nota_rapida(&mut self) {
        let Some(cartella) = self.cartella.as_mut() else { return };
        let nota = Nota::nuova(&self.preferenze.id_dispositivo);
        let id = nota.id.clone();
        cartella.note.insert(id.clone(), nota);
        self.interfaccia.selezionata = Some(id.clone());
        self.registra_modifica(&id);
        self.salva_ora();
        self.apri_widget(&id);
        if let Some(widget) = self.widget.iter_mut().find(|w| w.stato.id_nota == id) {
            widget.fuoco_testo = 30;
        }
    }

    /// Chiude il widget; se la nota e' rimasta completamente vuota (una nota
    /// rapida aperta e poi lasciata perdere) la elimina, cosi' non restano
    /// schede "Senza titolo" inutili.
    pub fn chiudi_widget_e_scarta_se_vuota(&mut self, id: &str) {
        self.chiudi_widget(id);
        self.scarta_se_vuota(id);
    }

    /// Elimina la nota se e' completamente vuota (niente titolo, niente
    /// testo, nemmeno spazi): per esempio una nota creata con "+" e lasciata
    /// senza scrivere. Non va nel cestino, perche' non c'e' niente da
    /// recuperare, e sparisce anche dagli altri dispositivi collegati.
    /// Restituisce `true` se l'ha eliminata.
    pub fn scarta_se_vuota(&mut self, id: &str) -> bool {
        let vuota = self
            .nota(id)
            .is_some_and(|n| !n.nel_cestino && n.titolo.trim().is_empty() && n.testo.trim().is_empty());
        if vuota {
            if let Some(nota) = self.nota_mut(id) {
                nota.nel_cestino = true;
            }
            self.elimina_definitivamente(id);
            if self.interfaccia.selezionata.is_none() {
                self.interfaccia.selezionata = self.note_attive_id().into_iter().next();
            }
        }
        vuota
    }

    /// Versioni precedenti della nota, dalla piu' recente. Quella uguale al
    /// testo attuale non viene mostrata: non ci sarebbe niente da ripristinare.
    pub fn cronologia(&self, id: &str) -> Vec<crate::archivio::storico::Versione> {
        let mut versioni = self.cartella.as_ref().map(|c| c.cronologia(id)).unwrap_or_default();
        if let Some(nota) = self.nota(id) {
            versioni.retain(|v| v.titolo != nota.titolo || v.testo != nota.testo);
        }
        versioni.reverse();
        versioni
    }

    /// Apre la finestra della cronologia per la nota.
    pub fn apri_cronologia(&mut self, id: &str) {
        // Le modifiche in sospeso vanno scritte prima di confrontare le versioni.
        self.salva_ora();
        let versioni = self.cronologia(id);
        self.avviso = Some(super::Avviso::Cronologia { id: id.to_owned(), versioni, scelta: 0 });
    }

    /// Riporta la nota a una versione precedente. Lo stato attuale entra prima
    /// nella cronologia, cosi' anche il ripristino si puo' annullare. E' una
    /// modifica come le altre: arriva anche agli altri computer.
    pub fn ripristina_versione(&mut self, id: &str, versione: &crate::archivio::storico::Versione) {
        if let Some(cartella) = self.cartella.as_mut() {
            cartella.registra_versione(id, true);
        }
        let Some(nota) = self.nota_mut(id).filter(|n| n.visibile()) else { return };
        nota.titolo = versione.titolo.clone();
        nota.testo = versione.testo.clone();
        self.registra_modifica(id);
        self.salva_ora();
    }

    /// Cambia la dimensione del testo delle note, moltiplicandola per `fattore`
    /// (1.0 = nessun cambio, `None` = torna alla dimensione predefinita).
    pub fn cambia_dimensione_testo(&mut self, fattore: Option<f32>) {
        use crate::archivio::preferenze::{
            DIMENSIONE_TESTO_MASSIMA, DIMENSIONE_TESTO_MINIMA, DIMENSIONE_TESTO_PREDEFINITA,
        };
        let nuova = match fattore {
            Some(f) => (self.preferenze.dimensione_testo * f).clamp(DIMENSIONE_TESTO_MINIMA, DIMENSIONE_TESTO_MASSIMA),
            None => DIMENSIONE_TESTO_PREDEFINITA,
        };
        // Andata e ritorno con Ctrl + e Ctrl - riporta esattamente al 100%.
        let nuova = if (nuova - DIMENSIONE_TESTO_PREDEFINITA).abs() < 0.05 { DIMENSIONE_TESTO_PREDEFINITA } else { nuova };
        if (nuova - self.preferenze.dimensione_testo).abs() > f32::EPSILON {
            self.preferenze.dimensione_testo = nuova;
            self.salva_preferenze_dopo();
        }
    }

    pub fn ripristina(&mut self, id: &str) {
        let Some(nota) = self.nota_mut(id).filter(|n| n.visibile()) else { return };
        if !nota.nel_cestino {
            return;
        }
        nota.nel_cestino = false;
        self.registra_modifica(id);
        self.interfaccia.selezionata = Some(id.to_owned());
        self.interfaccia.ricerca.clear();
        self.salva_ora();
    }

    /// Eliminazione definitiva di una nota del cestino. Resta un segnaposto
    /// vuoto, perche' anche gli altri computer collegati la eliminino.
    pub fn elimina_definitivamente(&mut self, id: &str) {
        let dispositivo = self.preferenze.id_dispositivo.clone();
        let Some(nota) = self.nota_mut(id).filter(|n| n.nel_cestino && n.visibile()) else { return };
        nota.elimina(&dispositivo);
        self.in_sospeso.insert(id.to_owned());
        self.modifiche_da_condividere = true;
        self.chiudi_widget(id);
        if self.interfaccia.selezionata.as_deref() == Some(id) {
            self.interfaccia.selezionata = None;
        }
        self.salva_ora();
    }

    /// Telefono: elimina definitivamente queste note del cestino, ma solo tra
    /// `DURATA_ANNULLA`: intanto spariscono dal cestino e "Annulla" le rimette
    /// li'. Un'eliminazione ancora in attesa si conferma subito.
    pub fn elimina_con_annulla(&mut self, ids: Vec<String>) {
        self.conferma_eliminazione();
        let ids: Vec<String> = ids.into_iter().filter(|id| self.nota(id).is_some_and(|n| n.nel_cestino && n.visibile())).collect();
        if ids.is_empty() {
            return;
        }
        if self.interfaccia.selezionata.as_ref().is_some_and(|s| ids.contains(s)) {
            self.interfaccia.selezionata = None;
        }
        self.eliminazione = Some((ids, Instant::now()));
    }

    /// Le note in attesa di eliminazione e quanto resta (da 1 a 0) per annullare.
    pub fn eliminazione_annullabile(&self) -> Option<(&[String], f32)> {
        let (ids, quando) = self.eliminazione.as_ref()?;
        let rimasto = 1.0 - quando.elapsed().as_secs_f32() / DURATA_ANNULLA.as_secs_f32();
        (rimasto > 0.0).then_some((ids.as_slice(), rimasto.clamp(0.0, 1.0)))
    }

    /// "Annulla" dopo Elimina definitivamente: le note restano nel cestino.
    pub fn annulla_eliminazione(&mut self) {
        self.eliminazione = None;
    }

    /// In attesa di eliminazione (nascosta nel cestino)?
    pub fn in_eliminazione(&self, id: &str) -> bool {
        self.eliminazione.as_ref().is_some_and(|(ids, _)| ids.iter().any(|a| a == id))
    }

    /// Elimina davvero le note in attesa (scaduto "Annulla", o l'app va in
    /// secondo piano).
    pub fn conferma_eliminazione(&mut self) {
        if let Some((ids, _)) = self.eliminazione.take() {
            for id in ids {
                self.elimina_definitivamente(&id);
            }
        }
    }

    /// Le note del cestino che si vedono (non quelle in attesa di eliminazione).
    pub fn note_cestino_visibili(&self) -> Vec<&Nota> {
        let Some(cartella) = &self.cartella else { return Vec::new() };
        cartella.note_cestino().into_iter().filter(|n| !self.in_eliminazione(&n.id)).collect()
    }

    pub fn svuota_cestino(&mut self) {
        let nel_cestino: Vec<String> = self
            .cartella
            .as_ref()
            .map(|c| c.note_cestino().into_iter().map(|n| n.id.clone()).collect())
            .unwrap_or_default();
        for id in nel_cestino {
            self.elimina_definitivamente(&id);
        }
    }

    /// Identificativi delle schede, nell'ordine mostrato.
    pub fn note_attive_id(&self) -> Vec<String> {
        self.note_ordinate().into_iter().map(|n| n.id.clone()).collect()
    }

    /// Scrive subito le note in sospeso. `true` se tutto e' salvato.
    pub fn salva_ora(&mut self) -> bool {
        self.scadenza_salvataggio = None;
        let Some(cartella) = self.cartella.as_mut() else { return true };
        if self.in_sospeso.is_empty() {
            self.salvataggio = StatoSalvataggio::Salvato;
            return true;
        }
        let id: Vec<String> = self.in_sospeso.iter().cloned().collect();
        match cartella.salva(&id) {
            Ok(()) => {
                self.in_sospeso.clear();
                self.modifiche_salvate += 1;
                self.salvataggio = StatoSalvataggio::Salvato;
                self.condividi_modifiche();
                true
            }
            Err(errore) => {
                self.salvataggio = StatoSalvataggio::Errore(errore);
                self.scadenza_salvataggio = Some(Instant::now() + NUOVO_TENTATIVO);
                false
            }
        }
    }
}
