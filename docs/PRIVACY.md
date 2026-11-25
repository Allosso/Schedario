# Informativa sulla privacy di Schedario

*In vigore dal 25/11/2026. English version below.*

Schedario è un programma di note per computer (Windows, Linux, macOS) e per
Android. Questa informativa descrive quali dati tratta e dove vanno.

## In breve

- **Nessun dato arriva allo sviluppatore.** Schedario non ha account, server,
  pubblicità, statistiche d'uso, tracciamento o invio automatico di errori.
- **Le note restano sui dispositivi dell'utente.** Escono solo se l'utente
  attiva la sincronizzazione in rete locale (cifrata) o l'assistente AI, oppure
  se esporta un backup o una nota.
- Schedario non si collega a Internet.

## Dati salvati sul dispositivo

- **Note**: titolo, testo, colore, data di modifica, versioni precedenti
  (cronologia) e note nel cestino.
  - Computer: nella cartella scelta dall'utente (all'inizio `Documenti/Schedario`).
  - Android: nella memoria privata dell'app, che le altre app non possono
    leggere. Il backup automatico di Android verso il cloud è disattivato per
    Schedario.
- **Preferenze**: lingua, tema, dimensione del testo, posizione delle finestre,
  nome e identificativo casuale del dispositivo (servono alla sincronizzazione).
- **Impostazioni della rete locale**, compreso il codice di gruppo, nella
  cartella delle note.

Disinstallando l'app su Android, note e preferenze vengono eliminate. Sul
computer restano nella cartella scelta finché l'utente non le cancella.

## Sincronizzazione in rete locale (facoltativa, spenta all'inizio)

Quando l'utente la attiva, Schedario scambia le note con gli altri dispositivi
**della stessa rete locale** (per esempio lo stesso Wi-Fi) che usano **lo stesso
codice di gruppo**. Vengono scambiati le note (anche quelle nel cestino), il
nome del dispositivo e il suo identificativo casuale.

- Tutto viaggia cifrato e autenticato (ChaCha20-Poly1305, chiave ricavata dal
  codice di gruppo con Argon2id). Chi non conosce il codice non può leggerlo.
- I dati vanno direttamente da un dispositivo all'altro, senza passare da
  server o da Internet.
- Per trovare gli altri dispositivi Schedario invia nella rete locale brevi
  annunci cifrati ogni 3 secondi (porta UDP 47810); lo scambio usa la porta TCP
  47811.

## Assistente AI (facoltativo, spento all'inizio, solo computer)

Quando l'utente lo attiva, Schedario apre un servizio raggiungibile **solo dallo
stesso computer** (indirizzo 127.0.0.1, mai dalla rete) e protetto da un codice.
Un programma di assistenza AI installato dall'utente e collegato con quel codice
(per esempio Claude Code) può leggere e cercare le note; può modificarle solo se
l'utente attiva anche "Permetti modifiche".

Ciò che quel programma fa con le note (per esempio inviarle al servizio AI che
usa) dipende dal programma stesso e dalla sua informativa, non da Schedario.

## Azioni dell'utente

- **Appunti**: Schedario legge o scrive gli appunti del sistema solo quando
  l'utente usa Copia, Taglia o Incolla.
- **Backup ed esportazione**: i file vanno solo dove l'utente sceglie.

## Permessi su Android

| Permesso | A cosa serve |
|---|---|
| Accesso alla rete (`INTERNET`) | Necessario ad Android per qualsiasi collegamento, anche solo in rete locale. Schedario non si collega a Internet. |
| Stato della rete e del Wi-Fi | Sapere se il telefono è in una rete locale. |
| Ricezione degli annunci (`CHANGE_WIFI_MULTICAST_STATE`) | Ricevere gli annunci degli altri dispositivi del gruppo. |
| Vibrazione | Breve vibrazione quando si tiene premuto. |

## Minori

Schedario non raccoglie dati di nessuno, quindi neanche di minori.

## Modifiche e contatti

Le modifiche a questa informativa compaiono in questo file, nel repository del
progetto su GitHub, con la data di entrata in vigore. Per domande: la sezione
"Issues" del repository, https://github.com/Allosso/schedario/issues.

---

# Schedario privacy policy

*Effective 25 November 2026.*

Schedario is a note-taking program for computers (Windows, Linux, macOS) and
Android. This policy describes which data it handles and where it goes.

## Summary

- **No data reaches the developer.** Schedario has no accounts, servers, ads,
  usage statistics, tracking or automatic error reports.
- **Notes stay on the user's devices.** They leave only if the user turns on
  local network sync (encrypted) or the AI assistant, or exports a backup or a
  note.
- Schedario does not connect to the Internet.

## Data stored on the device

- **Notes**: title, text, colour, modification date, previous versions
  (history) and notes in the trash.
  - Computer: in the folder chosen by the user (initially `Documents/Schedario`).
  - Android: in the app's private storage, which other apps cannot read.
    Android's automatic cloud backup is turned off for Schedario.
- **Preferences**: language, theme, text size, window positions, device name
  and a random device identifier (used by sync).
- **Local network settings**, including the group code, in the notes folder.

Uninstalling the app on Android deletes notes and preferences. On a computer
they stay in the chosen folder until the user deletes them.

## Local network sync (optional, off by default)

When the user turns it on, Schedario exchanges notes with the other devices
**on the same local network** (for example the same Wi-Fi) that use **the same
group code**. It exchanges the notes (including those in the trash), the device
name and its random identifier.

- Everything is encrypted and authenticated (ChaCha20-Poly1305, key derived
  from the group code with Argon2id). Without the code it cannot be read.
- Data goes directly from one device to another, never through servers or the
  Internet.
- To find the other devices Schedario sends short encrypted announcements on
  the local network every 3 seconds (UDP port 47810); the exchange uses TCP
  port 47811.

## AI assistant (optional, off by default, computers only)

When the user turns it on, Schedario opens a service reachable **only from the
same computer** (address 127.0.0.1, never from the network) and protected by a
code. An AI assistant program installed by the user and connected with that
code (for example Claude Code) can read and search the notes; it can change
them only if the user also turns on "Allow changes".

What that program does with the notes (for example sending them to the AI
service it uses) depends on that program and its own privacy policy, not on
Schedario.

## User actions

- **Clipboard**: Schedario reads or writes the system clipboard only when the
  user uses Copy, Cut or Paste.
- **Backup and export**: files go only where the user chooses.

## Android permissions

| Permission | Purpose |
|---|---|
| Network access (`INTERNET`) | Required by Android for any connection, even local network only. Schedario does not connect to the Internet. |
| Network and Wi-Fi state | Knowing whether the phone is on a local network. |
| Receiving announcements (`CHANGE_WIFI_MULTICAST_STATE`) | Receiving the announcements of the other devices in the group. |
| Vibration | Short vibration on touch and hold. |

## Children

Schedario collects no one's data, so no children's data either.

## Changes and contact

Changes to this policy appear in this file, in the project's GitHub
repository, with the effective date. For questions: the repository's "Issues"
section, https://github.com/Allosso/schedario/issues.
