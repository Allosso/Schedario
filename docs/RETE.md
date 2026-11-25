# Sincronizzazione in rete locale

Due o più computer con Schedario aperto, nella stessa rete (stesso Wi-Fi o
stesso router) e con lo **stesso codice di gruppo**, si scambiano le note in
automatico. Non serve nessun server, account o connessione a Internet.

## Come si attiva

Su ogni computer:

1. apri **Rete locale** nella barra laterale;
2. sul primo computer premi **Genera**: viene creato un codice casuale come
   `k7m2-q9xd-4tpa-zr8c`. Premi **Copia** e scrivi lo stesso codice sugli altri
   computer. Le maiuscole e gli spazi all'inizio o alla fine non contano;
3. spunta **Sincronizza con altri computer**.

Nome e codice si salvano da soli quando lasci il campo, premi Invio o premi
Genera. Se la sincronizzazione è già attiva, riparte subito con i nuovi valori.
Un codice troppo corto scritto con la sincronizzazione attiva non la spegne:
resta in uso il codice precedente e compare un errore.

Puoi anche scegliere un codice tuo (almeno 8 caratteri). Se è più corto di 12
caratteri o poco vario, il programma lo segnala come debole.

Entro pochi secondi gli altri computer compaiono in **Dispositivi collegati**.
Da quel momento ogni modifica viene inviata dopo circa mezzo secondo di pausa
nella scrittura. In più c'è un controllo automatico ogni 20 secondi.

Su Windows, la prima volta il **firewall** chiede se consentire a Schedario
l'accesso alla rete. Consenti le **reti private**. Su macOS il sistema chiede
il permesso per la "rete locale".

Sul **telefono Android** la sezione è la stessa: scrivi il codice nel campo
(premi Invio per salvarlo), poi spunta la casella. Il nome proposto è quello
del telefono, preso da Impostazioni → Informazioni sul telefono. La
sincronizzazione è pensata per l'app aperta: in secondo piano Android può
fermarla in qualsiasi momento, e riparte quando riapri l'app.

Se la scoperta automatica non funziona (alcune reti aziendali o ospiti la
bloccano), aggiungi l'indirizzo dell'altro computer in **Indirizzi aggiunti a
mano**, per esempio `192.168.1.20`. L'indirizzo di ogni computer è scritto
sotto "Attiva sulla porta …".

## Cosa viene sincronizzato

- titolo, testo, colore e puntina ("fissata in alto") di ogni nota, comprese le
  spunte delle liste di controllo, che fanno parte del testo;
- lo spostamento nel cestino e il ripristino;
- l'**eliminazione definitiva** (anche "Svuota cestino"): la nota sparisce
  anche dagli altri computer;
- le note nuove, comprese quelle importate da un backup.

Non vengono sincronizzati i widget aperti, il tema, la lingua, la dimensione
del testo, la scorciatoia globale e la posizione delle finestre: sono
impostazioni di ciascun computer. Anche la **cronologia delle versioni** resta
su ciascun computer: prima che una modifica arrivata dalla rete sostituisca il
testo, la versione precedente viene salvata nella cronologia locale.

La sincronizzazione riguarda la **cartella di lavoro aperta**. Il codice di
gruppo è salvato in quella cartella (`.schedario/rete.json`). Così cartelle
diverse possono avere gruppi diversi.

## Conflitti: nessuna modifica va persa

Ogni nota conta quante modifiche ha ricevuto da ciascun computer (un "vettore di
versioni"). Così il programma distingue due situazioni:

- **una versione segue l'altra**: per esempio hai scritto sul portatile e il
  fisso non aveva toccato la nota. La versione nuova sostituisce la vecchia;
- **due versioni sono nate in parallelo**: la stessa nota è stata modificata su
  due computer prima che si sentissero. Questo è un vero conflitto.

In caso di conflitto la nota tiene la versione salvata per ultima. L'altra
**non viene buttata**: diventa una nuova nota "copia in conflitto". La sua
scheda ha il contorno rosso e sopra il testo c'è un avviso con i pulsanti
**Apri l'originale** e **Segna come risolto**. Tutti i computer risolvono il
conflitto nello stesso identico modo, quindi non si creano copie doppie.

Anche un'eliminazione definitiva contemporanea a una modifica crea una copia
in conflitto, così il testo nuovo resta.

**Mentre scrivi** una nota, le versioni di quella nota in arrivo dagli altri
computer aspettano: vengono applicate due secondi dopo l'ultima battuta. Il
testo non cambia mai sotto le dita.

## Sicurezza

- Dal codice di gruppo si ricava una chiave a 256 bit con **Argon2id**, usando
  32 MiB di memoria e 3 passaggi. Ogni tentativo di indovinare il codice costa
  tempo e memoria, anche a chi registra il traffico e prova offline. Un codice
  generato ha circa 79 bit di casualità: indovinarlo per tentativi non è
  realistico.
- Ogni messaggio, compresi gli annunci di presenza, è cifrato e autenticato con
  **ChaCha20-Poly1305**. Un messaggio con un codice diverso o alterato viene
  scartato.
- Chi è in rete ma non conosce il codice vede solo pacchetti illeggibili e
  un'etichetta casuale del gruppo. Non vede nomi, titoli né testi.
- Il codice è salvato in chiaro nella cartella di lavoro: chi può leggere i
  tuoi file può già leggere anche le note.

## Dettagli tecnici

| Cosa | Porta | Protocollo |
|---|---|---|
| Annunci di presenza (broadcast ogni 3 s) | 47810 | UDP |
| Scambio delle note | 47811 (o una libera, se occupata) | TCP |

Uno scambio è fatto di tre messaggi:

```text
chi chiama                          chi risponde
  Offerta(id, data di ogni nota) ──▶
                                 ◀── Risposta(note più recenti, note richieste)
  Invio(note richieste) ───────────▶
```

"Prendi l'ordine delle schede da…" usa uno scambio a parte, di due messaggi,
sulla stessa porta e con la stessa cifratura:

```text
chi chiede                          chi risponde
  RichiestaOrdine(id) ─────────────▶
                                 ◀── Ordine(identificativi delle schede, in ordine)
```

L'ordine non viaggia mai da solo: lo si chiede solo con il pulsante, e cambia
solo il computer che lo chiede. Una versione precedente di Schedario non conosce
`RichiestaOrdine` e chiude il collegamento: chi chiede lo segnala all'utente
("usa una versione precedente"). Il numero di versione del protocollo resta 2,
quindi le note continuano a sincronizzarsi anche con le versioni precedenti.

Ogni messaggio TCP è `lunghezza (4 byte) || nonce (12) || JSON cifrato || tag (16)`.
Un annuncio UDP è `"SCHEDAR2" || etichetta gruppo (16) || annuncio cifrato`.
Questa è la versione 2 del protocollo, che porta i vettori di versioni: non
comunica con la versione 1.
Il codice è in `src/rete/`; la regola di fusione, che non usa la rete e si può
testare da sola, è in `src/rete/fusione.rs`.

### Android

Il codice di rete è lo stesso. L'app chiede i permessi `INTERNET`,
`ACCESS_NETWORK_STATE`, `ACCESS_WIFI_STATE` e `CHANGE_WIFI_MULTICAST_STATE`
(tutti concessi all'installazione, senza domande). Molti telefoni scartano i
pacchetti broadcast e multicast per risparmiare batteria: finché l'app è aperta
`MainActivity` tiene un `WifiManager.MulticastLock`, che li lascia passare, e
lo rilascia alla chiusura. Le impostazioni sono in
`.schedario/rete.json` dentro la cartella privata dell'app.
