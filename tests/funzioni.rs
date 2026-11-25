//! Annulla dopo Cancella, note fissate, cronologia e nota rapida, provati su
//! istanze complete del programma (senza finestre).

use std::fs;
use std::path::PathBuf;

use schedario::applicazione::Applicazione;
use schedario::modello::Nota;
use schedario::piattaforma::argomenti::Argomenti;

struct Temporanea(PathBuf);

impl Drop for Temporanea {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn istanza(nome: &str) -> (Applicazione, Temporanea) {
    let radice = std::env::temp_dir().join(format!("schedario-funzioni-{nome}-{}", schedario::modello::nota::nuovo_id()));
    let argomenti = Argomenti {
        cartella_note: Some(radice.join("note")),
        cartella_dati: Some(radice.join("dati")),
        ..Default::default()
    };
    let app = Applicazione::new(&argomenti);
    assert!(app.cartella.is_some(), "cartella di prova non aperta: {:?}", app.errore_cartella);
    (app, Temporanea(radice))
}

fn nuova(app: &mut Applicazione, testo: &str) -> String {
    app.nuova_nota();
    let id = app.interfaccia.selezionata.clone().unwrap();
    app.nota_mut(&id).unwrap().testo = testo.to_owned();
    app.registra_modifica(&id);
    app.salva_ora();
    id
}

fn tutte(app: &Applicazione) -> Vec<Nota> {
    app.cartella.as_ref().unwrap().note.values().cloned().collect()
}

#[test]
fn annulla_dopo_cancella_rimette_la_nota_al_suo_posto() {
    let (mut app, _t) = istanza("annulla");
    let prima = nuova(&mut app, "prima");
    let seconda = nuova(&mut app, "seconda");
    app.sposta_nel_cestino(&prima);
    assert_eq!(app.cancellazione_annullabile(), Some(prima.as_str()));
    assert_eq!(app.interfaccia.selezionata.as_deref(), Some(seconda.as_str()));

    app.annulla_cancellazione();
    assert!(!app.nota(&prima).unwrap().nel_cestino);
    assert_eq!(app.interfaccia.selezionata.as_deref(), Some(prima.as_str()));
    assert_eq!(app.cancellazione_annullabile(), None, "l'avviso sparisce");

    // Se la nota esce dal cestino in un altro modo, l'avviso non vale piu'.
    app.sposta_nel_cestino(&seconda);
    app.ripristina(&seconda);
    assert_eq!(app.cancellazione_annullabile(), None);
}

#[test]
fn eliminare_dal_cestino_si_puo_annullare_e_poi_diventa_definitivo() {
    let (mut app, _t) = istanza("elimina-annulla");
    let prima = nuova(&mut app, "da eliminare");
    let seconda = nuova(&mut app, "anche questa");
    app.sposta_nel_cestino(&prima);
    app.sposta_nel_cestino(&seconda);
    assert_eq!(app.note_cestino_visibili().len(), 2);

    // Eliminata con ANNULLA: sparisce dal cestino ma non e' ancora eliminata.
    app.elimina_con_annulla(vec![prima.clone()]);
    assert_eq!(app.eliminazione_annullabile().map(|(ids, _)| ids.to_vec()), Some(vec![prima.clone()]));
    assert_eq!(app.note_cestino_visibili().len(), 1);
    assert!(app.nota(&prima).is_some_and(|n| n.visibile() && n.nel_cestino));

    // Annulla: torna nel cestino.
    app.annulla_eliminazione();
    assert_eq!(app.eliminazione_annullabile(), None);
    assert_eq!(app.note_cestino_visibili().len(), 2);

    // Confermata (scaduto ANNULLA, o l'app va in secondo piano): eliminata davvero.
    app.elimina_con_annulla(vec![prima.clone(), seconda.clone()]);
    app.conferma_eliminazione();
    assert!(app.note_cestino_visibili().is_empty());
    assert!(app.nota(&prima).is_none_or(|n| !n.visibile()));
    assert!(app.nota(&seconda).is_none_or(|n| !n.visibile()));
}

#[test]
fn il_cestino_ha_la_sua_ricerca_e_le_sue_note_accanto() {
    let (mut app, _t) = istanza("ricerca-cestino");
    let attiva = nuova(&mut app, "pane attivo");
    let a = nuova(&mut app, "pane vecchio");
    let b = nuova(&mut app, "latte vecchio");
    let c = nuova(&mut app, "pane scaduto");
    for id in [&a, &b, &c] {
        app.sposta_nel_cestino(id);
    }

    // Senza ricerca le note del cestino si sfogliano tra loro, non con le altre.
    let ordine: Vec<String> = app.note_cestino_visibili().iter().map(|n| n.id.clone()).collect();
    assert_eq!(ordine.len(), 3);
    assert_eq!(app.nota_vicina(&ordine[0], 1).as_deref(), Some(ordine[1].as_str()));
    assert_eq!(app.nota_vicina(&ordine[0], -1), None);
    assert_eq!(app.nota_vicina(&attiva, 1), None);

    // La ricerca del Cestino vale per le note del cestino; quella delle note no.
    app.interfaccia.ricerca_cestino = "pane".to_owned();
    assert_eq!(app.ricerca_per(&a), "pane");
    assert_eq!(app.ricerca_per(&attiva), "");
    let trovate: Vec<&String> = ordine.iter().filter(|id| **id != b).collect();
    assert_eq!(app.nota_vicina(trovate[0], 1).as_deref(), Some(trovate[1].as_str()), "salta la nota che non corrisponde");

    // La ricerca delle note non filtra il cestino.
    app.interfaccia.ricerca_cestino.clear();
    app.interfaccia.ricerca = "latte".to_owned();
    assert_eq!(app.ricerca_per(&a), "");
    assert!(app.nota_vicina(&ordine[0], 1).is_some());
}

#[test]
fn le_note_fissate_vengono_prima_e_arrivano_agli_altri_computer() {
    let (mut a, _ta) = istanza("fissa-a");
    let (mut b, _tb) = istanza("fissa-b");
    let vecchia = nuova(&mut a, "vecchia");
    let recente = nuova(&mut a, "recente");
    assert_eq!(a.note_attive_id(), [vecchia.clone(), recente.clone()]);

    a.alterna_fissata(&recente);
    assert_eq!(a.note_attive_id(), [recente.clone(), vecchia.clone()]);

    b.applica_note_remote(tutte(&a), "A");
    assert!(b.nota(&recente).unwrap().fissata);
    assert_eq!(b.note_attive_id(), [recente.clone(), vecchia]);
}

#[test]
fn la_cronologia_conserva_il_testo_sostituito_dalla_rete_e_si_ripristina() {
    let (mut a, _ta) = istanza("storico-a");
    let (mut b, _tb) = istanza("storico-b");
    let id = nuova(&mut a, "versione di A");
    b.applica_note_remote(tutte(&a), "A");
    assert_eq!(b.nota(&id).unwrap().testo, "versione di A");

    // A cambia il testo: su B la versione precedente finisce in cronologia.
    a.nota_mut(&id).unwrap().testo = "riscritta da A".into();
    a.registra_modifica(&id);
    a.salva_ora();
    b.applica_note_remote(tutte(&a), "A");
    assert_eq!(b.nota(&id).unwrap().testo, "riscritta da A");
    let versioni = b.cronologia(&id);
    assert_eq!(versioni.first().map(|v| v.testo.as_str()), Some("versione di A"));

    // Ripristinare la vecchia versione mette in cronologia quella attuale.
    b.ripristina_versione(&id, &versioni[0]);
    assert_eq!(b.nota(&id).unwrap().testo, "versione di A");
    let testi: Vec<String> = b.cronologia(&id).into_iter().map(|v| v.testo).collect();
    assert!(testi.contains(&"riscritta da A".to_owned()), "{testi:?}");

    // L'eliminazione definitiva cancella anche la cronologia.
    b.sposta_nel_cestino(&id);
    b.elimina_definitivamente(&id);
    assert!(b.cartella.as_ref().unwrap().cronologia(&id).is_empty());
}

#[test]
fn la_nota_rapida_apre_subito_un_widget() {
    let (mut app, _t) = istanza("rapida");
    app.finestra_visibile = false;
    app.nota_rapida();
    let id = app.interfaccia.selezionata.clone().unwrap();
    assert!(app.nota(&id).is_some());
    let widget = app.widget.iter().find(|w| w.stato.id_nota == id).expect("widget aperto");
    assert!(widget.fuoco_testo > 0, "il cursore va nel testo");
    assert!(!app.finestra_visibile, "la finestra principale resta nascosta");

    // Chiusa senza scrivere nulla, la nota rapida sparisce.
    app.chiudi_widget_e_scarta_se_vuota(&id);
    assert!(app.nota(&id).is_none());

    // Con del testo invece resta.
    app.nota_rapida();
    let id = app.interfaccia.selezionata.clone().unwrap();
    app.nota_mut(&id).unwrap().testo = "da ricordare".into();
    app.registra_modifica(&id);
    app.chiudi_widget_e_scarta_se_vuota(&id);
    assert!(app.nota(&id).is_some());
}

#[test]
fn la_dimensione_del_testo_resta_nei_limiti() {
    let (mut app, _t) = istanza("testo");
    for _ in 0..50 {
        app.cambia_dimensione_testo(Some(1.1));
    }
    assert_eq!(app.preferenze.dimensione_testo, schedario::archivio::preferenze::DIMENSIONE_TESTO_MASSIMA);
    app.cambia_dimensione_testo(None);
    assert_eq!(app.preferenze.dimensione_testo, schedario::archivio::preferenze::DIMENSIONE_TESTO_PREDEFINITA);
}

#[test]
fn trascinare_una_scheda_cambia_solo_l_ordine_di_questo_computer() {
    let (mut app, t) = istanza("ordine");
    let a = nuova(&mut app, "a");
    let b = nuova(&mut app, "b");
    let c = nuova(&mut app, "c");
    assert_eq!(app.note_attive_id(), [a.clone(), b.clone(), c.clone()]);
    let versione_c = app.nota(&c).unwrap().clone();

    // "c" trascinata al primo posto: la nota non cambia (niente data, niente rete).
    app.sposta_scheda(&c, 0);
    assert_eq!(app.note_attive_id(), [c.clone(), a.clone(), b.clone()]);
    let dopo = app.nota(&c).unwrap();
    assert_eq!((dopo.modificata, &dopo.versioni), (versione_c.modificata, &versione_c.versioni));

    // Ctrl+Shift+frecce: un posto alla volta, fermandosi ai bordi.
    app.sposta_scheda_di(&c, 1);
    assert_eq!(app.note_attive_id(), [a.clone(), c.clone(), b.clone()]);
    app.sposta_scheda_di(&a, -1);
    assert_eq!(app.note_attive_id(), [a.clone(), c.clone(), b.clone()]);

    // Una nota nuova va in fondo.
    let d = nuova(&mut app, "d");
    assert_eq!(app.note_attive_id(), [a.clone(), c.clone(), b.clone(), d.clone()]);

    // L'ordine resta alla riapertura della cartella.
    drop(app);
    let argomenti = Argomenti {
        cartella_note: Some(t.0.join("note")),
        cartella_dati: Some(t.0.join("dati")),
        ..Default::default()
    };
    let app = Applicazione::new(&argomenti);
    assert_eq!(app.note_attive_id(), [a, c, b, d]);
}

#[test]
fn le_fissate_restano_un_gruppo_a_parte() {
    let (mut app, _t) = istanza("ordine-fissate");
    let a = nuova(&mut app, "a");
    let b = nuova(&mut app, "b");
    let c = nuova(&mut app, "c");
    // Fissare porta la nota in fondo al gruppo delle fissate.
    app.alterna_fissata(&b);
    app.alterna_fissata(&c);
    assert_eq!(app.note_attive_id(), [b.clone(), c.clone(), a.clone()]);
    // Una nota normale non entra tra le fissate, e una fissata non esce.
    app.sposta_scheda(&a, 0);
    assert_eq!(app.note_attive_id(), [b.clone(), c.clone(), a.clone()]);
    app.sposta_scheda(&b, 2);
    assert_eq!(app.note_attive_id(), [c.clone(), b.clone(), a.clone()]);
    // Togliere la puntina la mette all'inizio delle altre.
    app.alterna_fissata(&c);
    assert_eq!(app.note_attive_id(), [b, c, a]);
}

#[test]
fn l_ordine_preso_da_un_altro_computer_si_puo_annullare() {
    let (mut app, _t) = istanza("ordine-adotta");
    let a = nuova(&mut app, "a");
    let b = nuova(&mut app, "b");
    let solo_qui = nuova(&mut app, "solo qui");
    let remoto = vec![b.clone(), "solo-sull-altro".to_owned(), a.clone()];
    app.adotta_ordine(&remoto);
    assert_eq!(app.note_attive_id(), [b.clone(), a.clone(), solo_qui.clone()]);
    assert!(app.ordine_annullabile());
    app.annulla_ordine();
    assert_eq!(app.note_attive_id(), [a, b, solo_qui]);
    assert!(!app.ordine_annullabile());
}

#[test]
fn sfogliando_si_passa_alla_nota_accanto_nell_ordine_delle_schede() {
    let (mut app, _t) = istanza("sfoglia");
    let a = nuova(&mut app, "pane e latte");
    let b = nuova(&mut app, "idee");
    let c = nuova(&mut app, "pane nero");
    assert_eq!(app.nota_vicina(&b, 1).as_deref(), Some(c.as_str()));
    assert_eq!(app.nota_vicina(&b, -1).as_deref(), Some(a.as_str()));
    // Alle estremita' non si va oltre.
    assert_eq!(app.nota_vicina(&a, -1), None);
    assert_eq!(app.nota_vicina(&c, 1), None);
    // Segue l'ordine scelto trascinando le schede.
    app.sposta_scheda(&c, 0);
    assert_eq!(app.nota_vicina(&c, 1).as_deref(), Some(a.as_str()));
    // Con una ricerca, solo tra le note trovate.
    app.interfaccia.ricerca = "pane".to_owned();
    assert_eq!(app.nota_vicina(&c, 1).as_deref(), Some(a.as_str()));
    assert_eq!(app.nota_vicina(&a, 1), None, "\"idee\" non contiene pane");
    // Nel cestino non c'e' una nota accanto.
    app.interfaccia.ricerca.clear();
    app.sposta_nel_cestino(&b);
    assert_eq!(app.nota_vicina(&b, 1), None);
}
