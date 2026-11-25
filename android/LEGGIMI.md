# Schedario per Android

Versione per telefono di Schedario, **in sviluppo**: le funzioni sono quelle
del programma per computer, l'interfaccia è ancora provvisoria. La versione
di sviluppo si chiama "Schedario (sviluppo)" (`it.schedario.sviluppo`); quella
da pubblicare "Schedario", con l'identificativo definitivo ancora da scegliere
(vedi "Versione da pubblicare").

Funziona da Android 8.0 in su (API 26), su telefoni a 64 bit (arm64-v8a).

## Come è fatta

- `src/lib.rs`: la libreria `libschedario_android.so`. Contiene `android_main`,
  da cui parte l'app, e le funzioni `…Nativo` che la parte Java chiama
  (Indietro, Nuova nota, file aperto, file salvato). Tutto il resto è la
  libreria `schedario` della cartella superiore, la stessa del computer; le
  parti solo Android sono in `src/piattaforma/android/`.
- `progetto/`: progetto Gradle. `MainActivity.java` estende GameActivity e fa
  solo ciò che da Rust non si può fare: appunti, finestre di Android per
  salvare e aprire file, tasto Indietro, scorciatoia "Nuova nota" sull'icona,
  permesso di ricevere gli annunci della rete locale (`MulticastLock`), nome
  del telefono, tema chiaro/scuro del sistema e colori delle barre di sistema.
  Contiene anche due correzioni al collegamento con la tastiera (vedi
  `docs/ARCHITETTURA.md`, sezione Android).
- Icona dell'app: icona adattiva vettoriale, senza immagini
  (`res/mipmap-anydpi-v26/icona.xml`). Il fondo blu è in `res/values/colori.xml`,
  il disegno dello schedario in `res/drawable/icona_schede.xml`, la variante a un
  solo colore per le icone a tema (Android 13 e successivi) in
  `res/drawable/icona_monocromatica.xml`: le stesse forme di
  `src/interfaccia/icona_forme.rs`, da aggiornare insieme.
- `vendor/android-activity`: android-activity 0.6.1 (MIT OR Apache-2.0),
  collegata da `[patch.crates-io]` nel `Cargo.toml` principale. Le modifiche
  sono segnate "Correzione locale" / "Aggiunta locale":
  - con il testo della tastiera vuoto l'originale riceveva un puntatore nullo
    e l'app si chiudeva al primo tocco su un campo;
  - `AndroidApp::window_insets_schedario`: spazio occupato da barre di sistema
    e tastiera, che l'originale non rende disponibile;
  - `set_text_event_hook_schedario`: avvisa quando la tastiera cambia il
    testo, così l'app ridisegna solo allora invece che 30 volte al secondo;
  - `hide_soft_input` rimandato al fotogramma successivo
    (`applica_nascondi_tastiera_schedario`): un "mostra" subito dopo lo
    annulla, e la tastiera non scende e risale a ogni tocco nel testo.

  Quando una versione nuova di android-activity avrà queste cose, la copia si
  potrà togliere.

Le note stanno nella cartella privata dell'app
(`/data/user/0/it.schedario.sviluppo/files/Note`), le preferenze in `files/Dati`.
`files/Scambio` contiene i file temporanei dei backup in uscita e in entrata.

## Compilare

Servono gli strumenti già installati su questo PC: Java 21
(`%LOCALAPPDATA%\Programs\jdk-21`), Android SDK (`%LOCALAPPDATA%\Android\Sdk`,
piattaforme 35 e 36, build-tools 36.1.0), NDK 28.2, Rust con il target
`aarch64-linux-android`, Gradle 8.14.3 (`%LOCALAPPDATA%\Programs\gradle-8.14.3`).

```powershell
.\compila.ps1
```

1. `cargo build --release --target aarch64-linux-android -p schedario-android`
   con il compilatore dell'NDK, poi copia la `.so` in
   `progetto\app\src\main\jniLibs\arm64-v8a`.
2. Gradle aggiunge GameActivity e AppCompat e firma con la chiave di sviluppo
   standard di Android (`%USERPROFILE%\.android\debug.keystore`).

Risultato: `progetto\app\build\outputs\apk\debug\app-debug.apk` (circa 11 MB),
con nome "Schedario (sviluppo)" e identificativo `it.schedario.sviluppo`.

### Versione da pubblicare

```powershell
.\compila.ps1 -Pubblicazione
```

Crea `progetto\app\build\outputs\bundle\release\app-release.aab` (il pacchetto
per il Play Store) e `progetto\app\build\outputs\apk\release\app-release.apk`
(da distribuire altrove, per esempio su GitHub), con nome "Schedario". Prima
servono due cose:

1. **Identificativo definitivo**: `schedarioId` in `progetto\gradle.properties`,
   impostato a `io.github.allosso.schedario` (account GitHub Allosso). Una
   volta pubblicato non si può più cambiare. Senza, la compilazione si ferma
   con un messaggio.
2. **Chiave di firma**: il file `%USERPROFILE%\.schedario\firma.properties`
   (oppure il percorso nella variabile d'ambiente `SCHEDARIO_FIRMA`), fuori dal
   progetto e mai su GitHub, con `storeFile`, `storePassword`, `keyAlias`,
   `keyPassword`. Senza, i pacchetti escono non firmati, utili solo per
   controllare che si compilino (il Play Store li rifiuta). Della chiave va
   tenuta una copia di sicurezza: senza la stessa chiave gli aggiornamenti non
   si installano sopra la versione precedente.

Altro già pronto per il Play Store:

- Android 16 (API 36) come versione di riferimento (`targetSdk`), obbligatoria
  per le app nuove dal 31/08/2026; numero di versione preso da `Cargo.toml`
  (1.2.3 → `versionCode` 10203), da alzare a ogni aggiornamento.
- Indietro con il sistema "Indietro predittivo" (`OnBackPressedCallback` in
  `MainActivity`, `enableOnBackInvokedCallback` nel manifest): con le app per
  Android 16 il sistema non manda più il tasto `KEYCODE_BACK`.
- Libreria Rust allineata a pagine di 16 KB (richiesto dal 01/02/2027),
  architettura a 64 bit.
- Icona da 512 × 512 per la scheda dello Store, dallo stesso disegno dell'app:
  `cargo run --release --example icona_store -- dist\play-store\icona-512.png`.

## Provare sul telefono

1. Sul telefono: Impostazioni → Informazioni sul telefono → Informazioni
   software → tocca 7 volte "Numero build" per attivare le Opzioni
   sviluppatore; poi Opzioni sviluppatore → **Debug USB**.
2. Collega il telefono con il cavo e accetta la richiesta "Consentire il debug USB?".
   Se il telefono sparisce durante l'installazione, prova un altro cavo o
   un'altra porta USB.
3. Esegui `.\prova-sul-telefono.ps1`: installa l'app, la avvia e mostra i
   messaggi (Ctrl+C per smettere).

Per provare la sincronizzazione senza toccare le note vere del computer, avvia
sul PC una copia con cartelle di prova:

```powershell
..\target\debug\schedario.exe --data-dir C:\prova\dati --notes-dir C:\prova\note
```

## Provato

Il 22/11/2026 su un Galaxy S24 con la tastiera Samsung:

- scrittura in titolo, testo e codice di gruppo: maiuscola a inizio frase,
  suggerimenti, accenti, cancella, a capo, voci di lista con Invio;
- tasto Indietro, scorciatoia "Nuova nota" (con la tastiera già aperta);
- menu tenendo premuto, Copia negli appunti di Android, incolla;
- Esporta backup in Download e Importa backup;
- sincronizzazione con un PC nei due sensi;
- le note restano dopo aver chiuso e reinstallato l'app.

Interfaccia per il telefono, provata lo stesso giorno: pulsante ☰ e menu da
destra (si chiude con Indietro), elenco delle note trovate, schede che
scorrono con il dito, descrizione tenendo premuto un pulsante (senza
premerlo), tema "Come il telefono" / Chiaro / Scuro con le barre di sistema
negli stessi colori, titolo e strumenti della nota che stanno nello schermo.

Poi, sempre il 22/11/2026: riga delle schede che scorre in modo continuo,
menu aperto e chiuso trascinando dal bordo, panoramica delle note (tocco,
menu tenendo premuto, Cancella), barra sopra la tastiera (Annulla, Ripeti,
voce di lista, chiudi tastiera), caselle che si spuntano senza aprire la
tastiera, tastiera che resta su tra un tocco e l'altro e si riapre dopo
averla chiusa, "Parola 3 di 4" con ‹ › (provato sul PC con
`SCHEDARIO_TELEFONO=1`).

Poi: tastiera che non copre il menu Taglia/Copia/Incolla e non si apre
chiudendolo, vibrazione (registro di Android: HEAVY_CLICK), nota che scorre
con il dito anche mentre si scrive, cursore sotto il dito al primo tocco,
Indietro che chiude il menu. L'app ora resta in verticale.

Provati dall'utente: Esporta nota come testo, rinomina di una scheda, pizzico
a due dita (si bloccava facendo rimpicciolire e subito ingrandire: corretto
il conteggio delle dita, riprovato e funziona).

Quarto giro (22/11/2026, provato sul telefono e sul PC con
`SCHEDARIO_TELEFONO=1`): nota senza riquadro; titolo con il pulsante ⋮ e la
riga "oggi 22:41 · 1 parola"; pulsante tondo + in basso a destra; mentre si
scrive solo titolo e testo, con Fissa e Colore sulla barra sopra la tastiera
(i colori si aprono verso l'alto, la tastiera resta aperta); nota vuota
scartata passando a un'altra; menu senza logo e senza ›, con le sezioni come
righe; panoramica con la data e "Nota vuota"; scheda aperta sempre intera.
Corretto anche: toccare una casella con la tastiera aperta spostava il cursore
dentro "[ ]", e scrivendo la voce si rompeva.

Quinto giro (23/11/2026): barra sopra la tastiera con i pulsanti sempre allo
stesso posto (☐ spento nel titolo, Annulla|Ripeti e Fissa|Colore in blocchi
uniti); Indietro dalla nota apre la panoramica, il secondo esce dall'app.
Poi, sempre il 23/11/2026: niente piu' menu laterale. Schede da bordo a bordo,
barra in basso (Impostazioni, Cerca, Note, Nuova), pagine Impostazioni, Cerca e
Cestino a schermo intero (dal Cestino una nota si apre con Ripristina, Indietro
torna al Cestino); nella panoramica una scheda tenuta premuta si trascina per
cambiare l'ordine o sul Cestino (con Annulla); trascinando di lato la nota si
passa a quella accanto. `MainActivity` non riserva piu' il bordo destro al
gesto del menu. (Provato con `adb input motionevent`, non ancora con il dito.)
Poi: Fissa · Colore · Cronologia con le scritte sotto il titolo (via ⋮);
pulsante tondo ☐ sulla nota (aggiunge una voce in fondo o trasforma la riga);
barra sopra la tastiera solo Annulla|Ripeti e chiudi; Cestino in basso a
sinistra (cestina la nota aperta, con Annulla); schede trascinabili nella riga
e sul Cestino; menu Taglia/Copia/Incolla sempre sopra la tastiera.
Poi: riquadro grande "«Pane» nel cestino · ANNULLA" con il tempo che resta
(10 s); le schede si prendono solo con il dito fermo mezzo secondo (prima
bastava scorrere piano); menu del testo su una riga (Taglia · Copia · Incolla
· …) sopra la parola; pulsante ☐ piu' lontano dalla barra.
Poi (23/11/2026, mattina): la barra in basso non c'e' piu'. Riga della data
cliccabile (● colore, 📌 fissa, data → Cronologia, 🗑 Cestino in alto a
destra); "+" in basso a sinistra, maniglia al centro (tocco o verso l'alto:
panoramica), ☐ a destra, tutti piu' piccoli (46 pt); panoramica con + · Cestino
· Cerca · Impostazioni in alto; sfogliando oltre l'ultima nota se ne crea una;
Indietro dalla nota esce dall'app, dalla panoramica torna alla nota; barra
sopra la tastiera trascinabile verso il basso per chiuderla.
Poi (23/11/2026, pomeriggio): panoramica con la ricerca sempre aperta, il
Cestino grande in fondo e ⌄ (anche trascinando la testata in giu') per
tornare alla nota; doppio Indietro per uscire; cursore nascosto a tastiera
chiusa. Corretti: il testo corto si poteva far salire fuori dall'area (lo
spazio lasciato per ☐ si sommava all'altezza del campo) e la maniglia sembrava
fuori centro (☐ e "+" partivano da aree diverse: ora `schermo::area_pulsanti`).
Poi: stile sobrio in tutta l'app (angoli 6/8 invece di 8/10, pulsanti
principali con bordo invece che pieni), schede a linguetta sul telefono e sul
PC, Colore e Fissa come pulsanti quadrati da 40 pt, "+" e ☐ quadrati con bordo.
Poi: "+" anche in fondo alla riga delle schede (dopo l'ultima, nella fascia che
scorre); sul PC ☐ fluttuante come sul telefono e Lista tolta dalla barra
Fissa · Cronologia · Colore. Corretta una sovrapposizione di 4 pt tra le schede
(il titolo occupava spazio nella riga).
Poi: panoramica che sale e scende seguendo il dito (`Gesto::Pannello`,
animazione con `animate_value_with_time`), aperta da tutta la fascia in
basso e chiusa da tutta la griglia quando e' in cima; sfogliando entra la nota
vera e lo scorrimento si completa con un'animazione (il cambio lo fa
`editor::spostamento_sfoglia` a fine corsa). Nota per le prove: `adb input
motionevent` senza pausa iniziale non arriva come un dito; usare `input swipe`.
Poi: nella fascia in basso il gesto si decide dopo 0,4 s invece di 0,15
(`ATTESA_FASCIA`): un pollice fermo un attimo prima di salire diventava un
tocco normale e funzionava solo la lineetta (con adb non si vedeva, il dito
finto parte subito); tolta la descrizione della maniglia; la panoramica in
movimento senza la dissolvenza di egui (`fade_in(false)`), che mostrava per un
attimo la nota sotto; sfogliando la nota accanto e' disegnata da `contenuto`
come quella aperta (caselle, voci fatte, intestazione).
Poi: la panoramica si apre anche dalla meta' bassa della nota quando il testo
non scorre piu' giu' (`schermo::zona_apri_panoramica`, registrata da
`editor` con lo stato dello ScrollArea); a meta' di una nota lunga il gesto
resta uno scorrimento. Provato con una nota di 40 righe (poi cestinata).
Poi: sul testo della nota (`schermo::zona_testo`) il dito aspetta tutta la
pressione lunga (`ATTESA_GESTO + PRESSIONE_LUNGA`) prima di arrivare a egui;
se si muove prima e' sempre uno scorrimento (anche di lato), se resta fermo
arriva gia' come pressione lunga (`max_click_duration` portato a 0 per quel
tocco, `soglia_lunga`). Maniglie della selezione (`testo_nota::maniglie`):
gocce sotto inizio e fine, trascinabili (il tocco li' arriva subito a egui,
`schermo::maniglie_selezione`), menu nascosto durante il trascinamento e
riaperto con `Popup::open_id`; contro il bordo dello schermo la goccia si
gira. Le prove con adb: `input motionevent DOWN …; sleep 0.9; UP` per la
pressione lunga.
Poi: stile unico dei pulsanti come le schede (`componenti::fondo_pulsante`,
`disegna_fondo_pulsante`, `weak_bg_fill` trasparente in `tema::applica`);
Cestino della riga della data con il bordo; panoramica e Impostazioni
affiancate: `Gesto::Laterale` (verso sinistra dalla panoramica, verso destra
dalle Impostazioni), `schermo::scorri_impostazioni` e l'animazione
`apertura_impostazioni` in `principale::telefono`, che disegna le due pagine
in due `new_child` spostati.
Poi: Cestino a sinistra della panoramica (`Gesto::Laterale { pagina, apri }`,
`schermo::scorri_laterale`, `principale::pagine_affiancate`); pagine tirate
verso il basso (`Gesto::ChiudiPagina`, `schermo::zona_chiudi_pagina`,
`pagina_uscente`, `principale::pagina_in_discesa`); uscendo dal Cestino si
torna sempre alla panoramica (`pagine::torna_alla_panoramica`, con
`prima_del_cestino`): prima si girava in tondo tra Cestino e nota cestinata.
Eliminazione con ANNULLA (`Applicazione::elimina_con_annulla`, confermata da
`aggiorna` dopo 10 s o da `salva_tutto`). Ctrl+F apre la panoramica; tolta
la vecchia pagina Cerca. Contatori nelle schede della panoramica.
Nota per le prove: il giro di Claude tra due comandi adb puo' superare i 10 s
di ANNULLA; eliminazione e ANNULLA vanno nello stesso comando.
Poi: Cestino a schede come la panoramica (`schede::caselle_cestino`, con
`casella_di` in comune); le note del cestino si sfogliano tra loro
(`Applicazione::nota_vicina` usa `note_cestino_visibili` per una nota del
cestino). Aperta una nota del cestino la sua "panoramica" e' la pagina
Cestino: `principale::telefono` la disegna salendo in
`panoramica_in_movimento` e, salita del tutto, passa a `Pagina::Cestino`.
Trascinando in giu' la testata della nota del cestino
(`Programma::nota_del_cestino_da_chiudere`, `schermo::NotaCheScende`,
`principale::nota_in_discesa` con `editor::altra_nota`) si torna alle note;
Impostazioni e Cestino tirati giu' tornano alla nota di partenza, anche del
cestino (`pagine::torna_alla_nota`), mentre «‹», Indietro e il passaggio di
lato alla panoramica tornano alle note (`torna_alla_panoramica`). Elimina
definitivamente passa alla nota del cestino accanto.
Corretti poi tre difetti visti passando tra panoramica e Cestino: (1) egui
ricorda lo scorrimento in base al posto nell'albero dell'interfaccia, e la
panoramica e le pagine si disegnano in posti diversi (a riposo, di lato, in
salita/discesa): ognuna aveva il suo scorrimento e a fine animazione la
griglia saltava. Ora `schede::panoramica` e `pagine::mostra` usano un
`UiBuilder::scope_id` fisso. (2) Tirando appena giu' una pagina e lasciandola
risalire, l'animazione della panoramica veniva azzerata e la panoramica poi
risaliva dal basso: ora si azzera solo quando la pagina esce davvero.
(3) Dove si puo' passare di lato il gesto si decide dopo `SOGLIA_LATERALE`
(20 pt) invece di 10: l'inizio curvo del pollice non e' piu' preso per uno
scorrimento verticale. (Poi cambiato due volte. Con 20 pt un trascinamento
lento non ci arrivava entro `ATTESA_GESTO` e diventava un tocco normale; la
regola "verticale solo se 1,5 volte piu' verticale" faceva prendere per un
passaggio di lato gli scorrimenti del pollice sul lato destro, che salendo
piega a sinistra. Ora: dove si passa di lato il verso si decide dopo
`SOGLIA_LATERALE` = 24 pt, con la direzione di tutto il tratto; dopo 10 pt il
dito e' gia' un gesto (`InAttesa { mosso: true }`), quindi non scade in un
tocco normale anche se va piano, e se si solleva prima non fa niente.)
Scorrimento: egui spalma su ~0,1 s ogni spostamento di rotella oltre 8 punti
(come per la rotella del mouse), quindi con il dito veloce lo scorrimento
arrivava in ritardo e sembrava lento. Ora `smista_gesti` manda una rotella
con `TouchPhase::Start` all'inizio dello scorrimento (e la `End` al
fotogramma dopo la fine, o dello slancio): per egui e' un dito sul touchpad
e ogni spostamento arriva subito.
Slancio (abbrivio dopo un gesto rapido): `ATTRITO_SLANCIO` = 2 al secondo
(prima 3,5), cioe' un gesto a 2000 punti/s prosegue per circa 1000 punti in
circa 2 s, come lo scorrimento di egui stesso (`KineticScrollStyle::decay_time`
0,5 s). Lo slancio prima si fermava quasi subito perche' egui da' la rotella
all'area sotto il puntatore e, alzato il dito, Android toglie il puntatore:
`Gesto::Slancio` ricorda dove il dito e' stato sollevato e a ogni passo manda
`PointerMoved` li', poi `PointerGone` alla fine. Un dito appoggiato mentre
scorre ancora veloce (`SLANCIO_CHE_SI_FERMA`, 150 punti/s) lo ferma e basta:
sollevato subito non tocca la scheda sotto (`InAttesa { mosso: true }`).
Poi: Impostazioni nello stesso ordine del menu del computer (Aspetto,
Cartella e backup, Rete locale, Applicazione), Tema e Lingua come scelte di
uguale larghezza, niente Esci su Android, "Esporta come testo…" nel menu della
scheda della panoramica; regole grafiche comuni (vedi docs/ARCHITETTURA.md,
"Regole grafiche comuni"): margine 8 per schede e nota, titoli delle pagine
18, campi alti come i pulsanti, lineetta di trascinamento unica.
Quando un gesto diventa Pannello, ChiudiPagina, Laterale o Sfoglia si manda
a egui `PointerGone`: prima egui credeva il dito ancora fermo li', e cio' che
scorreva sotto (il "+" della panoramica, la barra di scorrimento del
Cestino) si illuminava come al passaggio del mouse.
Ricerca del Cestino: `StatoInterfaccia::ricerca_cestino`, campo
`ricerca::campo_cestino` (pagina Cestino sul telefono, sezione Cestino del
menu sul computer); `Applicazione::ricerca_per(id)` dice quale ricerca vale
per una nota (evidenziazione, note accanto, Indietro). La ricerca in cima
non mostra piu' "Anche N nel cestino".

24/11/2026, con Android 16 (API 36) come riferimento, su Android 17: Indietro
(tasto e pulsante della barra di navigazione) chiude la panoramica, mostra
"Premere di nuovo Indietro per uscire" e al secondo manda l'app in secondo
piano; con la tastiera aperta la chiude soltanto.

Da provare: scrittura a scorrimento (si attiva nelle impostazioni della
tastiera Samsung), gesto dal bordo con la navigazione a gesti di Android.
