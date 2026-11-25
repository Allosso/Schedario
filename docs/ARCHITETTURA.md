# Architettura

Schedario è scritto in Rust. È diviso in una **libreria** (`src/lib.rs`), che
contiene tutta la logica e si può testare, e in un **eseguibile** sottile
(`src/main.rs`), che la avvia.

## Macro categorie

```text
build.rs                 solo per Windows: crea l'icona .ico e la mette nell'eseguibile (windres di MinGW)
examples/icona_store.rs  icona 512 × 512 per la scheda del Play Store (PNG scritto senza librerie)
script/genera-licenze.ps1  raccoglie i testi delle licenze in assets/licenze/licenze.txt (Informazioni → Licenze)
.github/workflows/       verifica.yml (test su Windows/Linux/macOS), rilascio.yml (bozza di Release da un'etichetta v…)
docs/PUBBLICARE.md       cosa e' pronto e cosa resta per GitHub e Play Store; docs/PRIVACY.md informativa

src/
├── main.rs              avvio: chiama schedario::avvia()
├── lib.rs               elenco delle categorie, punto di ingresso
│
├── modello/             DATI PURI: nessun disco, rete o grafica
│   ├── nota.rs          la nota, colore, fissata, vettore di versioni, copie in conflitto
│   ├── lista.rs         liste di controllo nel testo: [ ] e [x], Invio, Ctrl+L
│   ├── ordine.rs        ordine delle schede: fissate prima, spostamenti, ordine preso da un altro computer
│   └── tempo.rs         istanti in millisecondi, formattazione date
│
├── archivio/            DISCO
│   ├── cartella_lavoro.rs  apertura, blocco, riconciliazione indice/file
│   ├── file_testo.rs    Note/Nota-<id>.txt e Cestino/
│   ├── indice.rs        .schedario/indice.json (metadati + copia di recupero)
│   ├── sessione.rs      widget aperti, sezioni, ordine delle schede, impostazioni di rete
│   ├── preferenze.rs    lingua, tema, ultima cartella, identità del computer
│   ├── backup.rs        esportazione/importazione JSON e .txt
│   ├── storico.rs       cronologia delle versioni (.schedario/storico/<id>.json)
│   └── scrittura_atomica.rs  file temporaneo + sostituzione
│
├── rete/                RETE LOCALE
│   ├── cifratura.rs     chiave Argon2id, ChaCha20-Poly1305, codici generati
│   ├── protocollo.rs    formato dei messaggi TCP e degli annunci UDP
│   ├── fusione.rs       regole pure: cosa scambiare, segue/precede/conflitto
│   ├── scoperta.rs      annunci broadcast e ascolto
│   ├── sincronizzazione.rs  scambio Offerta → Risposta → Invio
│   └── servizio.rs      thread in background e canale verso l'interfaccia
│
├── mcp/                 ASSISTENTI AI (Model Context Protocol), vedi MCP.md
│   ├── protocollo.rs    JSON-RPC: inizializzazione, elenco e chiamata degli strumenti
│   └── server.rs        HTTP su 127.0.0.1, codice di accesso, canale verso il coordinatore
│
├── lingue/              TESTI (assets/lingue/it.json, en.json)
├── piattaforma/         DIFFERENZE TRA SISTEMI
│   ├── percorsi.rs      cartelle di configurazione e Documenti (anche se spostata)
│   ├── avvio_automatico.rs  registro (Windows), autostart XDG, LaunchAgents
│   ├── ora_locale.rs    date nell'ora locale, fuso orario letto dal sistema
│   ├── argomenti.rs     opzioni della riga di comando
│   └── android/         SOLO ANDROID (compilato solo per il telefono)
│       ├── mod.rs       avvio, cartella privata dell'app, eventi dalla parte Java
│       │                (Indietro, Nuova nota, file scelti), appunti, finestre
│       │                per salvare/aprire file, nome del telefono, margini
│       └── tastiera.rs  ponte tra la tastiera del telefono e i campi di egui
│
├── applicazione/        COORDINATORE (nessun disegno)
│   ├── stato.rs         struttura Applicazione, apertura cartelle, uscita
│   ├── note.rs          nuova nota, nota rapida, cestino e Annulla, fissate,
│   │                    cronologia, dimensione del testo, salvataggio
│   ├── sessione.rs      widget aperti, stato da ricordare
│   ├── file.rs          backup ed esportazioni
│   ├── mcp.rs           strumenti per gli assistenti AI: leggere, cercare, modificare (se permesso)
│   └── rete.rs          avvio del servizio, fusione delle note in arrivo, rinvio
│
└── interfaccia/         GRAFICA (egui/eframe)
    ├── programma.rs     ciclo di ogni fotogramma, scorciatoie, dimensione testo, chiusura
    ├── avvio.rs         scelta della cartella, solo se quella usata non è raggiungibile
    ├── principale/      finestra principale: lavoro a sinistra, menu a destra
    │   ├── schede.rs         schede, "+", frecce, rinomina, puntina, menu del tasto destro
    │   ├── editor.rs         titolo, data di modifica, strumenti della nota, testo
    │   ├── ricerca.rs        ricerca in cima al menu (solo note), ricerca del Cestino, menu chiuso
    │   ├── riga_stato.rs     contatori e riquadro dei messaggi (PC), riga dei messaggi (telefono)
    │   ├── barra_laterale.rs logo e sezioni retraibili del menu
    │   └── dialoghi.rs       finestre di messaggio e cronologia delle versioni
    ├── testo_nota.rs    campo del testo: caselle delle liste, parole cercate, menu Taglia/Copia/Incolla
    │                    (sul telefono si apre tenendo premuto e seleziona la parola)
    ├── tastiera.rs      ogni campo di testo si annuncia qui; su Android lo collega alla tastiera
    ├── schermo.rs       disposizione da telefono o da computer, misure per il dito
    ├── widget_fluttuante.rs  note sul desktop, senza cornice, titolo nella barra
    ├── area_notifica.rs icona vicino all'orologio (tray-icon / ksni)
    ├── scorciatoia_globale.rs  nota rapida da qualsiasi programma (global-hotkey), tasti a scelta
    ├── tema.rs          tavolozze chiara/scura, tinte dei colori delle note, Inter, interlinea
    ├── componenti.rs    pulsanti, icone disegnate, scelta del colore, logo, riquadri
    ├── icona_forme.rs   disegno dell'icona (schedario aperto) e file .ico, senza dipendenze: lo usa anche build.rs
    └── icona.rs         icona della finestra e dell'area di notifica; il logo e' in componenti.rs

android/                 VERSIONE ANDROID (vedi android/LEGGIMI.md)
├── src/lib.rs           libreria per il telefono: android_main e funzioni chiamate da Java
├── vendor/android-activity/  android-activity 0.6.1 con due correzioni e due aggiunte locali
├── progetto/            progetto Gradle: manifest, testi, MainActivity.java
├── compila.ps1          libreria Rust con l'NDK, poi APK con Gradle
└── prova-sul-telefono.ps1  installa, avvia e mostra i messaggi dell'app

tests/
├── archivio.rs          salvataggio, file esterni, blocco, cestino, backup
├── conflitti.rs         due istanze complete: modifiche parallele, rinvio, eliminazione
├── funzioni.rs          Annulla, fissate, cronologia, nota rapida, dimensione del testo
├── impostazioni_rete.rs nome e codice salvati senza "Applica", codici troppo corti
└── rete.rs              due istanze reali che si sincronizzano su 127.0.0.1
```

Le dipendenze vanno in una sola direzione:

```text
interfaccia ──▶ applicazione ──▶ archivio ──▶ modello
                     │                          ▲
                     └──────▶ rete ─────────────┘
```

`modello` non dipende da niente; `archivio` e `rete` non sanno nulla
dell'interfaccia; `applicazione` non disegna.

## Flusso di una modifica

1. L'utente scrive nell'editor o in un widget. Entrambi modificano **la stessa
   nota in memoria**, quindi le due viste sono sempre allineate.
2. `Applicazione::registra_modifica` aggiorna data e computer di origine, mette
   la nota tra quelle "in sospeso" e programma il salvataggio dopo 450 ms.
3. `Applicazione::salva_ora` scrive il `.txt` (in modo atomico) e poi l'indice.
   Se fallisce, le modifiche restano in memoria e si riprova dopo 3 secondi;
   l'uscita viene bloccata finché il salvataggio non riesce.
4. Dopo il salvataggio, le note vengono consegnate al servizio di rete, che
   avvisa gli altri computer.

## Finestre

egui chiama "viewport" ogni finestra del sistema operativo. La finestra
principale è il viewport radice; ogni widget è un viewport *immediato*, senza
decorazioni, trasparente e (se richiesto) sempre in primo piano. Quando la
finestra principale viene chiusa con dei widget aperti, viene solo nascosta:
eframe continua a disegnare i widget visibili. Chiudere l'ultimo widget la fa
ricomparire.

## Android

Sul telefono gira **lo stesso codice** della versione per computer: modello,
archivio, rete, applicazione e interfaccia. `Cargo.toml` è un workspace con due
membri: `schedario` (la libreria e il programma per computer) e
`android/` (`schedario-android`, una libreria `.so` che dipende da `schedario`).
Le parti che esistono solo sul computer sono escluse con
`cfg(not(target_os = "android"))`: widget fluttuanti, area di notifica,
scorciatoia globale, avvio con il sistema, server MCP, finestre di scelta di
rfd e "Apri un'altra cartella".

Al posto di queste, una piccola classe Java (`MainActivity`, sottoclasse di
**GameActivity**) fa quello che da Rust non si può fare: appunti di Android,
finestre di sistema per salvare e aprire file (backup), tasto Indietro,
scorciatoia "Nuova nota" sull'icona, permesso di ricevere gli annunci UDP
della rete locale (`MulticastLock`) e nome del telefono. Rust la chiama con JNI
(`piattaforma/android/mod.rs`); lei risponde chiamando le funzioni `…Nativo`
di `android/src/lib.rs`, che mettono un evento in coda per il fotogramma
successivo.

### Disposizione sul telefono

`interfaccia::schermo` decide due cose:

- `TOCCO` (vero su Android): i controlli hanno le misure per il dito. Ogni
  misura si scrive `misura(computer, telefono)`, per esempio i pulsanti
  `misura(30.0, 42.0)`: con lo spazio attorno si arriva ai 48 dp consigliati da
  Android;
- `telefono(ctx)`: schermo largo meno di 720 punti (telefono in verticale).
  Non c'e' il menu di destra: schede da bordo a bordo, la nota sotto e, sopra
  la nota in basso, "+" a sinistra (`principale::pulsante_nuova_nota`), la
  maniglia al centro (`principale::maniglia`: tocco o trascinamento verso
  l'alto apre la panoramica) e ☐ a destra. La panoramica ha in alto
  + · Cestino · Cerca · Impostazioni. Impostazioni, Cerca e Cestino sono pagine a
  schermo intero (`principale::pagine`, scelte da `StatoInterfaccia::pagina`,
  enum `Pagina`); la ricerca mostra l'elenco delle note trovate
  (`ricerca::risultati`, estratto da `Nota::estratto`). Con lo schermo piu'
  largo si usa la disposizione del computer.

Stile: `tema::RAGGIO` (6) e `RAGGIO_CONTENITORI` (8) per tutti gli angoli; i
pulsanti principali (`pulsante_accento`, strumenti con `accento`) hanno fondo
piatto e bordo nel colore principale. Le schede (`schede::disegna_scheda`) sono
linguette: sul telefono una linea corre sotto la riga, sul computer la linea e'
il bordo del riquadro della nota, e la linguetta aperta scende fino a lui
(`principale::area_di_lavoro`, con l'inizio del riquadro da `editor::mostra`).

Regole grafiche comuni (costanti in `tema.rs`, aiuti in `componenti.rs`), da
usare in menu, pagine, dialoghi e widget invece di valori scritti a mano:

- testi: `TESTO_SECONDARIO` 12,5 (suggerimenti, etichette sopra i campi,
  stati, errori; anche `TextStyle::Small`), `TESTO_VOCE` 13 (voci piccole di
  un elenco), `TESTO_VUOTO` 14, `TITOLO_DIALOGO` 16, `TITOLO_PAGINA` 18
  (telefono), `carattere_titolo_piccolo` per i titoli delle note in piccolo;
- `nota_attenuata` per il testo secondario, `errore` per gli errori (rosso),
  `etichetta` sopra un campo, `titolo_dialogo` nei dialoghi;
- scelte tra poche voci: `scelte` (larghe uguali, quella scelta con scritta e
  bordo nel colore principale), una sola: `scelta` (per esempio Mostra);
- campi di testo alti come i pulsanti (`ALTEZZA_PULSANTE`), margine 10 a
  sinistra; quadrati con icona `LATO_ICONA`; il "+" di una nota nuova sempre
  nel colore principale (`pulsante_icona_accento`);
- contorni "scelto/attenzione" `TRATTO_ATTIVO` 1,5, altrimenti `t.tratto()`;
  angoli `RAGGIO` per pulsanti e schede, `RAGGIO_CONTENITORI` per i riquadri
  (anche il widget e l'avviso di conflitto); rosso del Cestino mentre ci si
  trascina una nota `PERICOLO_LEGGERO` / `PERICOLO_SOPRA`;
- menu: le voci che cancellano sono rosse, Ripristina verde, come i pulsanti;
- dialoghi: prima l'azione, poi Annulla o Chiudi a destra; quando l'azione
  elimina e' rossa e quella evidenziata e' Annulla;
- telefono: margine 8 dal bordo per schede, panoramica e pagine (il testo
  della nota a 12); la lineetta di trascinamento e' `disegna_lineetta`.

Altre parti del telefono:

- `schede.rs`: con il dito la riga delle schede e' una `ScrollArea`
  orizzontale (`schermo::scorrevole` attiva il trascinamento, che egui da
  solo accende solo se riconosce uno schermo touch); `panoramica` e' la
  griglia di tutte le note. Una casella tenuta premuta (`TENUTA_PER_SPOSTARE`)
  e poi mossa si trascina: il dito si legge direttamente dall'input, perche'
  dopo la pressione lunga egui apre il menu della casella e non segnala piu'
  il trascinamento. Lasciata sul Cestino in alto, la nota va nel cestino (con
  il riquadro Annulla). `caselle_cestino` disegna le note del cestino con le
  stesse caselle (`casella_di`) nella pagina Cestino;
- note del cestino: si sfogliano tra loro (`nota_vicina`); la loro
  "panoramica" e' la pagina Cestino, che sale dal basso
  (`principale::panoramica_in_movimento`); tirandone giu' la testata si torna
  alle note (`schermo::NotaCheScende`, `principale::nota_in_discesa`,
  `editor::altra_nota`);
- sfogliare le note: l'editor registra la sua area (`schermo::zona_sfoglia`);
  un movimento orizzontale che parte li' diventa `Gesto::Sfoglia` in
  `smista_gesti` (egui non lo vede), lo spostamento passa all'editor con
  `schermo::imposta_sfoglia` e al rilascio `Applicazione::nota_vicina` dice
  quale nota aprire;
- `editor::intestazione_telefono`: titolo e la riga `● · 📌 · data · parole  🗑`
  in cui ogni pezzo si tocca (colore, fissa, la data apre la Cronologia,
  Cestino); senza riquadro attorno alla nota. Con la tastiera aperta resta
  solo il titolo. Sfogliando oltre l'ultima nota c'e' una pagina bianca
  (`Applicazione::pagina_bianca_dopo`): sollevando il dito si crea una nota.
  `editor::pulsante_lista` e' il ☐ tondo sulla nota (`testo_nota::voce_lista`
  mentre si scrive, `aggiungi_voce_in_fondo` altrimenti);
- il Cestino (riga della data o panoramica) registra la sua area
  (`schermo::imposta_zona_cestino`) e diventa rosso quando qualcuno trascina
  una nota (`schermo::si_trascina_nota`). La barra sopra la tastiera e la
  maniglia registrano la loro zona (`schermo::imposta_zona_barra_tastiera`,
  `imposta_zona_maniglia`): `smista_gesti` riconosce li' il gesto verso il
  basso (chiude la tastiera) e verso l'alto (panoramica).
  Le schede della riga in alto si trascinano con il dito (`schede::SchedaPresa`,
  stessa regola della panoramica: tenuta e poi mossa);
- menu del testo (`testo_nota::menu`): sul telefono e' una striscia
  (Taglia · Copia · Incolla · …) aperta con `Popup::at_position` sopra la
  parola, dentro `schermo::area_libera` cioe' sopra la tastiera (egui la
  terrebbe solo dentro lo schermo intero); la sua zona e' in
  `proteggi_fuoco`, cosi' toccarla non chiude la tastiera;
- `principale::avviso_annulla`: il riquadro "«…» nel cestino · ANNULLA"
  sopra la barra (sul telefono la riga di stato non mostra piu' quell'Annulla,
  vedi `riga_stato::ha_qualcosa_telefono`);
- `schede::tenuta_e_mossa`: una scheda si prende solo se il dito e' rimasto
  fermo (`FERMO`) per `TENUTA_PER_SPOSTARE`; un movimento prima la esclude
  per tutta quella pressione;
- cambiando nota, `Applicazione::scarta_se_vuota` elimina quella lasciata vuota
  (`StatoInterfaccia::aperta_prima` ricorda quale era aperta);
- `barra_tastiera.rs`: Annulla, Ripeti e chiudi tastiera. Le azioni sul testo
  passano da `tastiera::richiedi` e arrivano al
  campo come Ctrl+Z, Ctrl+Y e Ctrl+L, quindi seguono le stesse regole della
  tastiera del computer. I blocchi uniti usano `componenti::barra_strumenti`
  (`Strumento::attivo` per i pulsanti spenti). `componenti::menu_colori` apre i colori verso l'alto
  se il pulsante e' nella meta' bassa dello schermo;
- `schermo::proteggi_fuoco`: zone (caselle delle liste, barra sopra la
  tastiera, riga dei colori) in cui un tocco non toglie il fuoco al testo. Le
  caselle sono controlli a se', sopra il testo, con `Sense::click_and_drag`:
  il testo con il fuoco sente anche il trascinamento, e una casella che
  sentisse solo il clic lascerebbe il tocco al testo (cursore dentro "[ ]");
- la tastiera non scende e risale a ogni tocco: egui la spegne e riaccende
  per interrompere la parola in composizione; nella copia di android-activity
  un "nascondi" seguito subito da "mostra" si annullano (se la tastiera e'
  gia' chiusa, "mostra" la riapre).

Per provare la disposizione da telefono sul computer, nella versione di
sviluppo: `SCHEDARIO_TELEFONO=1` prima di avviare `target\debug\schedario.exe`.

I testi che sul telefono vanno detti diversamente hanno una seconda versione
nei file delle lingue, con la chiave che finisce in `@telefono` (per esempio
`rete_nome@telefono`: "Nome di questo dispositivo"). Su Android prende il
posto di quella normale; una prova controlla che ogni versione per il
telefono corrisponda a un testo esistente.

Gesti del dito (`programma::smista_gesti`): egui tratta il dito come un mouse,
quindi appoggiarlo sul testo sposterebbe subito il cursore e trascinarlo
selezionerebbe. Il tocco resta trattenuto per 0,15 s: se il dito si muove in
verticale diventa "rotella" (la nota scorre, con un po' di slancio dopo un
gesto veloce), se si alza subito e' un tocco (pressione e rilascio arrivano a
egui in due fotogrammi, altrimenti il cursore finirebbe in fondo al testo),
altrimenti passa a egui com'e' (pressione lunga, trascinamenti orizzontali).
La tastiera aspetta il rilascio di un tocco breve per comparire
(`trattieni_tastiera`): con una pressione lunga resta chiusa e non copre il
menu, e un tocco che chiude un menu non la apre.

L'app e' bloccata in verticale (`android:screenOrientation="portrait"`).

Descrizioni e pizzico (`programma::tocchi`): egui mostra la descrizione di un
pulsante mentre il dito resta fermo, ma la toglie quando il dito si alza; dopo
una pressione lunga (0,45 s) l'evento "puntatore sparito" viene scartato, cosi'
la descrizione resta. Con due dita sullo schermo il puntatore resta fermo, e il
pizzico cambia solo la dimensione del testo.

### Tastiera del telefono

La tastiera di Android non manda tasti: modifica una copia del testo del campo
(accenti tenendo premuto, suggerimenti, correzioni). GameActivity tiene quella
copia; winit 0.30 però non la passa a egui. Il ponte (`piattaforma/android/tastiera.rs`):

1. quando un campo prende il fuoco, manda alla tastiera il suo testo, il
   cursore e il tipo di campo (a capo o una riga sola, maiuscola a inizio
   frase, nessun suggerimento per il codice di gruppo);
2. prima di ogni fotogramma confronta il testo della tastiera con l'ultimo su
   cui erano d'accordo. Per esempio "la citta" che diventa "la città ": il
   ponte seleziona nel campo la "a" finale e scrive "à " come se fosse stata
   digitata. Così Annulla, le voci di lista con Invio e lo scorrimento fino
   al cursore funzionano come sul computer;
3. se invece il testo cambia da egui (Taglia, una voce di lista nuova, un
   tocco che sposta il cursore), il nuovo stato va alla tastiera.

Ogni campo chiama `interfaccia::tastiera::campo` dopo essersi disegnato; sul
computer la funzione non fa nulla. L'app ridisegna solo quando la tastiera
cambia qualcosa (un avviso aggiunto nella copia di android-activity), non a
ripetizione.

Due correzioni sono nella parte Java, perché riguardano GameActivity:

- il tasto Indietro va preso in `dispatchKeyEvent`, altrimenti il
  collegamento con la tastiera lo consuma;
- il collegamento di GameActivity non risponde a `getExtractedText`, la
  domanda con cui la tastiera Samsung legge il testo del campo. Senza
  risposta la tastiera crede il campo vuoto e mette la maiuscola: toccando la
  fine di "Pan" e scrivendo "e" usciva "PanE". `MainActivity` sostituisce il
  collegamento con uno che risponde.

## Scelte tecniche

| Esigenza | Scelta | Motivo |
|---|---|---|
| Interfaccia | egui + eframe (backend OpenGL "glow") | Rust puro, stesso codice su Windows, Linux (X11/Wayland) e macOS, finestre multiple |
| Finestre di dialogo | rfd | Dialoghi nativi; su Linux usa il portale XDG |
| Archivio | file `.txt` + JSON | Le note restano leggibili senza il programma; nessun database binario |
| Cifratura | chacha20poly1305 + argon2 + sha2 (RustCrypto) | Implementazioni Rust pure, molto diffuse |
| Conflitti | vettori di versioni per nota | Distinguono "modifica successiva" da "modifica parallela" senza un server |
| Area di notifica | tray-icon (Windows/macOS), ksni (Linux) | Su Linux evita GTK: niente librerie C né pacchetti `-dev` |
| Scorciatoia globale | global-hotkey | Stessa API su Windows, macOS e X11; su Wayland non esistono tasti globali |
| Ora locale | chrono (solo `clock`) | Legge il fuso orario del sistema su tutti e tre i sistemi, ora legale compresa |
| Liste di controllo | segni `[ ]`/`[x]` nel testo, caselle disegnate sopra | Il `.txt` resta leggibile; non serve un editor ricco |
| Cronologia | un file JSON per nota, solo locale | Semplice da ispezionare, nessun traffico di rete in più |
| Rete | std::net + socket2 | Nessun runtime asincrono: tre thread semplici |
| Assistenti AI | MCP su HTTP locale, scritto a mano | Il protocollo è aperto (MIT); la libreria ufficiale richiederebbe tokio |
| Icone | disegnate con forme geometriche | Identiche ovunque, nessun font o file grafico |
| Carattere | Inter 4.1 (OFL), dentro l'eseguibile con `include_bytes!` | Molto leggibile a schermo, identico su tutti i sistemi |
| Android | GameActivity (AndroidX, Apache-2.0) + Gradle | A differenza dell'activity nativa ha un vero collegamento con la tastiera: accenti e suggerimenti arrivano |
| Strumenti Android | SDK e NDK di Google | Unica eccezione accettata alla regola "nessuna licenza da accettare": servono solo per compilare, non finiscono nell'app |

Tutte le dipendenze sono open source con licenze permissive: vedi
`LICENZE-TERZE-PARTI.md`.
