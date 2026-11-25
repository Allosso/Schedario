//! Il server MCP provato con richieste HTTP vere, su un'istanza completa del
//! programma (senza finestre): codice di accesso, sola lettura, modifiche.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use schedario::applicazione::Applicazione;
use schedario::piattaforma::argomenti::Argomenti;

struct Temporanea(PathBuf);

impl Drop for Temporanea {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn istanza() -> (Applicazione, Temporanea) {
    let radice = std::env::temp_dir().join(format!("schedario-mcp-{}", schedario::modello::nota::nuovo_id()));
    let argomenti = Argomenti {
        cartella_note: Some(radice.join("note")),
        cartella_dati: Some(radice.join("dati")),
        ..Default::default()
    };
    let mut app = Applicazione::new(&argomenti);
    app.preferenze.mcp_porta = 0; // una porta libera qualsiasi
    app.imposta_mcp(true);
    assert!(app.mcp.servizio.is_some(), "server non avviato: {:?}", app.mcp.errore);
    (app, Temporanea(radice))
}

/// Invia una richiesta HTTP dal thread di prova mentre il programma
/// continua a girare (come farebbe la finestra), e restituisce (stato, corpo).
fn richiesta(app: &mut Applicazione, codice: Option<&str>, origine: Option<&str>, corpo: &Value) -> (u16, Value) {
    let porta = app.mcp.servizio.as_ref().unwrap().porta();
    let corpo = corpo.to_string();
    let mut testa = format!("POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{porta}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nContent-Length: {}\r\n", corpo.len());
    if let Some(codice) = codice {
        testa.push_str(&format!("Authorization: Bearer {codice}\r\n"));
    }
    if let Some(origine) = origine {
        testa.push_str(&format!("Origin: {origine}\r\n"));
    }
    testa.push_str("\r\n");
    let (invia, ricevi) = mpsc::channel();
    std::thread::spawn(move || {
        let mut flusso = TcpStream::connect(("127.0.0.1", porta)).unwrap();
        flusso.write_all(testa.as_bytes()).unwrap();
        flusso.write_all(corpo.as_bytes()).unwrap();
        let mut risposta = String::new();
        flusso.read_to_string(&mut risposta).unwrap();
        invia.send(risposta).unwrap();
    });
    let inizio = Instant::now();
    let risposta = loop {
        app.aggiorna();
        if let Ok(r) = ricevi.try_recv() {
            break r;
        }
        assert!(inizio.elapsed() < Duration::from_secs(10), "nessuna risposta dal server");
        std::thread::sleep(Duration::from_millis(10));
    };
    let stato: u16 = risposta.split_whitespace().nth(1).unwrap().parse().unwrap();
    let corpo = risposta.split_once("\r\n\r\n").map(|(_, c)| c).unwrap_or("");
    (stato, serde_json::from_str(corpo).unwrap_or(Value::Null))
}

fn chiama(app: &mut Applicazione, strumento: &str, argomenti: Value) -> (bool, String) {
    let codice = app.preferenze.mcp_codice.clone();
    let messaggio = json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":strumento,"arguments":argomenti}});
    let (stato, corpo) = richiesta(app, Some(&codice), None, &messaggio);
    assert_eq!(stato, 200);
    let risultato = &corpo["result"];
    (risultato["isError"].as_bool().unwrap(), risultato["content"][0]["text"].as_str().unwrap().to_owned())
}

#[test]
fn senza_codice_o_da_una_pagina_web_non_risponde() {
    let (mut app, _t) = istanza();
    let ping = json!({"jsonrpc":"2.0","id":1,"method":"ping"});
    assert_eq!(richiesta(&mut app, None, None, &ping).0, 401);
    assert_eq!(richiesta(&mut app, Some("codice-sbagliato"), None, &ping).0, 401);
    let codice = app.preferenze.mcp_codice.clone();
    assert_eq!(richiesta(&mut app, Some(&codice), Some("https://sito.example"), &ping).0, 403);
    let (stato, corpo) = richiesta(&mut app, Some(&codice), None, &ping);
    assert_eq!((stato, &corpo["result"]), (200, &json!({})));
    assert_eq!(app.mcp.servizio.as_ref().unwrap().richieste(), 1, "contano solo le richieste accettate");
}

#[test]
fn legge_sempre_ma_modifica_solo_se_permesso() {
    let (mut app, _t) = istanza();
    let codice = app.preferenze.mcp_codice.clone();
    let (_, corpo) = richiesta(&mut app, Some(&codice), None, &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"prova","version":"1"}}}));
    assert_eq!(corpo["result"]["serverInfo"]["name"], "schedario");
    // Una notifica: accettata, senza risposta.
    assert_eq!(richiesta(&mut app, Some(&codice), None, &json!({"jsonrpc":"2.0","method":"notifications/initialized"})).0, 202);

    // Sola lettura (all'inizio): creare non e' permesso.
    let (errore, testo) = chiama(&mut app, "crea_nota", json!({"titolo":"Spesa","testo":"[ ] latte\n[ ] pane"}));
    assert!(errore && testo.contains("Permetti modifiche"), "{testo}");

    app.preferenze.mcp_modifiche = true;
    let (errore, testo) = chiama(&mut app, "crea_nota", json!({"titolo":"Spesa","testo":"[ ] latte\n[ ] pane"}));
    assert!(!errore, "{testo}");
    let id = serde_json::from_str::<Value>(&testo).unwrap()["id"].as_str().unwrap().to_owned();
    assert_eq!(app.nota(&id).unwrap().titolo, "Spesa");

    let (_, testo) = chiama(&mut app, "cerca", json!({"testo":"LATTE"}));
    assert!(testo.contains(&id) && testo.contains("[ ] latte"), "{testo}");

    let (errore, _) = chiama(&mut app, "spunta_voce", json!({"id":id,"voce":"pane","fatta":true}));
    assert!(!errore);
    assert_eq!(app.nota(&id).unwrap().testo, "[ ] latte\n[x] pane");

    let (errore, _) = chiama(&mut app, "aggiungi_testo", json!({"id":id,"testo":"[ ] uova"}));
    assert!(!errore);
    assert_eq!(app.nota(&id).unwrap().testo, "[ ] latte\n[x] pane\n[ ] uova");
    // Ogni modifica ha lasciato la versione precedente nella cronologia.
    assert!(!app.cronologia(&id).is_empty());

    let (errore, testo) = chiama(&mut app, "colore_nota", json!({"id":id,"colore":"fucsia"}));
    assert!(errore && testo.contains("giallo"), "{testo}");

    let (errore, _) = chiama(&mut app, "sposta_nel_cestino", json!({"id":id}));
    assert!(!errore);
    assert!(app.nota(&id).unwrap().nel_cestino);
    let (errore, testo) = chiama(&mut app, "modifica_nota", json!({"id":id,"testo":"altro"}));
    assert!(errore && testo.contains("cestino"), "{testo}");

    let (errore, testo) = chiama(&mut app, "leggi_nota", json!({"id":"inesistente"}));
    assert!(errore && testo.contains("elenca_note"), "{testo}");
}
