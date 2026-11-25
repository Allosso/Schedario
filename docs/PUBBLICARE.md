# Pubblicare Schedario

Cosa è già pronto e cosa resta da fare per pubblicare Schedario su GitHub e sul
Play Store. La parte tecnica e quella delle licenze sono fatte; restano gli
account, la chiave di firma e le scelte che spettano all'autore (segnate con ☐).

## Già pronto

- **Licenza** MIT (`LICENSE`). Tutte le librerie incluse hanno licenze
  permissive (elenco in `docs/LICENZE-TERZE-PARTI.md`).
- **Testi completi delle licenze** di librerie e caratteri inclusi, come
  chiedono MIT, BSD e Apache-2.0: `assets/licenze/licenze.txt`, generato da
  `script/genera-licenze.ps1` (da rieseguire quando cambiano le dipendenze). È
  dentro il programma (Informazioni → Licenze, anche sul telefono) e viene
  copiato in `dist/LICENZE.txt` insieme all'eseguibile.
- **Informativa sulla privacy**: `docs/PRIVACY.md` (italiano e inglese).
- **`.gitignore`**: fuori dal repository compilati, `dist/`, la libreria
  Android copiata, le cartelle di Gradle e qualsiasi chiave di firma (`*.jks`,
  `*.keystore`, `firma.properties`). **`.gitattributes`**: fine riga corrette
  per gli script (`.sh` con LF).
- **Compilazione automatica su GitHub** (`.github/workflows/`):
  - `verifica.yml`: test su Windows, Linux e macOS a ogni invio;
  - `rilascio.yml`: con un'etichetta `v1.0.0` compila i tre programmi e crea
    una Release in **bozza** con i file allegati (zip per Windows e macOS,
    tar.gz per Linux, ciascuno con licenze). La bozza si controlla e si
    pubblica a mano.
  - Usano solo strumenti di GitHub (`actions/checkout`, `upload-artifact`,
    `download-artifact`, licenza MIT) e `gh`. Non sono ancora state eseguite:
    partono la prima volta dopo l'invio a GitHub.
- **Android**: versione da pubblicare separata da quella di sviluppo,
  Android 16 (API 36), nuovo Indietro, librerie a 64 bit allineate a 16 KB,
  pacchetto `.aab` e APK con `android\compila.ps1 -Pubblicazione`, icona
  512 × 512 (vedi `android/LEGGIMI.md`, "Versione da pubblicare").
- **Nessun dato personale nei file del progetto** (controllato: niente email,
  nomi di dispositivi o percorsi personali).

## GitHub

1. ☑ Account GitHub: **Allosso**.
2. ☑ Identificativo dell'app `io.github.allosso.schedario` in
   `android/progetto/gradle.properties`; `repository` in `Cargo.toml`.
3. ☑ Repository pubblico `Schedario` (https://github.com/Allosso/Schedario),
   **senza** README, licenza o .gitignore (ci sono già).
4. ☐ Invio dei file. Git è installato; il repository locale esiste, con nome e
   email privata di GitHub già impostati solo per questo progetto
   (`Allosso`, `360863769+Allosso@users.noreply.github.com`): l'email
   personale non finisce nella cronologia. Dalla cartella `Note`:

   ```powershell
   git remote add origin https://github.com/Allosso/Schedario.git
   git push -u origin main
   ```

   Al primo invio Git apre il browser per l'accesso a GitHub.
   Prima del primo invio, `git status` deve elencare solo sorgenti, documenti,
   `assets/` e gli script: nessun `.jks`, `firma.properties`, `target/` o `dist/`.
5. ☐ Su GitHub: descrizione breve del repository e argomenti (per esempio
   `notes`, `rust`, `egui`, `android`, `local-first`).
6. ☐ Prima versione: etichetta `v1.0.0` (uguale alla versione in `Cargo.toml`)

   ```powershell
   git tag v1.0.0
   git push origin v1.0.0
   ```

   poi, nella bozza creata da `rilascio.yml`, aggiungere l'APK firmato
   (`android\compila.ps1 -Pubblicazione`, serve la chiave) e pubblicare.
7. ☐ Chiave di firma Android: crearla, conservarne una copia di sicurezza,
   scrivere `%USERPROFILE%\.schedario\firma.properties` (vedi
   `android/LEGGIMI.md`). Mai nel repository.

Note per chi scarica:
- Windows mostra "Windows ha protetto il PC" per i programmi non firmati con un
  certificato: "Ulteriori informazioni → Esegui comunque".
- macOS blocca i programmi non firmati da Apple: tasto destro sull'app → Apri.

## Play Store

Requisiti verificati a novembre 2026 (cambiano spesso: ricontrollare nella
Play Console prima di pubblicare).

1. ☐ Account sviluppatore Google Play (quota unica di 25 dollari, verifica
   dell'identità, email di contatto pubblica) e accettazione del contratto di
   distribuzione per gli sviluppatori.
2. ☐ Identificativo definitivo e chiave (punti 2 e 7 sopra). Sul Play Store la
   chiave dell'app la conserva Google ("Play App Signing"); la propria è la
   chiave di caricamento.
3. ☐ Test chiuso: almeno **12 tester** iscritti per **14 giorni** di fila
   (account personali creati dopo il 13/11/2023), poi richiesta di accesso alla
   produzione.
4. ☐ Immagini: icona `dist\play-store\icona-512.png` (pronta, si rigenera con
   `cargo run --release --example icona_store -- dist\play-store\icona-512.png`),
   immagine in evidenza 1024 × 500, almeno 2 schermate del telefono.
5. ☐ Scheda dello Store: testi proposti qui sotto, da rivedere.
6. ☐ Moduli: risposte proposte qui sotto, da confermare con le domande
   effettive della Play Console.

### Scheda dello Store (bozza)

- **Nome** (max 30): `Schedario`
- **Descrizione breve** (max 80):
  - it: `Note a schede, sincronizzate e cifrate in rete locale. Senza account né cloud.` (78)
  - en: `Tabbed notes, encrypted sync on the local network. No account, no cloud.` (72)
- **Categoria**: Produttività.
- **Descrizione completa** (it):

  > Schedario è un programma di note semplice e veloce. Le note stanno in
  > schede, come in un raccoglitore: si aprono con un tocco, si sfogliano
  > trascinando di lato e si ritrovano con la ricerca.
  >
  > • Liste con caselle da spuntare, note fissate in cima, un colore per nota.
  > • Cronologia: ogni nota conserva le versioni precedenti.
  > • Cestino con ripristino.
  > • Sincronizzazione con i computer e gli altri telefoni della stessa rete
  >   locale (Wi-Fi), cifrata con un codice di gruppo. Nessun server, nessun
  >   cloud: le note vanno direttamente da un dispositivo all'altro.
  > • Backup in un file, da esportare e importare.
  > • Tema chiaro o scuro, in italiano e in inglese.
  >
  > Nessun account, nessuna pubblicità, nessun dato raccolto. Software libero
  > con licenza MIT; esiste anche la versione per Windows, Linux e macOS.

### Risposte proposte per i moduli

| Modulo | Risposta | Perché |
|---|---|---|
| Sicurezza dei dati: l'app raccoglie o condivide dati? | **No** | Nessun dato arriva allo sviluppatore o a terzi. La sincronizzazione è avviata dall'utente, va solo ai suoi dispositivi in rete locale ed è cifrata da un capo all'altro. *Da confermare con la guida di Google al momento della compilazione.* |
| Dati cifrati durante il trasferimento | Sì | ChaCha20-Poly1305. |
| Richiesta di eliminazione dei dati | Non applicabile | Nessun dato conservato dallo sviluppatore; disinstallando l'app si cancella tutto. |
| Informativa sulla privacy (URL) | `https://github.com/Allosso/Schedario/blob/main/docs/PRIVACY.md` | |
| Accesso all'app | Nessuna restrizione, nessun accesso richiesto | Non ci sono account. |
| Annunci pubblicitari | No | |
| Pubblico di destinazione | 13 anni e oltre | Sotto i 13 anni si applicano i requisiti del programma "Famiglie". |
| Classificazione dei contenuti (IARC) | Categoria "Utilità / Produttività"; nessun contenuto violento, sessuale, gioco d'azzardo; nessuna interazione tra utenti, nessuna condivisione della posizione, nessun acquisto | |
| App governativa, finanziaria, sanitaria, di notizie | No | |
| Permessi sensibili da dichiarare | Nessuno | `INTERNET`, stato della rete e del Wi-Fi, multicast e vibrazione sono permessi normali. |
