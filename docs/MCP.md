# Assistente AI (MCP)

Schedario può offrire le note a un assistente AI compatibile con il **Model
Context Protocol** (MCP), per esempio Claude Code o Claude Desktop. L'assistente
legge e cerca le note e, solo se lo permetti, le modifica.

## Come si attiva

1. Nel menu di destra apri **Assistente AI (MCP)** e spunta **Consenti
   l'accesso agli assistenti AI**. Schedario crea un codice di accesso casuale.
2. Premi **Copia comando per Claude Code** e incolla la riga in un terminale,
   una volta sola:

   ```sh
   claude mcp add --scope user --transport http schedario http://127.0.0.1:47820/mcp --header "Authorization: Bearer <codice>"
   ```

   Da quel momento Claude Code vede Schedario in ogni cartella.
3. Se vuoi che l'assistente possa anche scrivere, spunta **Permetti modifiche**.

Schedario deve essere aperto (anche solo nell'area di notifica, o ridotto a
icona). **Nuovo codice** rende subito inutilizzabile quello vecchio: va
aggiornato nell'assistente ripetendo il passo 2.

## Cosa può fare l'assistente

| Strumento | Cosa fa | Serve "Permetti modifiche" |
|---|---|---|
| `stato` | cartella, numero di note, salvataggio, computer collegati | no |
| `elenca_note` | le note nell'ordine delle schede, o quelle nel cestino | no |
| `leggi_nota` | titolo e testo completo | no |
| `cerca` | note che contengono delle parole, con la riga trovata | no |
| `crea_nota` | una nota nuova | sì |
| `modifica_nota` | cambia titolo e/o testo | sì |
| `aggiungi_testo` | aggiunge righe in fondo | sì |
| `spunta_voce` | spunta o toglie la spunta a una voce `[ ]` | sì |
| `fissa_nota` | fissa o toglie dalle fissate | sì |
| `colore_nota` | cambia il colore | sì |
| `sposta_nel_cestino` | sposta nel cestino (ripristinabile) | sì |

Non esistono strumenti per eliminare definitivamente, svuotare il cestino,
cambiare le impostazioni di rete, importare backup o cambiare cartella: queste
operazioni restano solo all'utente.

## Sicurezza

- Il server ascolta solo su `127.0.0.1`: è raggiungibile solo da questo
  computer, mai dalla rete locale o da Internet.
- Ogni richiesta deve presentare il codice (`Authorization: Bearer …`); senza,
  la risposta è `401`.
- Le richieste con un'intestazione `Origin` non locale (una pagina web aperta
  nel browser) ricevono `403`.
- Prima di ogni modifica la versione precedente della nota va nella
  cronologia, quindi tutto si può riportare indietro. Una nota che stai
  scrivendo proprio in quel momento non viene toccata.
- Le modifiche dell'assistente passano dagli stessi percorsi di quelle fatte a
  mano: salvataggio, cronologia e sincronizzazione con gli altri computer.
- Il testo delle note può contenere istruzioni copiate da fuori (pagine web,
  email). L'assistente è avvisato di trattarlo come dati; le modifiche sono
  comunque spente all'inizio e quelle pericolose non esistono.

## Dettagli tecnici

- Trasporto "Streamable HTTP" dell'MCP nella forma più semplice: ogni messaggio
  JSON-RPC arriva con `POST /mcp` e la risposta è JSON (nessun flusso di
  eventi, nessuna sessione). Versioni del protocollo: 2025-11-25, 2025-06-18,
  2025-03-26.
- Nessuna libreria in più: HTTP con `std::net`, JSON con `serde_json`.
- Il codice è in `src/mcp/` (protocollo e server) e in
  `src/applicazione/mcp.rs` (esecuzione degli strumenti). I test sono in
  `tests/mcp.rs`, con richieste HTTP vere.
- Porta predefinita 47820; si cambia nelle preferenze (`mcp_porta`).
