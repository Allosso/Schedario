# Schedario

> **Realizzato in collaborazione con [Claude Code](https://claude.com/claude-code)**,
> l'assistente di programmazione di Anthropic: l'autore ha deciso funzioni,
> interfaccia e priorità e ha provato ogni versione; Claude Code ha scritto il
> codice e la documentazione.

Note con schede per Windows, Linux, macOS e Android, sincronizzate e cifrate
tra i dispositivi della stessa rete locale. Nessun account, nessun cloud,
nessun abbonamento. Scritto in Rust con egui.

*English: a tabbed notes app for Windows, Linux, macOS and Android, with
encrypted sync between devices on the same local network. No account, no
cloud. Built in collaboration with Claude Code. Interface in Italian and
English; documentation in Italian.*

## Cosa fa

- **Schede**, come le linguette di un raccoglitore, con ricerca nel titolo e
  nel testo.
- **Liste da spuntare**, note fissate in cima, un colore per nota.
- **Cronologia** delle versioni di ogni nota e **cestino** con ripristino.
- **Widget fluttuanti** sul desktop e **nota rapida** da qualsiasi programma
  (Ctrl+Alt+N).
- **Sincronizzazione in rete locale** tra computer e telefoni, cifrata con un
  codice di gruppo, senza server.
- **Backup** in un file; ogni nota è anche un normale file `.txt`.
- Tema chiaro o scuro, italiano e inglese.
- Facoltativo, solo sul computer: accesso alle note per un **assistente AI**
  (per esempio Claude Code), dallo stesso computer e con un codice.

Descrizione completa: [docs/GUIDA.md](docs/GUIDA.md).

## Scaricare

Dalla pagina [**Releases**](https://github.com/Allosso/Schedario/releases):
si estrae l'archivio e si avvia il programma, senza installazione.

- **Windows** avvisa "Windows ha protetto il PC" (il programma non ha un
  certificato a pagamento): **Ulteriori informazioni → Esegui comunque**.
- **macOS**: tasto destro sull'app → **Apri**.
- **Android**: aprire l'APK e consentire l'installazione quando Android lo
  chiede.

## Compilare

Serve Rust (https://rustup.rs).

```sh
cargo run                            # prova in modalità sviluppo
.\script\compila-windows.ps1         # Windows: dist\Schedario.exe
sh script/compila-linux-macos.sh     # Linux e macOS: dist/
```

Per Android: [android/LEGGIMI.md](android/LEGGIMI.md).

## Documentazione

- [Guida completa](docs/GUIDA.md)
- [Rete locale](docs/RETE.md) e [assistente AI](docs/MCP.md)
- [Architettura del codice](docs/ARCHITETTURA.md)
- [Pubblicare una nuova versione](docs/PUBBLICARE.md)

## Privacy

Schedario non raccoglie dati e non si collega a Internet. Le note escono dal
dispositivo solo con la sincronizzazione in rete locale o l'assistente AI,
entrambi spenti finché non li si attiva. Dettagli: [docs/PRIVACY.md](docs/PRIVACY.md).

## Licenza

MIT ([LICENSE](LICENSE)). Tutte le librerie incluse hanno licenze permissive:
riepilogo in [docs/LICENZE-TERZE-PARTI.md](docs/LICENZE-TERZE-PARTI.md), testi
completi in [assets/licenze/licenze.txt](assets/licenze/licenze.txt) e nel
programma (Informazioni → Licenze).
