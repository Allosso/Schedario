//! Sincronizzazione reale tra due servizi sullo stesso computer (127.0.0.1).

use std::time::{Duration, Instant};

use schedario::modello::Nota;
use schedario::rete::{ConfigurazioneRete, Evento, ServizioRete};

fn avvia(porta: u16, codice: &str, note: Vec<Nota>, manuali: &[&str]) -> ServizioRete {
    let id = schedario::modello::nota::nuovo_id();
    let mut configurazione = ConfigurazioneRete::new(&id, &format!("pc-{porta}"), codice);
    configurazione.porta_sincronizzazione = porta;
    configurazione.solo_locale = true;
    configurazione.indirizzi_manuali = manuali.iter().map(|s| s.to_string()).collect();
    ServizioRete::avvia(configurazione, note, || {}).expect("avvio del servizio")
}

fn nota(titolo: &str) -> Nota {
    let mut nota = Nota::nuova("prova");
    nota.titolo = titolo.into();
    nota
}

/// Attende fino a `limite` che arrivino note; restituisce i titoli ricevuti.
fn attendi_note(servizio: &ServizioRete, limite: Duration) -> Vec<String> {
    let inizio = Instant::now();
    let mut titoli = Vec::new();
    while inizio.elapsed() < limite && titoli.is_empty() {
        for evento in servizio.eventi() {
            if let Evento::NoteRicevute { note, .. } = evento {
                titoli.extend(note.into_iter().map(|n| n.titolo));
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    titoli
}

#[test]
fn due_istanze_si_scambiano_le_note_in_entrambe_le_direzioni() {
    let a = avvia(47_931, "codice-di-prova", vec![nota("da A")], &[]);
    let b = avvia(47_941, "codice-di-prova", vec![nota("da B")], &["127.0.0.1:47931"]);
    b.sincronizza_ora();

    // B chiama A: riceve la nota di A nella risposta...
    assert_eq!(attendi_note(&b, Duration::from_secs(10)), ["da A"]);
    // ...e A riceve quella di B nell'invio finale.
    assert_eq!(attendi_note(&a, Duration::from_secs(10)), ["da B"]);

    // B ora conosce A come dispositivo raggiungibile.
    let dispositivi = b.dispositivi();
    assert!(dispositivi.iter().any(|d| d.nome == "pc-47931" && d.raggiungibile == Some(true)));
}

#[test]
fn una_modifica_piu_recente_vince() {
    let originale = nota("prima versione");
    let mut aggiornata = originale.clone();
    aggiornata.titolo = "seconda versione".into();
    aggiornata.segna_modificata("altro-pc");

    let a = avvia(47_951, "codice-di-prova", vec![aggiornata], &[]);
    let b = avvia(47_961, "codice-di-prova", vec![originale], &["127.0.0.1:47951"]);
    b.sincronizza_ora();
    assert_eq!(attendi_note(&b, Duration::from_secs(10)), ["seconda versione"]);
    // A ha gia' la versione piu' recente: non riceve nulla.
    assert!(attendi_note(&a, Duration::from_secs(2)).is_empty());
}

#[test]
fn un_codice_diverso_non_scambia_nulla() {
    let a = avvia(47_971, "codice-uno-123", vec![nota("segreta")], &[]);
    let b = avvia(47_981, "codice-due-456", vec![], &["127.0.0.1:47971"]);
    b.sincronizza_ora();
    assert!(attendi_note(&b, Duration::from_secs(3)).is_empty());
    assert!(attendi_note(&a, Duration::from_secs(1)).is_empty());
}

#[test]
fn si_prende_l_ordine_delle_schede_da_un_altro_computer() {
    let a = avvia(47_991, "codice-di-prova", vec![nota("x")], &[]);
    a.aggiorna_ordine(vec!["tre".into(), "uno".into(), "due".into()]);
    let b = avvia(48_001, "codice-di-prova", vec![], &["127.0.0.1:47991"]);
    b.sincronizza_ora();
    // B deve prima conoscere A (dopo la prima sincronizzazione).
    let inizio = Instant::now();
    let id_a = loop {
        if let Some(d) = b.dispositivi().into_iter().find(|d| d.nome == "pc-47991") {
            break d.id;
        }
        assert!(inizio.elapsed() < Duration::from_secs(10), "A non trovato");
        std::thread::sleep(Duration::from_millis(50));
    };
    b.chiedi_ordine(&id_a);
    let inizio = Instant::now();
    let ordine = 'attesa: loop {
        for evento in b.eventi() {
            match evento {
                Evento::OrdineRicevuto { ordine, da } => {
                    assert_eq!(da, "pc-47991");
                    break 'attesa ordine;
                }
                Evento::OrdineNonRicevuto { errore, .. } => panic!("ordine non ricevuto: {errore:?}"),
                _ => {}
            }
        }
        assert!(inizio.elapsed() < Duration::from_secs(10), "nessuna risposta");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(ordine, ["tre", "uno", "due"]);

    // Un computer sconosciuto: l'errore arriva come evento, senza bloccare nulla.
    b.chiedi_ordine("inesistente");
    let inizio = Instant::now();
    loop {
        if b.eventi().into_iter().any(|e| matches!(e, Evento::OrdineNonRicevuto { .. })) {
            break;
        }
        assert!(inizio.elapsed() < Duration::from_secs(5), "nessun errore");
        std::thread::sleep(Duration::from_millis(50));
    }
}