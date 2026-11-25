# Schedario: guida

Descrizione completa di come funziona Schedario, sul computer e sul telefono.
Per una presentazione breve vedere il [README](../README.md).

## Avvio

- **Windows**: apri `dist\Schedario.exe` (generato da `script\compila-windows.ps1`).
- **Linux / macOS**: esegui `sh script/compila-linux-macos.sh`, poi avvia
  `dist/schedario` (oppure `dist/Schedario.app` su macOS).

Al primo avvio le note vanno in `Documenti/Schedario`, senza domande. Se la
cartella Documenti è stata spostata (per esempio in OneDrive), viene usata
quella vera. Per usare un'altra cartella: **Cartella e backup → Apri un'altra
cartella**; dal riavvio successivo viene riaperta quella. Se la cartella usata
l'ultima volta non è più raggiungibile (per esempio un disco esterno
scollegato), Schedario lo dice e chiede quale usare, senza creare di nascosto
una cartella di note vuota.

## Cosa sa fare

**Finestra principale**: l'area di lavoro a sinistra, il menu a destra.
- Le **schede** delle note stanno in alto, come le linguette di un
  raccoglitore: quella aperta si unisce al riquadro della nota. Il **+**
  subito dopo l'ultima scheda crea una nota nuova (come Ctrl+N). Le schede si restringono quando
  sono tante; quando non entrano più compaiono le frecce per scorrerle.
  **Doppio clic** o **F2** per rinominare una nota direttamente sulla scheda.
  Il **tasto destro** su una scheda offre Rinomina, Apri widget, Fissa in alto,
  Cronologia, Sposta a sinistra/destra e Cancella.
- **Riordinare le schede**: tieni premuto su una scheda e trascinala; le altre
  si spostano per farle posto e al rilascio resta lì. Vicino alle frecce le
  schede scorrono. Con la tastiera: **Ctrl+Shift+←/→** sposta la scheda
  selezionata (fuori dai campi di testo, dove le stesse frecce selezionano le
  parole). Le note fissate restano sempre prima delle altre; una nota nuova va
  in fondo. Durante una ricerca il trascinamento è spento. L'ordine vale solo
  su questo computer: spostare una scheda non modifica la nota, la sua data o
  la cronologia, e non manda niente in rete.
- **Editor** con titolo e testo e salvataggio automatico.
  - Nella riga del titolo, a destra, le azioni su tutta la nota: **Apri
    widget** (anche **Ctrl+D**) e **Cancella**, con la scritta rossa, che
    sposta la nota nel cestino. Subito dopo, in basso al centro della nota,
    compare per 10 secondi un riquadro "Nota spostata nel cestino ·
    **Annulla**".
  - In basso a destra nella nota, il pulsante **☐**: mentre scrivi trasforma
    la riga del cursore in una voce da spuntare, altrimenti aggiunge una voce
    in fondo alla nota (stesso pulsante del telefono).
  - Sotto il titolo, come sul telefono: **● Colore** (il pallino ha il colore
    della nota, vuoto se non ne ha), **📌 Fissa** ("Fissata", con la puntina
    colorata, se la nota è fissata) e la data con l'orologio, "🕘 Modificata
    oggi alle 10:42": un clic apre la **Cronologia** (al passaggio del mouse
    anche la data di creazione).
  - Tutti i pulsanti hanno lo stile sobrio delle schede: fondo come quello
    su cui stanno, bordo sottile e neutro, angoli poco arrotondati; il colore
    solo dove dice qualcosa (rosso per Cancella e il Cestino, verde per
    Ripristina, il colore principale per l'azione più importante e per ciò
    che è attivo). Vale anche sul telefono.
- **Ricerca** in cima al menu, alla stessa altezza delle schede: filtra le
  schede per titolo e testo mentre scrivi ed evidenzia in giallo le parole
  trovate nel testo. Cerca solo tra le note: il cestino ha la sua ricerca
  (vedi sotto). Esc svuota la ricerca.
- Il pulsante **›** accanto alla ricerca riduce il menu a una striscia sottile
  e il testo prende tutta la larghezza; **‹** lo riapre e la lente riapre e
  cerca (anche Ctrl+F). Il programma ricorda se l'hai lasciato chiuso.
- La nota arriva fino in fondo alla finestra, alla stessa altezza del menu.
  A destra nella riga della data, in grigio sotto Apri widget e Cancella, i
  **contatori**: "Righe: 12 · Parole: 85" e, se la nota ha una lista, anche
  le voci spuntate ("· Spunte: 2 di 5"); con la finestra stretta si accorciano
  con "…" prima della data. Il salvataggio è automatico e non si scrive "Salvato"; i messaggi
  brevi (Annulla dopo Cancella o dopo aver preso l'ordine delle schede, note
  arrivate dalla rete, un errore di salvataggio) compaiono in un riquadro
  piccolo in basso al centro della nota e spariscono da soli (l'errore resta
  finché c'è).

**Liste di controllo**
- Una riga che inizia con `[ ]` diventa una casella da spuntare con un clic;
  spuntata diventa `[x]` e la riga appare barrata e in grigio. Funziona
  nell'editor e nei widget.
- Per creare una voce: scrivi `[ ] ` all'inizio della riga, oppure premi
  **Ctrl+L**, il pulsante **☐** in basso a destra nella nota o "Voce di lista"
  nel menu del tasto destro. **Invio** su una voce ne crea un'altra; Invio su
  una voce vuota chiude la lista.
- **Cancellando** il segno di una voce (Backspace subito dopo `[ ] ` o `[x] `)
  sparisce tutto il segno, non una parentesi alla volta: la voce torna testo
  normale.
- Nel file `.txt` resta scritto `[ ] latte` / `[x] pane`: si legge con qualsiasi
  editor e molti programmi di note riconoscono lo stesso formato.

**Note fissate**: la puntina tiene la nota prima delle altre nelle schede, con
una piccola puntina sulla scheda. Vale anche sugli altri computer collegati.

**Menu del tasto destro** nel testo e nei titoli: Taglia, Copia, Incolla,
Seleziona tutto e, nel testo, Voce di lista.

**Dimensione del testo**: Ctrl + rotella del mouse, Ctrl + e Ctrl - (Ctrl 0
torna al 100%), oppure i pulsanti in **Aspetto**. Vale per l'editor e per tutti
i widget ed è un'impostazione di ciascun computer.

**Cronologia delle versioni**: il pulsante con l'orologio mostra le versioni
precedenti della nota, con data e anteprima; **Ripristina questa versione** la
riporta indietro (il testo attuale resta anch'esso nella cronologia).
Schedario salva una versione ogni 5 minuti di scrittura, prima di ogni modifica
arrivata da un altro computer e prima di rileggere un file cambiato con un altro
editor. Tiene le ultime 50 versioni di ogni nota (quelle con più di 30 giorni
vengono tolte, ma le ultime 10 restano sempre). La cronologia resta su questo
computer e sparisce con l'eliminazione definitiva della nota.

**Nota rapida da qualsiasi programma**: **Ctrl+Alt+N** (Ctrl+Option+N su macOS)
apre una nota nuova come widget, con il cursore pronto, anche con Schedario
nascosto. Se chiudi il widget senza scrivere nulla, la nota vuota sparisce. In
**Applicazione** si disattiva oppure, con **Cambia combinazione**, si sceglie
un'altra combinazione (serve Ctrl o Alt insieme a una lettera, un numero, un
tasto F o lo spazio), utile se Ctrl+Alt+N è già usata da un altro programma. Funziona su Windows, macOS e Linux con X11; su
Wayland i programmi non possono registrare tasti globali, quindi lì conviene
associare la combinazione a Schedario dalle impostazioni del desktop.
- **Menu di destra**, con il logo in cima e sezioni retraibili, il cui stato aperto/chiuso viene ricordato:
  - **Cestino**: in cima il campo **Cerca nel cestino…**, che cerca solo tra
    le note cancellate (per esempio "pane": restano solo quelle che lo
    contengono, "Corrispondono alla ricerca: 2 su 9", e nella nota aperta la
    parola è evidenziata); sotto, l'elenco delle note cancellate. Un clic apre la nota nell'editor,
    dove in alto a destra ci sono **Ripristina** ed **Elimina definitivamente**.
    **Svuota cestino**, in rosso, le elimina tutte. Prima di eliminare viene
    chiesta una conferma, perché l'eliminazione vale anche sugli altri dispositivi
    collegati.
  - **Aspetto**: **Tema** con tre scelte, **Sistema** (segue il tema chiaro o
    scuro di Windows e cambia insieme a lui), **Chiaro** o **Scuro**, e
    dimensione del testo; per la finestra e per tutti i widget.
  - **Cartella e backup**: apri un'altra cartella, esporta o importa un backup
    JSON. Una singola nota si esporta come `.txt` dal suo menu (clic destro
    sulla scheda, **Esporta come testo…**).
  - **Rete locale**: sincronizzazione con gli altri dispositivi (vedi sotto).
    In cima l'interruttore con lo stato, poi nome, codice (**Mostra · Copia ·
    Nuovo codice**, come nell'Assistente AI), dispositivi collegati e, in fondo,
    gli indirizzi aggiunti a mano.
  - **Assistente AI (MCP)**: permette a un assistente come Claude Code di
    leggere e cercare le note e, con **Permetti modifiche**, di crearle e
    cambiarle (mai eliminarle definitivamente). Spento all'inizio, solo da
    questo computer, con un codice di accesso. Dettagli in [docs/MCP.md](MCP.md).
  - **Applicazione**: lingua (italiano/inglese), **Avvia con il sistema**,
    **Nota rapida con Ctrl+Alt+N**, informazioni, esci.

**Area di notifica**
- L'icona di Schedario vicino all'orologio resta sempre disponibile. Un clic
  mostra la finestra principale. Il menu del tasto destro offre **Mostra
  Schedario**, **Nuova nota** ed **Esci**. È utile quando sono aperti solo i
  widget: la finestra è nascosta e i widget non compaiono nella barra delle
  applicazioni.
- Su Linux usa il protocollo StatusNotifierItem: funziona su KDE, su Ubuntu e su
  GNOME con l'estensione AppIndicator.

**Avvio con il sistema**
- Se attivato, Schedario si apre all'accesso con i widget dove li avevi lasciati.
  Usa il meccanismo standard di ogni sistema, senza permessi di amministratore:
  il registro utente su Windows, `~/.config/autostart` su Linux e
  `~/Library/LaunchAgents` su macOS.

**Widget fluttuanti**
- **Apri widget** a destra nella riga del titolo, accanto a Cancella (oppure Ctrl+D, o il menu
  del tasto destro sulla scheda) mette la nota sul desktop. Puoi aprirne quanti ne vuoi.
- Il **titolo** della nota è nella barra in alto: un clic per cambiarlo. Se è
  lungo viene accorciato con "…".
- Il **pallino colorato** dà un colore alla nota (giallo, arancione, rosa,
  viola, azzurro, verde o nessuno). Il colore tinge tutto il widget, compare
  come striscia sulla scheda e si sceglie anche dalla finestra principale,
  accanto a **Cancella**. Viaggia con la nota sugli altri computer.
- **Sposta** il widget trascinando la barra in alto. **Ridimensionalo**
  trascinando l'angolo in basso a destra.
- Il pallino pieno o vuoto attiva o disattiva il primo piano, il trattino riduce il widget a
  una barra (anche con doppio clic) e la croce lo chiude. La nota resta.
- **Note** riapre la finestra principale su quella nota.
- Chiudere la finestra principale lascia aperti i widget. Chiudere l'ultimo
  widget fa ricomparire la finestra. Al riavvio i widget tornano dove li avevi lasciati.

**Descrizioni**: fermando il mouse per mezzo secondo su un pulsante o su un
campo compare una breve descrizione di cosa fa (e della scorciatoia, se c'è).
Sui pulsanti disattivati la descrizione spiega perché lo sono.

**Carattere**: tutto il programma usa Inter (licenza OFL), incluso
nell'eseguibile; i titoli delle note sono in Inter SemiBold, più grandi. Nel
testo delle note le righe sono un po' più distanziate (1,45 volte la dimensione
del carattere) per leggere meglio i testi lunghi.

**Titolo della nota**: finché è vuoto compare "Inserisci un titolo", e la
scheda usa la prima riga del testo (lo ricorda anche la descrizione al
passaggio del mouse). Una linea sottile sotto il titolo compare al passaggio
del mouse e diventa blu mentre scrivi.

**Date e orari** sono mostrati nell'ora locale del computer, presa dal sistema
operativo, ora legale compresa.

**Scorciatoie**: `Ctrl+N` nuova nota · `Ctrl+Alt+N` nota rapida da qualsiasi
programma · `Ctrl+F` cerca · `Ctrl+S` salva subito · `Ctrl+L` voce di lista · `Ctrl+D` apri come widget ·
`Ctrl+Shift+←/→` sposta la scheda ·
`Ctrl` + rotella, `Ctrl +`/`Ctrl -`/`Ctrl 0` dimensione del testo · `F2` rinomina
(su macOS `Cmd` al posto di `Ctrl`).

## Dove finiscono le note

```text
Cartella scelta/
  Note/Nota-<id>.txt        una nota per file: titolo, riga vuota, testo
  Cestino/Nota-<id>.txt     note nel cestino
  .schedario/indice.json     date e copia di recupero di ogni nota
  .schedario/sessione.json   widget aperti e stato della finestra
  .schedario/rete.json       impostazioni della rete per questa cartella
  .schedario/storico/<id>.json  versioni precedenti di ogni nota
  .schedario.lock            impedisce di aprire la stessa cartella due volte
```

- I file `.txt` si possono leggere e modificare con qualsiasi editor **a
  programma chiuso**: alla riapertura le modifiche vengono rilette.
- Un file cancellato per errore viene ricreato dalla copia nell'indice.
- Un file `Nota-<id>.txt` aggiunto a mano viene adottato come nuova nota.
- Il salvataggio parte dopo 450 ms di pausa nella scrittura. Se fallisce, le
  modifiche restano in memoria e si riprova. Il programma non si chiude
  finché non è riuscito a salvare.
- **Importa backup** aggiunge copie delle note e non sovrascrive mai quelle
  esistenti. Accetta anche i backup del prototipo "Notte".

Le preferenze generali (lingua, tema, ultima cartella) sono in `%APPDATA%\Schedario`
(Windows), `~/.config/schedario` (Linux) o
`~/Library/Application Support/Schedario` (macOS).

## Rete locale

Premi **Nuovo codice** su un dispositivo, copia lo **stesso codice di gruppo** sugli
altri e attiva **Sincronizza con altri dispositivi**. Nome e codice si salvano da
soli quando lasci il campo: non c'è un pulsante "Applica". I computer si trovano da soli
e si scambiano le note in pochi secondi. Il traffico è cifrato con
ChaCha20-Poly1305, con una chiave ricavata dal codice tramite Argon2id.

**Prendi l'ordine delle schede da…** (in Rete locale) apre una finestra con i
computer collegati: scegline uno e, dopo **Conferma**, questo computer usa
l'ordine delle schede di quello. Cambia solo questo computer, mai l'altro; le
note non cambiano. Per qualche secondo in basso nella nota c'è **Annulla**. Le
note che esistono solo qui restano in fondo; se l'altro computer ha una
versione precedente di Schedario, un messaggio lo dice.

Se la stessa nota viene modificata in parallelo su due computer, nessun testo va
perso: la versione che perde diventa una **copia in conflitto**, segnalata in
rosso. Mentre scrivi, le modifiche in arrivo per quella nota aspettano che tu
faccia una pausa. Su Windows, alla prima attivazione, consenti l'accesso nella
finestra del firewall (reti private).
Tutti i dettagli sono in [docs/RETE.md](RETE.md).

## Android (in sviluppo)

Sul telefono Schedario usa lo stesso codice del computer: note, liste, cestino,
cronologia, ricerca e sincronizzazione sono le stesse. L'app si chiama per ora
"Schedario (sviluppo)".

**Com'è fatta sul telefono in verticale**:
- la nota usa tutta la larghezza, senza riquadro attorno. La riga delle schede
  va da bordo a bordo e scorre con il dito in modo continuo, come una pagina;
  una piccola freccia sul bordo dice che ci sono altre schede da quella parte,
  e la scheda aperta si vede sempre intera;
- **trascinando il dito di lato sulla nota** si passa alla nota accanto, come
  sfogliando un quaderno: da destra a sinistra la successiva, da sinistra a
  destra la precedente. La nota segue il dito e dal bordo entra l'altra,
  intera come quando è aperta (titolo, riga della data, caselle e voci
  fatte); oltre un terzo dello schermo, o con un gesto
  veloce, lo scorrimento si completa da solo, altrimenti la nota torna al suo
  posto. Mentre scrivi è spento;
- le **schede** sono linguette, come in un raccoglitore: quella aperta ha
  fondo e bordo e si unisce alla nota, le altre sono solo il titolo sopra una
  linea sottile. Il colore della nota è una striscia in cima alla linguetta.
  Lo stesso stile vale sul computer;
- in cima alla nota ci sono il titolo (toccandolo lo si rinomina) e una riga
  in cui ogni pezzo si tocca: `[●] [📌] oggi 21:15      🗑`. Righe, parole
  e spunte non sono ripetute qui: si vedono nelle schede della panoramica e
  del Cestino.
  Colore, Fissa e Cestino sono pulsanti quadrati con il bordo, grandi come un
  dito e ben distanziati. Il **pallino** è il colore della nota e lo cambia; la **puntina** è
  colorata se la nota è fissata e la fissa o la toglie; la **data** apre la
  Cronologia; il **Cestino**, rosso in alto a destra, lontano dal pollice,
  mette la nota nel cestino. Per 10 secondi in basso compare un riquadro
  chiaro, "«Spese» nel cestino", con un grande **ANNULLA** e una linea che si
  accorcia con il tempo che resta;
- in basso, sopra la nota, ci sono due pulsanti tondi e una maniglia:
  - **+** a sinistra crea una nota nuova; si crea anche con il **+** in fondo
    alla riga delle schede (si vede scorrendo fino all'ultima) e **sfogliando
    oltre l'ultima nota**, dove compare "+ Nuova nota". Una nota lasciata vuota
    (senza titolo né testo) viene scartata appena ne apri un'altra;
  - **☐** a destra fa le liste da spuntare: mentre scrivi trasforma la riga
    del cursore in una voce (o la riporta a testo normale), mentre leggi
    aggiunge una voce vuota in fondo alla nota e apre la tastiera lì. Per
    esempio, in "Spese" lo tocchi e scrivi subito "Uova". Mentre scrivi resta
    solo lui, sopra la tastiera;
  - la **maniglia** al centro, toccata o trascinata verso l'alto (da tutta la
    fascia in basso tra il + e ☐, anche partendo piano), apre la panoramica,
    che sale seguendo il dito. Se la nota è corta, o l'hai già letta fino in
    fondo, basta trascinare verso l'alto da qualsiasi punto della metà bassa
    dello schermo; in una nota lunga, a metà lettura, lo stesso gesto fa
    scorrere il testo. Si chiude allo stesso modo trascinando verso il basso la testata o,
    quando è in cima, qualsiasi punto della griglia. La **panoramica**
    mostra tutte le note come schede grandi: a sinistra titolo, prime righe
    e in fondo la data di modifica ("Nota vuota" se non c'è ancora niente);
    a destra, divisi da una linea sottile, i contatori, con l'etichetta sopra
    il numero (Righe 12, Parole 85 e, se la nota ha una lista, Spunte 2/5). In alto ci
    sono **+** (nuova nota), **Impostazioni** (Aspetto, Rete locale, Cartella e
    backup, Applicazione) e **⌄**, che torna alla nota; si torna anche
    trascinando la testata verso il basso. Sotto c'è la **ricerca**, sempre
    aperta. In fondo c'è il **Cestino**, grande. Cestino e Impostazioni sono
    pagine a schermo intero ai due lati della panoramica:
    `[ Cestino ] ← [ Panoramica ] → [ Impostazioni ]`. Trascinando il dito
    verso destra sulla panoramica entra il Cestino, verso sinistra le
    Impostazioni, seguendo il dito; nel verso opposto si torna alla
    panoramica (oltre un terzo dello schermo, o con un gesto veloce, si passa
    all'altra; anche ⚙, il pulsante Cestino, «‹» e Indietro le fanno
    scorrere). Trascinando verso il basso la testata della pagina (o tutta la
    pagina, quando è in cima) si torna alla nota da cui si era partiti, anche
    se è una nota del cestino.
    Il **Cestino** mostra le note cancellate come schede, come la panoramica
    (anteprima, data, righe, parole e spunte), con in cima il suo campo
    **Cerca nel cestino…**: cerca solo tra le note cancellate, mentre la
    ricerca della panoramica cerca solo tra le note. Un tocco apre la nota a pagina
    intera e funziona come per le note principali: trascinando di lato si
    passa alle altre note del cestino; dal basso verso l'alto (o con
    Indietro) risalgono le schede del Cestino; trascinando in giù la testata
    si torna alle note principali. Il titolo ha la sua riga e sotto ci sono
    **Ripristina** ed **Elimina definitivamente**, che anche qui offre
    ANNULLA invece della conferma e passa alla nota del cestino accanto.
    Tenendo premuta una scheda del Cestino compare un menu con **Ripristina**
    ed **Elimina definitivamente**: la nota sparisce e per 10 secondi c'è
    "«Spese» eliminata · ANNULLA" (l'eliminazione vera avviene solo dopo, o
    subito se l'app va in secondo piano). **Svuota cestino** chiede prima
    conferma e poi offre anche lui ANNULLA per 10 secondi.
    Un tocco su una scheda la apre; tenendola premuta si apre il suo menu
    (fissarla, cronologia, cancellarla); tenendo il dito fermo e poi
    trascinandola si cambia l'ordine delle note, o la si lascia sul Cestino;
- **tenendo il dito fermo su una scheda** in alto per circa mezzo secondo e
  poi trascinandola, la scheda si solleva (con un'ombra) e segue il dito:
  lasciata più a destra o più a sinistra nella riga cambia posto (vicino al
  bordo la riga scorre), lasciata sul **Cestino** in alto a destra, che
  diventa rosso, la nota va nel cestino. Se il dito si muove subito, la riga
  scorre e basta; tenendola premuta senza muoverla si apre il suo menu;
- nella pagina del **Cestino**, toccando una scheda leggi la nota, scorri di
  lato tra le altre del cestino e scegli Ripristina o Elimina
  definitivamente (Indietro riporta alle schede del Cestino); c'è anche
  Svuota cestino;
- mentre scrivi, in cima restano solo titolo e testo; sopra la tastiera c'è
  una **barra** con **Annulla | Ripeti** e chiudi tastiera. Trascinando la
  barra verso il basso (ha una lineetta al centro) la tastiera si chiude;
- **tenendo premuto sul testo** si apre una striscia piccola sopra la parola
  (sotto, se sopra non c'è posto): **Taglia · Copia · Incolla · …**. "…" mostra
  Seleziona tutto, Annulla, Ripeti e Voce di lista. La striscia sta sempre
  sopra la tastiera e toccarla non la chiude. La parola tenuta premuta si
  seleziona e sotto l'inizio e la fine compaiono due **maniglie** a goccia:
  trascinandole si allarga o si restringe la selezione, anche su più righe
  (in "La città è bella", tenendo premuto su "città" e portando la maniglia di
  destra fino a "bella", si seleziona "città è bella"). Mentre trascini la
  striscia si nasconde e ricompare quando alzi il dito. Sul testo, invece,
  trascinare il dito fa sempre scorrere la nota, anche se il dito era rimasto
  fermo un attimo: si seleziona solo tenendo premuto;
- **Indietro** chiude per prima cosa una pagina (Impostazioni, Cestino; nel
  Cestino, se c'è una ricerca, prima svuota quella) o la panoramica, e si
  torna alla nota; poi, se nella nota ci sono parole evidenziate, svuota la
  ricerca (per una nota del cestino quella del Cestino);
  sulla nota compare "Premi di nuovo Indietro per uscire", e un secondo
  Indietro entro due secondi manda l'app in secondo piano;
- con la tastiera chiusa il **cursore** lampeggiante non si vede: non si può
  scrivere, quindi distrarrebbe soltanto;
- toccando una **casella** delle liste la voce si spunta senza aprire la
  tastiera e senza spostare il cursore;
- con una ricerca attiva, sopra il testo compare "Parola 2 di 5" con **‹ ›**
  per passare da una parola trovata all'altra; la nota scorre fino a lei;
- tenendo premuto, il telefono **vibra** leggermente quando compare il menu o
  la descrizione;
- la **ricerca** sta in cima alla panoramica (anche con Ctrl+F): scrivendo
  "pane" restano solo le schede delle note che lo contengono, e nella nota
  aperta la parola è evidenziata. Il Cestino ha la sua ricerca, che vale
  per le note cancellate e per lo scorrimento di lato tra loro;
- **tenendo premuto un pulsante** compare la sua descrizione, per esempio
  "Usa sempre i colori chiari"; sollevando il dito il pulsante non viene premuto;
- pulsanti, caselle delle liste e campi sono più grandi, per il dito;
- **allargando o stringendo due dita** sul testo cambia la sua dimensione;
- in **Aspetto** si sceglie **Sistema** (il tema chiaro o scuro segue
  quello di Android e cambia insieme a lui), **Chiaro** o **Scuro**. Barra di
  stato e barra di navigazione prendono i colori dell'app;
- i testi non citano le scorciatoie da tastiera, e in Applicazione non c'è
  **Esci**: Indietro manda l'app in secondo piano e Android la chiude quando
  serve;

- trascinando il dito **in verticale** sul testo la nota scorre (anche mentre
  scrivi: il cursore resta dov'era); per selezionare si tiene premuto;
- tenendo premuto sul testo si apre il menu Taglia/Copia/Incolla senza aprire
  la tastiera; un tocco fuori dal menu, o Indietro, lo chiude senza aprirla.

L'app si usa **solo in verticale**: in orizzontale la tastiera lascia troppo
poco spazio per scrivere. Su un tablet (almeno 720 punti di larghezza) la
disposizione è quella del computer: lavoro a sinistra, menu sempre aperto a
destra.

Altre differenze rispetto al computer:

- le note stanno nella **memoria riservata dell'app**: le altre app non le
  vedono. Per averne una copia usa **Esporta backup**, che apre la finestra
  di Android per scegliere dove salvare (per esempio Download o Drive);
  **Importa backup** e **Esporta come testo…** (tenendo premuta la scheda
  nella panoramica) funzionano allo stesso modo;
- la tastiera del telefono funziona in tutti i campi: accenti tenendo
  premuto, suggerimenti e correzioni, maiuscola a inizio frase. Invio su una
  voce di lista ne crea un'altra, come sul computer;
- **tenendo premuto** sul testo si apre il menu Taglia/Copia/Incolla/Seleziona
  tutto e si seleziona la parola sotto il dito; Copia e Incolla usano gli
  appunti di Android;
- il tasto **Indietro** chiude, in quest'ordine, la finestra di messaggio, la
  rinomina, il menu e la ricerca; se non c'è niente da chiudere manda l'app
  in secondo piano senza chiuderla;
- **tenendo premuta l'icona** dell'app compare **Nuova nota**;
- le note si salvano anche appena passi a un'altra app o spegni lo schermo;
- non ci sono widget fluttuanti, icona nell'area di notifica, nota rapida con
  Ctrl+Alt+N, avvio con il sistema e server per gli assistenti AI.

Come compilarla e provarla: [android/LEGGIMI.md](../android/LEGGIMI.md).

## Sviluppo

Struttura del codice e scelte tecniche: [docs/ARCHITETTURA.md](ARCHITETTURA.md).

```sh
cargo run                 # avvia in modalità sviluppo
cargo test                # test unitari e di integrazione (archivio, rete, conflitti)
cargo build --release     # eseguibile ottimizzato in target/release/
```

Opzioni di avvio:

```sh
schedario --notes-dir <cartella>   # apre direttamente questa cartella di note
schedario --data-dir <cartella>    # preferenze in un'altra cartella (versione portatile)
schedario --library                # mostra sempre la finestra principale
```

**Strumenti su questo PC Windows**: Rust stable con toolchain **GNU**
(`x86_64-pc-windows-gnu`, installata con rustup in `%USERPROFILE%\.cargo`) e
**MinGW-w64** (WinLibs, in `%LOCALAPPDATA%\Programs\mingw64`, aggiunto al
PATH utente). Sono entrambi open source. Si è scelta la toolchain GNU invece
di MSVC per non dover installare Visual Studio Build Tools, che richiede di
accettare una licenza Microsoft.

**Linux**: per compilare non servono pacchetti `-dev`. Per eseguire servono
OpenGL/EGL, libxkbcommon e X11 o Wayland, già presenti in qualsiasi desktop, più
`xdg-desktop-portal` per le finestre di scelta delle cartelle. Su Wayland il
gestore delle finestre può ignorare la posizione dei widget e il "sempre in
primo piano": è una limitazione di Wayland, non del programma.
