//! Pannello "Rete locale" senza pulsante "Applica": nome e codice si salvano
//! quando si lascia il campo. Nessun test qui accende davvero la rete.

use std::fs;
use std::path::PathBuf;

use schedario::applicazione::Applicazione;
use schedario::piattaforma::argomenti::Argomenti;

struct Temporanea(PathBuf);

impl Drop for Temporanea {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn istanza() -> (Applicazione, Temporanea, Argomenti) {
    let radice = std::env::temp_dir().join(format!("schedario-impostazioni-{}", schedario::modello::nota::nuovo_id()));
    let argomenti = Argomenti {
        cartella_note: Some(radice.join("note")),
        cartella_dati: Some(radice.join("dati")),
        ..Default::default()
    };
    let app = Applicazione::new(&argomenti);
    assert!(app.cartella.is_some(), "cartella di prova non aperta: {:?}", app.errore_cartella);
    (app, Temporanea(radice), argomenti)
}

#[test]
fn nome_e_codice_si_salvano_lasciando_il_campo() {
    let (mut app, _t, argomenti) = istanza();
    app.rete.bozza_nome = "  Portatile  ".into();
    app.rete.bozza_codice = "abcd-efgh-jkmn-pqrs".into();
    app.conferma_bozze_rete();
    assert!(app.rete.errore.is_none());
    assert!(!app.rete.impostazioni.attiva, "salvare il codice non accende la rete");
    assert!(app.rete.servizio.is_none());
    drop(app);

    // Riaprendo il programma i valori ci sono ancora.
    let riaperta = Applicazione::new(&argomenti);
    assert_eq!(riaperta.rete.impostazioni.codice_gruppo, "abcd-efgh-jkmn-pqrs");
    assert_eq!(riaperta.rete.bozza_codice, "abcd-efgh-jkmn-pqrs");
    assert_eq!(riaperta.preferenze.nome_dispositivo, "Portatile");
}

#[test]
fn con_la_rete_spenta_un_codice_corto_si_salva_ma_non_si_accende() {
    let (mut app, _t, _a) = istanza();
    app.rete.bozza_codice = "corto".into();
    app.conferma_bozze_rete();
    assert_eq!(app.rete.impostazioni.codice_gruppo, "corto");
    assert!(app.rete.errore.is_none());

    // La casella "Sincronizza" lo rifiuta.
    app.applica_impostazioni_rete(true);
    assert!(!app.rete.impostazioni.attiva);
    assert!(app.rete.servizio.is_none());
    assert!(app.rete.errore.is_some());
}

#[test]
fn con_la_rete_accesa_un_codice_corto_non_sostituisce_quello_buono() {
    let (mut app, _t, _a) = istanza();
    // Simula una rete gia' accesa senza aprire porte vere.
    app.rete.impostazioni.attiva = true;
    app.rete.impostazioni.codice_gruppo = "abcd-efgh-jkmn-pqrs".into();
    app.rete.bozza_codice = "corto".into();
    app.conferma_bozze_rete();
    assert_eq!(app.rete.impostazioni.codice_gruppo, "abcd-efgh-jkmn-pqrs");
    assert!(app.rete.impostazioni.attiva);
    assert!(app.rete.errore.is_some());
}

#[test]
fn senza_modifiche_non_succede_nulla() {
    let (mut app, _t, _a) = istanza();
    app.rete.errore = Some("precedente".into());
    app.conferma_bozze_rete();
    assert_eq!(app.rete.errore.as_deref(), Some("precedente"));
}
