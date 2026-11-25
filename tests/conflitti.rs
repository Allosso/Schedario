//! Due istanze complete del programma (senza finestre) che modificano la
//! stessa nota in parallelo e poi si scambiano le versioni.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

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
    let radice = std::env::temp_dir().join(format!("schedario-conflitti-{nome}-{}", schedario::modello::nota::nuovo_id()));
    let argomenti = Argomenti {
        cartella_note: Some(radice.join("note")),
        cartella_dati: Some(radice.join("dati")),
        ..Default::default()
    };
    let app = Applicazione::new(&argomenti);
    assert!(app.cartella.is_some(), "cartella di prova non aperta: {:?}", app.errore_cartella);
    (app, Temporanea(radice))
}

fn scrivi(app: &mut Applicazione, id: &str, testo: &str) {
    app.nota_mut(id).unwrap().testo = testo.to_owned();
    app.registra_modifica(id);
    app.salva_ora();
}

/// Tutte le note visibili, ordinate, come (id, testo).
fn contenuto(app: &Applicazione) -> Vec<(String, String)> {
    let mut note: Vec<_> = app
        .cartella
        .as_ref()
        .unwrap()
        .note
        .values()
        .filter(|n| n.visibile())
        .map(|n| (n.id.clone(), n.testo.clone()))
        .collect();
    note.sort();
    note
}

fn tutte(app: &Applicazione) -> Vec<Nota> {
    app.cartella.as_ref().unwrap().note.values().cloned().collect()
}

#[test]
fn modifiche_in_parallelo_non_perdono_testo_e_convergono() {
    let (mut a, _ta) = istanza("a");
    let (mut b, _tb) = istanza("b");

    // A crea la nota, B la riceve.
    a.nuova_nota();
    let id = a.interfaccia.selezionata.clone().unwrap();
    scrivi(&mut a, &id, "versione comune");
    std::thread::sleep(Duration::from_millis(2100));
    b.applica_note_remote(tutte(&a), "A");
    assert_eq!(b.nota(&id).unwrap().testo, "versione comune");

    // Entrambi modificano la stessa nota senza essersi sentiti.
    scrivi(&mut a, &id, "modificata su A");
    scrivi(&mut b, &id, "modificata su B");

    // B riceve mentre sta ancora scrivendo: la nota non cambia sotto le dita.
    b.applica_note_remote(tutte(&a), "A");
    assert_eq!(b.nota(&id).unwrap().testo, "modificata su B");

    // Passata la pausa di scrittura, entrambi si scambiano le versioni.
    std::thread::sleep(Duration::from_millis(2100));
    b.aggiorna();
    a.applica_note_remote(tutte(&b), "B");
    b.applica_note_remote(tutte(&a), "A");
    a.applica_note_remote(tutte(&b), "B");

    let finale_a = contenuto(&a);
    let finale_b = contenuto(&b);
    assert_eq!(finale_a, finale_b, "i due computer devono arrivare allo stesso risultato");
    let testi: Vec<&str> = finale_a.iter().map(|(_, t)| t.as_str()).collect();
    assert_eq!(finale_a.len(), 2, "nota unita + copia in conflitto: {testi:?}");
    assert!(testi.contains(&"modificata su A") && testi.contains(&"modificata su B"), "{testi:?}");

    // La copia e' riconoscibile e punta all'originale.
    let copie: Vec<_> = tutte(&a).into_iter().filter(|n| n.conflitto_di.is_some()).collect();
    assert_eq!(copie.len(), 1);
    assert_eq!(copie[0].conflitto_di.as_deref(), Some(id.as_str()));
}

#[test]
fn il_colore_arriva_all_altro_computer_e_resta_dopo_la_riapertura() {
    let (mut a, ta) = istanza("a");
    let (mut b, _tb) = istanza("b");
    a.nuova_nota();
    let id = a.interfaccia.selezionata.clone().unwrap();
    scrivi(&mut a, &id, "nota colorata");
    b.applica_note_remote(tutte(&a), "A");

    // Cambiare solo il colore e' una modifica come le altre.
    a.nota_mut(&id).unwrap().colore = "verde".into();
    a.registra_modifica(&id);
    a.salva_ora();
    b.applica_note_remote(tutte(&a), "A");
    assert_eq!(b.nota(&id).unwrap().colore, "verde");

    // Il colore e' salvato nell'indice e si ritrova riaprendo la cartella.
    let cartella_dati = ta.0.join("dati");
    let cartella_note = ta.0.join("note");
    drop(a);
    let argomenti = Argomenti { cartella_note: Some(cartella_note), cartella_dati: Some(cartella_dati), ..Default::default() };
    let riaperta = Applicazione::new(&argomenti);
    assert_eq!(riaperta.nota(&id).unwrap().colore, "verde");
}

#[test]
fn una_eliminazione_definitiva_arriva_anche_all_altro_computer() {
    let (mut a, _ta) = istanza("a");
    let (mut b, _tb) = istanza("b");
    a.nuova_nota();
    let id = a.interfaccia.selezionata.clone().unwrap();
    scrivi(&mut a, &id, "da eliminare");
    b.applica_note_remote(tutte(&a), "A");
    assert!(b.nota(&id).is_some());

    std::thread::sleep(Duration::from_millis(2100));
    a.sposta_nel_cestino(&id);
    a.svuota_cestino();
    assert!(a.nota(&id).is_none());
    let file = a.cartella.as_ref().unwrap().cartella_note().join(schedario::archivio::file_testo::nome_file(&id));
    assert!(!file.exists(), "il file di testo deve sparire dal disco");

    b.applica_note_remote(tutte(&a), "A");
    assert!(b.nota(&id).is_none(), "l'eliminazione deve propagarsi");
}
