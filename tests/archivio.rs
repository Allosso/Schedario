//! Cartella di lavoro su disco: salvataggio, riconciliazione, blocco, backup.

use std::fs;
use std::path::PathBuf;

use schedario::archivio::cartella_lavoro::ERRORE_IN_USO;
use schedario::archivio::file_testo::nome_file;
use schedario::archivio::{CartellaLavoro, backup};
use schedario::modello::Nota;

/// Cartella temporanea eliminata alla fine del test.
struct Temporanea(PathBuf);

impl Temporanea {
    fn new() -> Self {
        let percorso = std::env::temp_dir().join(format!("schedario-test-{}", schedario::modello::nota::nuovo_id()));
        fs::create_dir_all(&percorso).unwrap();
        Self(percorso)
    }
}

impl Drop for Temporanea {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn aggiungi(cartella: &mut CartellaLavoro, titolo: &str, testo: &str) -> String {
    let mut nota = Nota::nuova("pc");
    nota.titolo = titolo.into();
    nota.testo = testo.into();
    let id = nota.id.clone();
    cartella.note.insert(id.clone(), nota);
    cartella.salva(&[id.clone()]).unwrap();
    id
}

#[test]
fn salva_e_riapre() {
    let dir = Temporanea::new();
    let id = {
        let mut cartella = CartellaLavoro::apri(&dir.0, "pc").unwrap();
        aggiungi(&mut cartella, "Spesa", "latte\npane")
    };
    let file = dir.0.join("Note").join(nome_file(&id));
    assert_eq!(fs::read_to_string(&file).unwrap(), "Spesa\n\nlatte\npane");

    let cartella = CartellaLavoro::apri(&dir.0, "pc").unwrap();
    assert_eq!(cartella.note[&id].testo, "latte\npane");
}

#[test]
fn una_seconda_istanza_non_puo_aprire_la_stessa_cartella() {
    let dir = Temporanea::new();
    let _prima = CartellaLavoro::apri(&dir.0, "pc").unwrap();
    assert_eq!(CartellaLavoro::apri(&dir.0, "pc").err().as_deref(), Some(ERRORE_IN_USO));
}

#[test]
fn rilegge_le_modifiche_fatte_con_un_altro_editor() {
    let dir = Temporanea::new();
    let id = {
        let mut cartella = CartellaLavoro::apri(&dir.0, "pc").unwrap();
        aggiungi(&mut cartella, "Titolo", "vecchio")
    };
    fs::write(dir.0.join("Note").join(nome_file(&id)), "Titolo nuovo\r\n\r\ntesto modificato").unwrap();
    let cartella = CartellaLavoro::apri(&dir.0, "pc").unwrap();
    assert_eq!(cartella.note[&id].titolo, "Titolo nuovo");
    assert_eq!(cartella.note[&id].testo, "testo modificato");
}

#[test]
fn ricrea_un_file_cancellato_e_adotta_quelli_nuovi() {
    let dir = Temporanea::new();
    let id = {
        let mut cartella = CartellaLavoro::apri(&dir.0, "pc").unwrap();
        aggiungi(&mut cartella, "Da recuperare", "contenuto")
    };
    let file = dir.0.join("Note").join(nome_file(&id));
    fs::remove_file(&file).unwrap();
    let estraneo = schedario::modello::nota::nuovo_id();
    fs::write(dir.0.join("Note").join(nome_file(&estraneo)), "Arrivata\n\ndall'esterno").unwrap();

    let cartella = CartellaLavoro::apri(&dir.0, "pc").unwrap();
    assert!(file.exists(), "il file mancante va ricreato dall'indice");
    assert_eq!(cartella.note[&estraneo].titolo, "Arrivata");
}

#[test]
fn il_cestino_sposta_il_file_e_il_ripristino_lo_riporta() {
    let dir = Temporanea::new();
    let mut cartella = CartellaLavoro::apri(&dir.0, "pc").unwrap();
    let id = aggiungi(&mut cartella, "Temporanea", "");
    cartella.note.get_mut(&id).unwrap().nel_cestino = true;
    cartella.salva(&[id.clone()]).unwrap();
    assert!(dir.0.join("Cestino").join(nome_file(&id)).exists());
    assert!(!dir.0.join("Note").join(nome_file(&id)).exists());

    cartella.note.get_mut(&id).unwrap().nel_cestino = false;
    cartella.salva(&[id.clone()]).unwrap();
    assert!(dir.0.join("Note").join(nome_file(&id)).exists());
    assert!(!dir.0.join("Cestino").join(nome_file(&id)).exists());
}

#[test]
fn importa_anche_i_backup_del_prototipo() {
    let dir = Temporanea::new();
    let percorso = dir.0.join("Notte-backup.json");
    fs::write(
        &percorso,
        r#"{"format":"notte","version":1,"notes":[
            {"id":"a","title":"Uno","body":"primo","deleted":false},
            {"id":"b","title":"Due","body":"","deleted":true}]}"#,
    )
    .unwrap();
    let note = backup::importa(&percorso, "pc").unwrap();
    assert_eq!(note.len(), 2);
    assert_eq!((note[0].titolo.as_str(), note[0].testo.as_str(), note[0].nel_cestino), ("Uno", "primo", false));
    assert!(note[1].nel_cestino);
    assert!(note.iter().all(|n| schedario::modello::nota::id_valido(&n.id)), "nuovi identificativi");
}

#[test]
fn esporta_e_reimporta_un_backup() {
    let dir = Temporanea::new();
    let mut cartella = CartellaLavoro::apri(&dir.0.join("archivio"), "pc").unwrap();
    aggiungi(&mut cartella, "Nota", "testo");
    let percorso = dir.0.join("backup.json");
    backup::esporta(&percorso, cartella.note.values().collect()).unwrap();
    let reimportate = backup::importa(&percorso, "pc").unwrap();
    assert_eq!(reimportate.len(), 1);
    assert_eq!(reimportate[0].testo, "testo");
}
