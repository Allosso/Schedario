//! Gli strumenti del server MCP, eseguiti qui come se li usasse l'utente:
//! le modifiche passano da `registra_modifica` (salvataggio, sincronizzazione)
//! e prima di ognuna viene salvata una versione nella cronologia.
//!
//! Regole:
//! - leggere e cercare: sempre, se il server e' acceso;
//! - modificare: solo con "Permetti modifiche" (`Preferenze::mcp_modifiche`);
//! - mai eliminare definitivamente, svuotare il cestino, toccare la rete,
//!   importare backup o cambiare cartella (questi strumenti non esistono);
//! - una nota che l'utente sta scrivendo in questo momento non viene toccata.

use serde_json::{Value, json};

use super::stato::Applicazione;
use crate::mcp::ServizioMcp;
use crate::modello::{Nota, lista, nota};
use crate::piattaforma::ora_locale;
use crate::rete::cifratura;

/// Stato del server MCP mostrato nel menu di destra.
#[derive(Default)]
pub struct StatoMcp {
    pub servizio: Option<ServizioMcp>,
    pub errore: Option<String>,
    pub mostra_codice: bool,
}

impl Applicazione {
    /// Accende o spegne il server secondo le preferenze (all'avvio e a ogni cambio).
    pub fn avvia_mcp(&mut self) {
        self.mcp.servizio = None;
        self.mcp.errore = None;
        if !self.preferenze.mcp_attivo {
            return;
        }
        if self.preferenze.mcp_codice.trim().is_empty() {
            self.preferenze.mcp_codice = cifratura::genera_codice();
            self.salva_preferenze();
        }
        let ridisegna = self.rete.ridisegna.clone();
        let avviso = move || {
            if let Some(r) = &ridisegna {
                r();
            }
        };
        match ServizioMcp::avvia(self.preferenze.mcp_porta, &self.preferenze.mcp_codice, avviso) {
            Ok(servizio) => self.mcp.servizio = Some(servizio),
            Err(errore) => {
                let porta = self.preferenze.mcp_porta.to_string();
                self.mcp.errore = Some(self.tr.f("mcp_errore", &[("porta", &porta), ("errore", &errore)]));
            }
        }
    }

    pub fn imposta_mcp(&mut self, attivo: bool) {
        self.preferenze.mcp_attivo = attivo;
        self.salva_preferenze();
        self.avvia_mcp();
    }

    /// Un codice nuovo: quello vecchio smette di funzionare subito.
    pub fn rigenera_codice_mcp(&mut self) {
        self.preferenze.mcp_codice = cifratura::genera_codice();
        self.salva_preferenze();
        self.avvia_mcp();
    }

    /// Riga da incollare una volta nel terminale per collegare Claude Code.
    pub fn comando_claude_code(&self) -> String {
        format!(
            "claude mcp add --scope user --transport http schedario http://127.0.0.1:{}/mcp --header \"Authorization: Bearer {}\"",
            self.preferenze.mcp_porta, self.preferenze.mcp_codice
        )
    }

    /// Esegue le chiamate arrivate dal server (a ogni fotogramma).
    pub fn gestisci_richieste_mcp(&mut self) {
        let Some(servizio) = &self.mcp.servizio else { return };
        for chiamata in servizio.chiamate() {
            let esito = self.esegui_strumento(&chiamata.nome, &chiamata.argomenti);
            chiamata.rispondi(esito);
        }
    }

    fn esegui_strumento(&mut self, nome: &str, a: &Value) -> Result<String, String> {
        let modifica = crate::mcp::protocollo::STRUMENTI.iter().any(|s| s.nome == nome && s.modifica);
        if modifica && !self.preferenze.mcp_modifiche {
            return Err(self.tr.t("mcp_modifiche_spente").to_owned());
        }
        if self.cartella.is_none() {
            return Err("Nessuna cartella di note aperta.".to_owned());
        }
        // `registra_modifica` segna la nota come "l'utente sta scrivendo": per le
        // modifiche dell'assistente non vale (bloccherebbe la modifica successiva
        // e rinvierebbe le note in arrivo dalla rete). Si torna allo stato di prima.
        let digitazione_prima = self.ultima_digitazione.clone();
        let esito = self.esegui_strumento_dentro(nome, a);
        self.ultima_digitazione.retain(|id, quando| digitazione_prima.get(id) == Some(quando));
        for (id, quando) in digitazione_prima {
            self.ultima_digitazione.entry(id).or_insert(quando);
        }
        esito
    }

    fn esegui_strumento_dentro(&mut self, nome: &str, a: &Value) -> Result<String, String> {
        let testo = |chiave: &str| a.get(chiave).and_then(Value::as_str).map(str::to_owned);
        let vero = |chiave: &str| a.get(chiave).and_then(Value::as_bool);
        let risultato = match nome {
            "stato" => self.mcp_stato(),
            "elenca_note" => self.mcp_elenca(vero("cestino").unwrap_or(false)),
            "leggi_nota" => self.mcp_leggi(&self.nota_esistente(&testo("id"))?),
            "cerca" => self.mcp_cerca(&testo("testo").unwrap_or_default(), vero("cestino").unwrap_or(true)),
            "crea_nota" => self.mcp_crea(&testo("titolo").unwrap_or_default(), &testo("testo").unwrap_or_default()),
            "modifica_nota" => {
                let id = self.nota_modificabile(&testo("id"))?;
                self.mcp_modifica(&id, testo("titolo"), testo("testo"))
            }
            "aggiungi_testo" => {
                let id = self.nota_modificabile(&testo("id"))?;
                let attuale = self.nota(&id).map(|n| n.testo.clone()).unwrap_or_default();
                let aggiunta = testo("testo").unwrap_or_default();
                let separatore = if attuale.is_empty() || attuale.ends_with('\n') { "" } else { "\n" };
                self.mcp_modifica(&id, None, Some(format!("{attuale}{separatore}{aggiunta}")))
            }
            "spunta_voce" => {
                let id = self.nota_modificabile(&testo("id"))?;
                self.mcp_spunta(&id, &testo("voce").unwrap_or_default(), vero("fatta").unwrap_or(true))
            }
            "fissa_nota" => {
                let id = self.nota_modificabile(&testo("id"))?;
                let voluta = vero("fissata").unwrap_or(true);
                if self.nota(&id).is_some_and(|n| n.fissata != voluta) {
                    self.alterna_fissata(&id);
                }
                Ok(json!({ "id": id, "fissata": voluta }))
            }
            "colore_nota" => {
                let id = self.nota_modificabile(&testo("id"))?;
                let colore = testo("colore").unwrap_or_default();
                if !colore.is_empty() && !nota::colore_valido(&colore) {
                    return Err(format!("Colore sconosciuto: «{colore}». Usa uno di: {}.", nota::COLORI.join(", ")));
                }
                if let Some(n) = self.nota_mut(&id) {
                    n.colore = colore.clone();
                }
                self.registra_modifica(&id);
                self.salva_ora();
                Ok(json!({ "id": id, "colore": colore }))
            }
            "sposta_nel_cestino" => {
                let id = self.nota_modificabile(&testo("id"))?;
                self.sposta_nel_cestino(&id);
                Ok(json!({ "id": id, "nel_cestino": true, "nota": "L'utente puo' ripristinarla dal cestino." }))
            }
            _ => Err(format!("strumento sconosciuto: {nome}")),
        }?;
        serde_json::to_string_pretty(&risultato).map_err(|e| e.to_string())
    }

    fn nota_esistente(&self, id: &Option<String>) -> Result<String, String> {
        let id = id.clone().ok_or("Manca l'id della nota.")?;
        match self.nota(&id) {
            Some(n) if n.visibile() => Ok(id),
            _ => Err(format!("Nessuna nota con id «{id}». Usa elenca_note o cerca per trovarla.")),
        }
    }

    /// Una nota che si puo' modificare: esiste, non e' nel cestino e l'utente
    /// non la sta scrivendo proprio ora.
    fn nota_modificabile(&self, id: &Option<String>) -> Result<String, String> {
        let id = self.nota_esistente(id)?;
        if self.nota(&id).is_some_and(|n| n.nel_cestino) {
            return Err("La nota e' nel cestino: l'utente deve prima ripristinarla.".to_owned());
        }
        if self.sta_scrivendo(&id) {
            return Err("L'utente sta scrivendo in questa nota proprio ora: riprova tra qualche secondo.".to_owned());
        }
        Ok(id)
    }

    fn descrivi(&self, n: &Nota) -> Value {
        json!({
            "id": n.id,
            "titolo": n.titolo_visibile(self.tr.t("senza_titolo")),
            "modificata": ora_locale::data_ora(n.modificata, self.tr.codice()),
            "fissata": n.fissata,
            "colore": n.colore,
            "parole": n.conta_parole(),
            "nel_cestino": n.nel_cestino,
            "copia_in_conflitto": n.conflitto_di.is_some(),
        })
    }

    fn mcp_stato(&self) -> Result<Value, String> {
        let cartella = self.cartella.as_ref().ok_or("nessuna cartella")?;
        let salvataggio = match &self.salvataggio {
            super::StatoSalvataggio::Salvato => "salvato".to_owned(),
            super::StatoSalvataggio::InCorso => "salvataggio in corso".to_owned(),
            super::StatoSalvataggio::Errore(e) => format!("errore: {e}"),
        };
        let computer: Vec<String> = self.rete.dispositivi().into_iter().map(|d| d.nome).collect();
        Ok(json!({
            "programma": format!("{} {}", crate::NOME_APP, crate::VERSIONE),
            "cartella": cartella.cartella_note().display().to_string(),
            "note": cartella.note_attive().len(),
            "nel_cestino": cartella.note_cestino().len(),
            "salvataggio": salvataggio,
            "sincronizzazione": self.rete.servizio.is_some(),
            "computer_collegati": computer,
            "modifiche_permesse": self.preferenze.mcp_modifiche,
        }))
    }

    fn mcp_elenca(&self, cestino: bool) -> Result<Value, String> {
        let note: Vec<&Nota> = if cestino {
            self.cartella.as_ref().map(|c| c.note_cestino()).unwrap_or_default()
        } else {
            self.note_ordinate()
        };
        Ok(json!({ "note": note.into_iter().map(|n| self.descrivi(n)).collect::<Vec<_>>() }))
    }

    fn mcp_leggi(&self, id: &str) -> Result<Value, String> {
        let n = self.nota(id).ok_or("nota sparita")?;
        let mut dati = self.descrivi(n);
        dati["titolo_scritto"] = json!(n.titolo);
        dati["testo"] = json!(n.testo);
        dati["creata"] = json!(ora_locale::data_ora(n.creata, self.tr.codice()));
        Ok(dati)
    }

    fn mcp_cerca(&self, ricerca: &str, cestino: bool) -> Result<Value, String> {
        if ricerca.trim().is_empty() {
            return Err("Manca il testo da cercare.".to_owned());
        }
        let cartella = self.cartella.as_ref().ok_or("nessuna cartella")?;
        let mut note = self.note_ordinate();
        if cestino {
            note.extend(cartella.note_cestino());
        }
        let trovate: Vec<Value> = note
            .into_iter()
            .filter(|n| n.corrisponde(ricerca))
            .map(|n| {
                let mut dati = self.descrivi(n);
                // La riga dove compare la ricerca, per decidere senza leggere tutto.
                let estratto = n
                    .testo
                    .lines()
                    .find(|r| !Nota::occorrenze(r, ricerca).is_empty())
                    .map(|r| r.chars().take(200).collect::<String>());
                dati["estratto"] = json!(estratto);
                dati
            })
            .collect();
        Ok(json!({ "trovate": trovate }))
    }

    fn mcp_crea(&mut self, titolo: &str, testo: &str) -> Result<Value, String> {
        let mut nota = Nota::nuova(&self.preferenze.id_dispositivo);
        nota.titolo = titolo.to_owned();
        nota.testo = testo.to_owned();
        let id = nota.id.clone();
        self.cartella.as_mut().ok_or("nessuna cartella")?.note.insert(id.clone(), nota);
        // La nota nuova compare come scheda, senza togliere all'utente quella che sta guardando.
        if self.interfaccia.selezionata.is_none() {
            self.interfaccia.selezionata = Some(id.clone());
        }
        self.registra_modifica(&id);
        self.salva_ora();
        Ok(json!({ "id": id, "creata": true }))
    }

    fn mcp_modifica(&mut self, id: &str, titolo: Option<String>, testo: Option<String>) -> Result<Value, String> {
        if titolo.is_none() && testo.is_none() {
            return Err("Niente da cambiare: indica titolo e/o testo.".to_owned());
        }
        // Prima la versione attuale nella cronologia: si puo' sempre tornare indietro.
        if let Some(cartella) = self.cartella.as_mut() {
            cartella.registra_versione(id, true);
        }
        let n = self.nota_mut(id).ok_or("nota sparita")?;
        if let Some(titolo) = titolo {
            n.titolo = titolo;
        }
        if let Some(testo) = testo {
            n.testo = testo;
        }
        self.registra_modifica(id);
        self.salva_ora();
        self.mcp_leggi(id)
    }

    fn mcp_spunta(&mut self, id: &str, voce: &str, fatta: bool) -> Result<Value, String> {
        let cercata = voce.trim().to_lowercase();
        if cercata.is_empty() {
            return Err("Indica il testo della voce da spuntare.".to_owned());
        }
        let mut testo = self.nota(id).map(|n| n.testo.clone()).unwrap_or_default();
        let caratteri: Vec<char> = testo.chars().collect();
        let trovata = lista::voci(&testo).into_iter().find(|v| {
            let riga: String = caratteri[v.inizio_testo..v.fine_riga].iter().collect();
            riga.to_lowercase().contains(&cercata)
        });
        let Some(trovata) = trovata else {
            return Err(format!("Nessuna voce di lista contiene «{voce}»."));
        };
        if trovata.fatta != fatta {
            lista::alterna(&mut testo, trovata.segno);
            self.mcp_modifica(id, None, Some(testo))?;
        }
        Ok(json!({ "id": id, "voce": voce, "fatta": fatta }))
    }
}
