//! Il ciclo di ogni fotogramma: tema, scorciatoie, finestre e chiusura.

use std::sync::Arc;
use std::time::Duration;

use egui::{Key, Modifiers, ViewportCommand, ViewportId};

use super::area_notifica::{self, AreaNotifica, Comando};
#[cfg(not(target_os = "android"))]
use super::scorciatoia_globale::ScorciatoiaGlobale;
#[cfg(not(target_os = "android"))]
use crate::archivio::preferenze::Combinazione;
use super::tema::{self, Tavolozza};
#[cfg(not(target_os = "android"))]
use super::widget_fluttuante;
use super::{avvio, principale};
use crate::applicazione::Applicazione;
#[cfg(target_os = "android")]
use crate::applicazione::Pagina;

pub struct Programma {
    app: Applicazione,
    tema_applicato: Option<bool>,
    /// Visibilita' della finestra principale effettivamente applicata.
    finestra_mostrata: bool,
    fotogrammi: u64,
    chiusura_inviata: bool,
    area: Option<AreaNotifica>,
    /// Lingua con cui e' stato creato il menu dell'area di notifica.
    lingua_area: String,
    ridisegna: area_notifica::Ridisegna,
    /// Richiesta di portare davanti la finestra (anche se gia' visibile).
    porta_davanti: bool,
    /// Scorciatoia globale per la nota rapida, se attiva e disponibile.
    #[cfg(not(target_os = "android"))]
    scorciatoia: Option<ScorciatoiaGlobale>,
    /// Combinazione registrata l'ultima volta che si e' provato (`Some(None)`:
    /// scorciatoia spenta). Cambia quando l'utente la disattiva o ne sceglie un'altra.
    #[cfg(not(target_os = "android"))]
    scorciatoia_voluta: Option<Option<Combinazione>>,
    /// Collegamento con la tastiera del telefono.
    #[cfg(target_os = "android")]
    ponte: crate::piattaforma::android::tastiera::Ponte,
    /// Spazio coperto in basso nell'ultimo fotogramma (tastiera o barra), in pixel.
    #[cfg(target_os = "android")]
    basso_prima: i32,
    /// Tocchi: quando e' cominciata la pressione in corso, e se l'ultima e'
    /// stata lunga (vedi `raw_input_hook`).
    #[cfg(target_os = "android")]
    premuto_da: Option<f64>,
    #[cfg(target_os = "android")]
    pressione_lunga: bool,
    /// Dita sullo schermo in questo momento (per il pizzico a due dita).
    #[cfg(target_os = "android")]
    dita: std::collections::HashSet<u64>,
    /// Pressione lunga per cui il telefono ha gia' vibrato (istante d'inizio).
    #[cfg(target_os = "android")]
    vibrato_per: Option<f64>,
    /// La tastiera aspetta a comparire: il dito e' stato appoggiato con la
    /// tastiera chiusa e non si sa ancora se sara' un tocco (allora si apre)
    /// o una pressione lunga (allora resta chiusa, per non coprire il menu
    /// Taglia/Copia/Incolla che sta per aprirsi).
    #[cfg(target_os = "android")]
    trattieni_tastiera: bool,
    /// Che gesto sta facendo il dito (vedi `smista_gesti`).
    #[cfg(target_os = "android")]
    gesto: Gesto,
    /// Rilascio di un tocco breve, da dare a egui nel fotogramma dopo la pressione.
    #[cfg(target_os = "android")]
    rilascio_in_sospeso: Vec<egui::Event>,
    /// Il dito e' stato appoggiato mentre era aperto un menu.
    #[cfg(target_os = "android")]
    tocco_chiude_menu: bool,
    /// Il dito, prima di diventare una pressione lunga, era su un controllo.
    #[cfg(target_os = "android")]
    dito_su_controllo: bool,
    /// Dove e' stato appoggiato il dito.
    #[cfg(target_os = "android")]
    premuto_in: Option<egui::Pos2>,
    /// Dopo quanto (secondi dalla pressione data a egui) il dito fermo e' una
    /// pressione lunga: `PRESSIONE_LUNGA`, oppure 0 sul testo della nota, dove
    /// il dito ha gia' aspettato tutto il tempo prima di arrivare a egui.
    #[cfg(target_os = "android")]
    soglia_lunga: f64,
}

/// Tenendo premuto per questo tempo (secondi) compare la descrizione del
/// pulsante, e sollevando il dito il pulsante non viene premuto. Si conta
/// dopo `ATTESA_GESTO`: in tutto circa mezzo secondo.
#[cfg(target_os = "android")]
const PRESSIONE_LUNGA: f64 = 0.35;

/// Per quanto (secondi) un dito appoggiato aspetta prima di diventare un
/// tocco "normale": se nel frattempo si muove in verticale, e' uno scorrimento.
#[cfg(target_os = "android")]
const ATTESA_GESTO: f64 = 0.15;

/// Quanto si aspetta nella fascia in basso sulla nota, da dove si apre la
/// panoramica trascinando verso l'alto.
#[cfg(target_os = "android")]
const ATTESA_FASCIA: f64 = 0.4;

/// Di quanto (punti) deve muoversi il dito per decidere che e' uno scorrimento.
#[cfg(target_os = "android")]
const SOGLIA_GESTO: f32 = 10.0;

/// Dove il dito puo' anche passare di lato tra panoramica, Cestino e
/// Impostazioni, il verso del gesto si decide dopo questo tratto (punti):
/// conta la direzione di tutto il tratto, non la curva iniziale del pollice.
#[cfg(target_os = "android")]
const SOGLIA_LATERALE: f32 = 24.0;

/// Slancio dopo un gesto rapido: di quanto rallenta al secondo (la velocita'
/// si moltiplica per e^-ATTRITO ogni secondo). Con 2 un gesto a 2000 punti/s
/// prosegue per circa 1000 punti e si ferma in circa 2 secondi, come nelle
/// altre app Android.
#[cfg(target_os = "android")]
const ATTRITO_SLANCIO: f32 = 2.0;

/// Sopra questa velocita' (punti/s) un dito appoggiato ferma lo slancio e
/// basta, senza toccare cio' che c'e' sotto.
#[cfg(target_os = "android")]
const SLANCIO_CHE_SI_FERMA: f32 = 150.0;

/// Il gesto del dito, deciso nei primi istanti (vedi `Programma::smista_gesti`).
#[cfg(target_os = "android")]
#[derive(Clone, Copy, Debug)]
enum Gesto {
    Nessuno,
    /// Dito appoggiato: il tocco e' trattenuto finche' non si sa che gesto e'.
    /// `mosso`: non e' un tocco, anche se il dito si solleva subito: si e' gia'
    /// mosso oltre `SOGLIA_GESTO` (il verso si decide a `SOGLIA_LATERALE`),
    /// oppure si e' appoggiato per fermare uno slancio.
    InAttesa { pos: egui::Pos2, modifiers: egui::Modifiers, inizio: f64, mosso: bool },
    /// Scorrimento in verticale: diventa "rotella", il testo non lo vede.
    Scorre { ultimo: egui::Pos2, tempo: f64, velocita: f32 },
    /// Dopo uno scorrimento veloce la pagina continua per un attimo.
    /// `punto`: dove il dito e' stato sollevato. egui da' la rotella all'area
    /// sotto il puntatore, che con il dito alzato non c'e' piu': glielo si ridice.
    Slancio { velocita: f32, tempo: f64, punto: egui::Pos2 },
    /// Dito trascinato di lato sulla nota: la nota lo segue e, sollevandolo,
    /// si passa alla nota accanto. egui non vede niente.
    Sfoglia { origine: egui::Pos2, ultimo: egui::Pos2, tempo: f64, velocita: f32 },
    /// Dito che trascina la panoramica: verso l'alto dalla fascia in basso
    /// sulla nota (`apri`), verso il basso dalla panoramica per chiuderla. La
    /// panoramica segue il dito; egui non vede niente.
    Pannello { apri: bool, origine: egui::Pos2, ultimo: egui::Pos2, tempo: f64, velocita: f32 },
    /// Dito trascinato di lato tra la panoramica e una pagina accanto: le
    /// Impostazioni a destra (verso sinistra le tira fuori), il Cestino a
    /// sinistra (verso destra). `apri`: dalla panoramica alla pagina; se no,
    /// dalla pagina alla panoramica.
    Laterale { pagina: Pagina, apri: bool, origine: egui::Pos2, ultimo: egui::Pos2, tempo: f64, velocita: f32 },
    /// Impostazioni o Cestino tirati verso il basso: scendono seguendo il
    /// dito; lasciati oltre un terzo si torna alla nota.
    ChiudiPagina { origine: egui::Pos2, ultimo: egui::Pos2, tempo: f64, velocita: f32 },
    /// Tocco normale (fermo, pressione lunga o movimento orizzontale).
    Normale,
    /// Due dita (pizzico): nessun tocco arriva a egui.
    Ignora,
}

impl Programma {
    pub fn new(cc: &eframe::CreationContext<'_>, mut app: Applicazione) -> Self {
        // I thread di rete e l'area di notifica possono chiedere un nuovo fotogramma.
        let ctx = cc.egui_ctx.clone();
        // Ctrl +/- e Ctrl + rotella cambiano il testo delle note, non tutta l'interfaccia.
        ctx.options_mut(|o| o.zoom_with_keyboard = false);
        ctx.set_fonts(tema::caratteri());
        #[cfg(target_os = "android")]
        {
            crate::piattaforma::android::imposta_contesto(&ctx);
            // Tenere premuto: oltre questo tempo non e' piu' un tocco, ma
            // apre la descrizione o il menu (come le altre app di Android).
            ctx.options_mut(|o| o.input_options.max_click_duration = PRESSIONE_LUNGA);
            app.interfaccia.tema_sistema = crate::piattaforma::android::tema_scuro_di_sistema();
        }
        let ridisegna: area_notifica::Ridisegna = Arc::new(move || ctx.request_repaint());
        app.rete.ridisegna = Some(ridisegna.clone());
        app.riavvia_rete();
        // Il server per gli assistenti AI e' solo per i computer.
        #[cfg(not(target_os = "android"))]
        app.avvia_mcp();
        let mut programma = Self {
            app,
            tema_applicato: None,
            finestra_mostrata: true,
            fotogrammi: 0,
            chiusura_inviata: false,
            area: None,
            lingua_area: String::new(),
            ridisegna,
            porta_davanti: false,
            #[cfg(not(target_os = "android"))]
            scorciatoia: None,
            #[cfg(not(target_os = "android"))]
            scorciatoia_voluta: None,
            #[cfg(target_os = "android")]
            ponte: Default::default(),
            #[cfg(target_os = "android")]
            basso_prima: 0,
            #[cfg(target_os = "android")]
            premuto_da: None,
            #[cfg(target_os = "android")]
            pressione_lunga: false,
            #[cfg(target_os = "android")]
            dita: Default::default(),
            #[cfg(target_os = "android")]
            vibrato_per: None,
            #[cfg(target_os = "android")]
            trattieni_tastiera: false,
            #[cfg(target_os = "android")]
            dito_su_controllo: false,
            #[cfg(target_os = "android")]
            soglia_lunga: PRESSIONE_LUNGA,
            #[cfg(target_os = "android")]
            tocco_chiude_menu: false,
            #[cfg(target_os = "android")]
            gesto: Gesto::Nessuno,
            #[cfg(target_os = "android")]
            rilascio_in_sospeso: Vec::new(),
            #[cfg(target_os = "android")]
            premuto_in: None,
        };
        programma.crea_area_notifica();
        programma
    }

    fn crea_area_notifica(&mut self) {
        // La vecchia icona va rimossa prima di crearne una nuova.
        self.area = None;
        let tr = &self.app.tr;
        let testi = area_notifica::Testi {
            mostra: tr.t("area_mostra").to_owned(),
            nuova: tr.t("area_nuova").to_owned(),
            esci: tr.t("area_esci").to_owned(),
            suggerimento: crate::NOME_APP.to_owned(),
        };
        self.area = AreaNotifica::crea(testi, self.ridisegna.clone());
        self.lingua_area = tr.codice().to_owned();
    }

    fn gestisci_area_notifica(&mut self) {
        if self.lingua_area != self.app.tr.codice() {
            self.crea_area_notifica();
        }
        let Some(area) = &self.area else { return };
        for comando in area.comandi() {
            match comando {
                Comando::Mostra => {
                    self.app.mostra_finestra();
                    self.porta_davanti = true;
                }
                Comando::NuovaNota if self.app.cartella.is_some() => {
                    self.app.nuova_nota();
                    self.porta_davanti = true;
                }
                Comando::NuovaNota => self.app.mostra_finestra(),
                Comando::Esci => {
                    self.app.prepara_uscita();
                }
            }
        }
    }

    /// Attiva o disattiva la scorciatoia globale come chiedono le preferenze
    /// e crea la nota rapida quando viene premuta.
    #[cfg(not(target_os = "android"))]
    fn gestisci_scorciatoia_globale(&mut self) {
        // Mentre si sceglie una combinazione nuova la vecchia e' sospesa:
        // altrimenti il sistema se la prenderebbe prima che arrivi a noi.
        let attiva = self.app.preferenze.scorciatoia_globale && !self.app.interfaccia.cattura_scorciatoia;
        let voluta = attiva.then(|| self.app.preferenze.combinazione.clone());
        if self.scorciatoia_voluta.as_ref() != Some(&voluta) {
            self.scorciatoia = None;
            if !self.app.interfaccia.cattura_scorciatoia {
                self.app.errore_scorciatoia = None;
            }
            if let Some(combinazione) = &voluta {
                match ScorciatoiaGlobale::crea(self.ridisegna.clone(), combinazione) {
                    Ok(s) => self.scorciatoia = Some(s),
                    Err(errore) => self.app.errore_scorciatoia = Some(errore),
                }
            }
            self.scorciatoia_voluta = Some(voluta);
        }
        if self.scorciatoia.as_ref().is_some_and(ScorciatoiaGlobale::premuta) {
            if self.app.cartella.is_some() {
                self.app.nota_rapida();
            } else {
                self.app.mostra_finestra();
                self.porta_davanti = true;
            }
        }
    }

    /// Android: Indietro, "Nuova nota" dall'icona, file scelti o salvati.
    #[cfg(target_os = "android")]
    fn gestisci_android(&mut self, ctx: &egui::Context) {
        use crate::applicazione::Avviso;
        use crate::piattaforma::android::{self, Evento};
        for evento in android::eventi() {
            match evento {
                Evento::Indietro => self.indietro(ctx),
                Evento::NuovaNota if self.app.cartella.is_some() => {
                    self.app.avviso = None;
                    self.app.nuova_nota();
                    // La nota nuova si vede subito, non la pagina o la panoramica di prima.
                    self.app.interfaccia.pagina = Pagina::Nota;
                    self.app.interfaccia.panoramica = false;
                }
                Evento::NuovaNota => {}
                Evento::FileAperto(percorso) => self.app.avviso = Some(Avviso::ConfermaImportazione(percorso)),
                Evento::FileSalvato(true) => self.app.avviso = Some(Avviso::Info(self.app.tr.t("file_salvato").to_owned())),
                Evento::FileSalvato(false) => {}
                Evento::TemaCambiato => self.app.interfaccia.tema_sistema = android::tema_scuro_di_sistema(),
            }
        }
    }

    /// Indietro chiude la cosa aperta piu' in alto; se non c'e' nulla da
    /// chiudere manda l'app in secondo piano (non la chiude: resta com'era).
    #[cfg(target_os = "android")]
    fn indietro(&mut self, ctx: &egui::Context) {
        if egui::Popup::is_any_open(ctx) {
            // Un menu aperto (Taglia/Copia/Incolla, menu di una scheda, colori).
            egui::Popup::close_all(ctx);
        } else if self.app.avviso.is_some() {
            self.app.avviso = None;
        } else if self.app.interfaccia.rinomina.is_some() {
            self.app.interfaccia.rinomina = None;
        } else if super::schermo::telefono(ctx)
            && self.app.interfaccia.pagina == Pagina::Cestino
            && !self.app.interfaccia.ricerca_cestino.is_empty()
        {
            // Nel Cestino con una ricerca in corso: Indietro prima la toglie.
            self.app.interfaccia.ricerca_cestino.clear();
        } else if super::schermo::telefono(ctx) && self.app.interfaccia.pagina != Pagina::Nota {
            // Da Impostazioni o Cestino si torna alla panoramica, che sta tra
            // le due (mai a una nota del cestino: si girerebbe in tondo).
            super::principale::pagine::torna_alla_panoramica(&mut self.app);
        } else if super::schermo::telefono(ctx) && self.app.interfaccia.panoramica {
            // Dalla panoramica si torna alla nota: e' il posto in cui si lavora.
            self.app.interfaccia.panoramica = false;
        } else if self.parole_evidenziate_nella_nota() {
            // La ricerca evidenzia parole nella nota: Indietro le toglie.
            // (Se nella nota non ce ne sono, Indietro non fa "niente" di visibile:
            // allora si passa oltre.) Per una nota del cestino e' la ricerca
            // del Cestino.
            if self.nota_nel_cestino_aperta() {
                self.app.interfaccia.ricerca_cestino.clear();
            } else {
                self.app.interfaccia.ricerca.clear();
            }
        } else if super::schermo::telefono(ctx) && self.nota_nel_cestino_aperta() {
            // Da una nota del cestino si torna alle schede del Cestino: salgono
            // dal basso, come la panoramica (`principale::telefono`).
            self.app.interfaccia.panoramica = true;
        } else if super::schermo::telefono(ctx)
            && self
                .app
                .interfaccia
                .uscita_chiesta
                .is_none_or(|q| q.elapsed() >= super::principale::DURATA_AVVISO_USCITA)
        {
            // Primo Indietro sulla nota: "Premere di nuovo Indietro per uscire".
            // (Come in molte app: un tocco per sbaglio non fa uscire.)
            self.app.interfaccia.uscita_chiesta = Some(std::time::Instant::now());
            ctx.request_repaint();
        } else {
            // Secondo Indietro: l'app va in secondo piano (non si passa dalla
            // panoramica: provandolo si e' visto che e' scomodo).
            self.app.interfaccia.uscita_chiesta = None;
            self.salva_tutto();
            crate::piattaforma::android::in_secondo_piano();
        }
    }

    /// Nella nota aperta la ricerca in corso evidenzia qualche parola?
    #[cfg(target_os = "android")]
    fn parole_evidenziate_nella_nota(&self) -> bool {
        self.app.interfaccia.selezionata.as_deref().and_then(|id| self.app.nota(id)).is_some_and(|n| {
            let ricerca = self.app.ricerca_per(&n.id);
            !ricerca.is_empty() && !crate::modello::nota::Nota::occorrenze(&n.testo, ricerca).is_empty()
        })
    }

    /// La nota aperta e' nel cestino?
    #[cfg(target_os = "android")]
    fn nota_nel_cestino_aperta(&self) -> bool {
        self.app.interfaccia.selezionata.as_deref().and_then(|id| self.app.nota(id)).is_some_and(|n| n.nel_cestino)
    }

    /// La nota del cestino aperta a pagina intera, se la si puo' tirare giu'
    /// per tornare alle note (niente panoramica, menu o avvisi sopra).
    #[cfg(target_os = "android")]
    fn nota_del_cestino_da_chiudere(&self, ctx: &egui::Context) -> Option<String> {
        let interfaccia = &self.app.interfaccia;
        let libera = super::schermo::telefono(ctx)
            && interfaccia.pagina == Pagina::Nota
            && !interfaccia.panoramica
            && self.app.avviso.is_none()
            && !egui::Popup::is_any_open(ctx)
            && super::schermo::nota_che_scende(ctx).is_none();
        interfaccia
            .selezionata
            .clone()
            .filter(|id| libera && super::principale::pagine::nel_cestino_visibile(&self.app, id))
    }

    /// Android puo' chiudere un'app in secondo piano senza avvisare: appena
    /// si passa a un'altra app si salva tutto.
    #[cfg(target_os = "android")]
    fn salva_tutto(&mut self) {
        if self.app.cartella.is_some() {
            self.app.conferma_rinomina();
            // Un'eliminazione con ANNULLA ancora in corso diventa definitiva:
            // Android potrebbe chiudere l'app prima della fine.
            self.app.conferma_eliminazione();
            self.app.salva_ora();
            self.app.salva_sessione();
            self.app.salva_preferenze();
        }
    }

    /// Due aggiustamenti al dito, prima che egui veda gli eventi:
    ///
    /// - **descrizioni tenendo premuto**: egui mostra la descrizione di un
    ///   pulsante mentre il dito resta fermo, ma la toglie appena il dito si
    ///   alza (il puntatore "sparisce"). Dopo una pressione lunga si lascia il
    ///   puntatore dov'era, cosi' la descrizione resta da leggere finche' non
    ///   si tocca altrove. Per esempio: tenendo premuto l'orologio compare
    ///   "Mostra le versioni precedenti della nota…" e resta;
    /// - **pizzico a due dita**: egui segue il primo dito come un mouse, quindi
    ///   allargando le dita sul testo selezionerebbe parole. Con due dita sullo
    ///   schermo il puntatore resta fermo: le dita cambiano solo la dimensione
    ///   del testo (vedi `gestisci_dimensione_testo`).
    #[cfg(target_os = "android")]
    fn tocchi(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        let adesso = raw_input.time.unwrap_or(0.0);
        self.smista_gesti(ctx, raw_input, adesso);
        for evento in &raw_input.events {
            match evento {
                egui::Event::PointerButton { pressed: true, pos, .. } => {
                    self.premuto_da = Some(adesso);
                    self.premuto_in = Some(*pos);
                    self.pressione_lunga = false;
                    // Con la tastiera chiusa si aspetta di sapere che tocco e'.
                    self.trattieni_tastiera = !crate::piattaforma::android::tastiera_aperta();
                    // Un tocco che chiude un menu aperto (per esempio Taglia/Copia/
                    // Incolla) serve solo a chiuderlo: la tastiera resta chiusa.
                    self.tocco_chiude_menu = egui::Popup::is_any_open(ctx);
                }
                egui::Event::PointerButton { pressed: false, pos, .. } => {
                    // Un tocco breve e fermo apre la tastiera (se tocca un campo);
                    // una pressione lunga o uno scorrimento la lasciano chiusa.
                    // (Un trascinamento lento, come quello del menu dal bordo, non
                    // e' una pressione lunga: il dito si e' mosso.)
                    let fermo = self.premuto_in.take().is_some_and(|da| da.distance(*pos) < 12.0);
                    self.pressione_lunga = fermo && self.premuto_da.take().is_some_and(|da| adesso - da >= self.soglia_lunga);
                    self.premuto_da = None;
                    if !self.pressione_lunga && fermo && !self.tocco_chiude_menu {
                        self.trattieni_tastiera = false;
                    }
                }
                _ => {}
            }
        }
        let lunga = self.pressione_lunga;
        let pizzico = self.dita.len() >= 2;
        raw_input.events.retain(|evento| match evento {
            egui::Event::PointerGone => !lunga,
            egui::Event::PointerMoved(_) => !pizzico,
            _ => true,
        });
    }

    /// Decide che gesto fa il dito prima che egui lo veda. egui tratta il dito
    /// come un mouse: appoggiarlo sul testo sposta subito il cursore e
    /// trascinarlo seleziona. Sul telefono invece trascinare in verticale deve
    /// far scorrere la nota. Per questo il tocco resta trattenuto per un
    /// attimo (`ATTESA_GESTO`):
    ///
    /// - se il dito si muove in verticale e' uno **scorrimento**: arriva a egui
    ///   come rotella, il campo di testo non lo vede e il cursore resta dov'era;
    ///   sollevando il dito dopo un movimento veloce la pagina continua un poco;
    /// - se il dito si alza subito, e' un **tocco**;
    /// - se resta fermo, o si muove in orizzontale (schede, menu dal bordo), il
    ///   tocco passa a egui come sempre (pressione lunga, menu, trascinamenti).
    #[cfg(target_os = "android")]
    fn smista_gesti(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput, adesso: f64) {
        use egui::{Event, PointerButton};
        let premi = |pos: egui::Pos2, modifiers: egui::Modifiers, pressed: bool| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers,
        };
        let rotella = |dy: f32, source: egui::MouseWheelSource| Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, dy),
            phase: egui::TouchPhase::Move,
            source,
            modifiers: egui::Modifiers::NONE,
        };
        // Inizio e fine di uno scorrimento con il dito. Durante lo scorrimento
        // egui da' subito ogni spostamento alla nota; senza, gli spostamenti
        // grandi (dito veloce) li spalma su un decimo di secondo, come per la
        // rotella di un mouse, e lo scorrimento sembra lento e incostante.
        let fase_rotella = |phase: egui::TouchPhase| Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::Vec2::ZERO,
            phase,
            source: egui::MouseWheelSource::Trackpad,
            modifiers: egui::Modifiers::NONE,
        };
        let mut nuovi = Vec::with_capacity(raw_input.events.len() + 3);
        // Il rilascio del tocco breve del fotogramma prima (vedi sotto).
        nuovi.append(&mut self.rilascio_in_sospeso);
        for evento in std::mem::take(&mut raw_input.events) {
            if let Event::Touch { id, phase, .. } = &evento {
                match phase {
                    egui::TouchPhase::Start => {
                        // Un dito che si appoggia senza nessun gesto in corso apre
                        // un gesto nuovo: dita ricordate da prima (un "dito alzato"
                        // andato perso) farebbero credere a un pizzico infinito e
                        // ogni tocco verrebbe scartato.
                        if matches!(self.gesto, Gesto::Nessuno | Gesto::Slancio { .. }) {
                            self.dita.clear();
                        }
                        self.dita.insert(id.0);
                    }
                    egui::TouchPhase::End | egui::TouchPhase::Cancel => {
                        self.dita.remove(&id.0);
                        // Tutte le dita alzate: il pizzico e' finito.
                        if self.dita.is_empty() && matches!(self.gesto, Gesto::Ignora) {
                            self.gesto = Gesto::Nessuno;
                        }
                    }
                    egui::TouchPhase::Move => {}
                }
                // Secondo dito: e' un pizzico, nessun tocco per egui.
                if self.dita.len() >= 2 && !matches!(self.gesto, Gesto::Normale) {
                    self.gesto = Gesto::Ignora;
                }
                nuovi.push(evento);
                continue;
            }
            match (self.gesto, &evento) {
                (_, Event::PointerButton { pressed: true, button: PointerButton::Primary, pos, modifiers }) => {
                    self.imposta_soglia_lunga(ctx, PRESSIONE_LUNGA);
                    nuovi.push(Event::PointerMoved(*pos));
                    if super::schermo::maniglie_selezione(ctx).iter().any(|z| z.contains(*pos)) {
                        // Una maniglia della selezione: il dito la trascina subito.
                        self.gesto = Gesto::Normale;
                        nuovi.push(premi(*pos, *modifiers, true));
                    } else {
                        // La pressione si trattiene: arrivera' piu' tardi, se serve.
                        // Se la nota stava ancora scorrendo da sola, il dito la
                        // ferma e basta (come nelle altre app): sollevato subito,
                        // non e' un tocco su quello che c'e' sotto.
                        let ferma_slancio = matches!(self.gesto, Gesto::Slancio { velocita, .. } if velocita.abs() > SLANCIO_CHE_SI_FERMA);
                        self.gesto = Gesto::InAttesa { pos: *pos, modifiers: *modifiers, inizio: adesso, mosso: ferma_slancio };
                    }
                }
                (Gesto::InAttesa { pos, modifiers, inizio, .. }, Event::PointerMoved(p)) => {
                    let d = *p - pos;
                    if d.length() < SOGLIA_GESTO {
                        continue;
                    }
                    // Dove si passa di lato (panoramica, Cestino, Impostazioni)
                    // il verso si decide piu' avanti: il pollice comincia spesso
                    // con una curva (sul lato destro, salendo, piega a sinistra) e
                    // i primi punti direbbero "di lato" a uno scorrimento, o "in
                    // giu'" a un passaggio di lato. Intanto il dito e' gia' un
                    // gesto: niente tocco, anche se ci mette un po'.
                    let di_lato = self.verso_laterale(ctx, 1.0).is_some() || self.verso_laterale(ctx, -1.0).is_some();
                    if di_lato && d.length() < SOGLIA_LATERALE {
                        self.gesto = Gesto::InAttesa { pos, modifiers, inizio, mosso: true };
                        continue;
                    }
                    let verticale = d.x.abs() < d.y.abs();
                    let in_zona = |zona: Option<egui::Rect>| zona.is_some_and(|z| z.contains(pos));
                    let apre = in_zona(super::schermo::zona_maniglia(ctx)) || in_zona(super::schermo::zona_apri_panoramica(ctx));
                    if d.y < 0.0 && verticale && !self.app.interfaccia.panoramica && apre {
                        // Dalla fascia in basso (o dalla meta' bassa, se il testo
                        // non scorre piu' giu') verso l'alto: la panoramica sale e
                        // segue il dito.
                        self.app.conferma_rinomina();
                        self.gesto = Gesto::Pannello { apri: true, origine: pos, ultimo: *p, tempo: adesso, velocita: 0.0 };
                        self.segui_pannello(ctx, true, p.y - pos.y);
                    } else if d.y > 0.0
                        && verticale
                        && self.app.interfaccia.pagina != Pagina::Nota
                        && in_zona(super::schermo::zona_chiudi_pagina(ctx))
                    {
                        // Impostazioni o Cestino tirati verso il basso: scendono seguendo il dito.
                        self.gesto = Gesto::ChiudiPagina { origine: pos, ultimo: *p, tempo: adesso, velocita: 0.0 };
                        self.segui_chiudi_pagina(ctx, p.y - pos.y);
                    } else if let Some(id) = self.nota_del_cestino_da_chiudere(ctx).filter(|_| {
                        d.y > 0.0 && verticale && in_zona(super::schermo::zona_chiudi_pagina(ctx))
                    }) {
                        // Una nota del cestino tirata giu' dalla testata: scende
                        // seguendo il dito e sotto c'e' gia' la nota a cui si torna.
                        super::principale::pagine::esci_dal_cestino(&mut self.app);
                        super::schermo::imposta_nota_che_scende(ctx, Some(super::schermo::NotaCheScende { id, esce: false }));
                        self.gesto = Gesto::ChiudiPagina { origine: pos, ultimo: *p, tempo: adesso, velocita: 0.0 };
                        self.segui_chiudi_pagina(ctx, p.y - pos.y);
                    } else if d.y > 0.0
                        && verticale
                        && self.app.interfaccia.panoramica
                        && in_zona(super::schermo::zona_chiudi_panoramica(ctx))
                    {
                        // La panoramica tirata verso il basso: scende seguendo il dito.
                        self.gesto = Gesto::Pannello { apri: false, origine: pos, ultimo: *p, tempo: adesso, velocita: 0.0 };
                        self.segui_pannello(ctx, false, p.y - pos.y);
                    } else if d.y > 0.0 && verticale && in_zona(super::schermo::zona_barra_tastiera(ctx)) {
                        // La barra sopra la tastiera tirata verso il basso: la tastiera si chiude.
                        ctx.memory_mut(|m| {
                            if let Some(campo) = m.focused() {
                                m.surrender_focus(campo);
                            }
                        });
                        self.gesto = Gesto::Ignora;
                        ctx.request_repaint();
                    } else if verticale {
                        self.gesto = Gesto::Scorre { ultimo: *p, tempo: adesso, velocita: 0.0 };
                        nuovi.push(Event::PointerMoved(*p));
                        nuovi.push(fase_rotella(egui::TouchPhase::Start));
                        nuovi.push(rotella(d.y, egui::MouseWheelSource::Trackpad));
                    } else if let Some((pagina, apri)) = self.verso_laterale(ctx, d.x) {
                        self.gesto = Gesto::Laterale { pagina, apri, origine: pos, ultimo: *p, tempo: adesso, velocita: 0.0 };
                        self.segui_laterale(ctx, pagina, apri, d.x);
                    } else if self.si_sfoglia(ctx, pos) {
                        self.gesto = Gesto::Sfoglia { origine: pos, ultimo: *p, tempo: adesso, velocita: 0.0 };
                        self.segui_sfoglia(ctx, d.x);
                    } else if in_zona(super::schermo::zona_testo(ctx)) {
                        // Sul testo trascinare non seleziona (si seleziona tenendo
                        // premuto, poi con le maniglie): anche di lato fa scorrere.
                        self.gesto = Gesto::Scorre { ultimo: *p, tempo: adesso, velocita: 0.0 };
                        nuovi.push(Event::PointerMoved(*p));
                        nuovi.push(fase_rotella(egui::TouchPhase::Start));
                        nuovi.push(rotella(d.y, egui::MouseWheelSource::Trackpad));
                    } else {
                        self.gesto = Gesto::Normale;
                        nuovi.push(Event::PointerMoved(pos));
                        nuovi.push(premi(pos, modifiers, true));
                        nuovi.push(evento);
                    }
                    // Panoramica, pagine e note che seguono il dito: per egui il
                    // dito non c'e' piu'. Altrimenti quello che scorre sotto il
                    // dito fermo (il "+", la barra di scorrimento) si illumina come
                    // al passaggio del mouse.
                    if matches!(
                        self.gesto,
                        Gesto::Pannello { .. } | Gesto::ChiudiPagina { .. } | Gesto::Laterale { .. } | Gesto::Sfoglia { .. }
                    ) {
                        nuovi.push(Event::PointerGone);
                    }
                }
                (Gesto::Scorre { ultimo, tempo, velocita }, Event::PointerMoved(p)) => {
                    let dy = p.y - ultimo.y;
                    let dt = ((adesso - tempo) as f32).max(1.0 / 120.0);
                    // Velocita' media degli ultimi movimenti, per lo slancio.
                    let velocita = 0.6 * (dy / dt) + 0.4 * velocita;
                    self.gesto = Gesto::Scorre { ultimo: *p, tempo: adesso, velocita };
                    nuovi.push(Event::PointerMoved(*p));
                    nuovi.push(rotella(dy, egui::MouseWheelSource::Trackpad));
                }
                (Gesto::Sfoglia { origine, ultimo, tempo, velocita }, Event::PointerMoved(p)) => {
                    let dt = ((adesso - tempo) as f32).max(1.0 / 120.0);
                    let velocita = 0.6 * ((p.x - ultimo.x) / dt) + 0.4 * velocita;
                    self.gesto = Gesto::Sfoglia { origine, ultimo: *p, tempo: adesso, velocita };
                    self.segui_sfoglia(ctx, p.x - origine.x);
                }
                (Gesto::Sfoglia { origine, ultimo, velocita, .. }, Event::PointerButton { pressed: false, .. }) => {
                    self.gesto = Gesto::Nessuno;
                    self.fine_sfoglia(ctx, ultimo.x - origine.x, velocita);
                }
                (Gesto::Sfoglia { .. }, Event::PointerButton { .. }) => {}
                (Gesto::Pannello { apri, origine, ultimo, tempo, velocita }, Event::PointerMoved(p)) => {
                    let dt = ((adesso - tempo) as f32).max(1.0 / 120.0);
                    let velocita = 0.6 * ((p.y - ultimo.y) / dt) + 0.4 * velocita;
                    self.gesto = Gesto::Pannello { apri, origine, ultimo: *p, tempo: adesso, velocita };
                    self.segui_pannello(ctx, apri, p.y - origine.y);
                }
                (Gesto::Pannello { apri, origine, ultimo, velocita, .. }, Event::PointerButton { pressed: false, .. }) => {
                    self.gesto = Gesto::Nessuno;
                    self.fine_pannello(ctx, apri, ultimo.y - origine.y, velocita);
                }
                (Gesto::Pannello { .. }, Event::PointerButton { .. }) => {}
                (Gesto::Laterale { pagina, apri, origine, ultimo, tempo, velocita }, Event::PointerMoved(p)) => {
                    let dt = ((adesso - tempo) as f32).max(1.0 / 120.0);
                    let velocita = 0.6 * ((p.x - ultimo.x) / dt) + 0.4 * velocita;
                    self.gesto = Gesto::Laterale { pagina, apri, origine, ultimo: *p, tempo: adesso, velocita };
                    self.segui_laterale(ctx, pagina, apri, p.x - origine.x);
                }
                (Gesto::Laterale { pagina, apri, origine, ultimo, velocita, .. }, Event::PointerButton { pressed: false, .. }) => {
                    self.gesto = Gesto::Nessuno;
                    self.fine_laterale(ctx, pagina, apri, ultimo.x - origine.x, velocita);
                }
                (Gesto::Laterale { .. }, Event::PointerButton { .. }) => {}
                (Gesto::ChiudiPagina { origine, ultimo, tempo, velocita }, Event::PointerMoved(p)) => {
                    let dt = ((adesso - tempo) as f32).max(1.0 / 120.0);
                    let velocita = 0.6 * ((p.y - ultimo.y) / dt) + 0.4 * velocita;
                    self.gesto = Gesto::ChiudiPagina { origine, ultimo: *p, tempo: adesso, velocita };
                    self.segui_chiudi_pagina(ctx, p.y - origine.y);
                }
                (Gesto::ChiudiPagina { origine, ultimo, velocita, .. }, Event::PointerButton { pressed: false, .. }) => {
                    self.gesto = Gesto::Nessuno;
                    self.fine_chiudi_pagina(ctx, ultimo.y - origine.y, velocita);
                }
                (Gesto::ChiudiPagina { .. }, Event::PointerButton { .. }) => {}
                (Gesto::InAttesa { mosso: true, .. }, Event::PointerButton { pressed: false, .. }) => {
                    // Un gesto cortissimo, sollevato prima di sapere il verso:
                    // non e' un tocco, non fa niente.
                    self.gesto = Gesto::Nessuno;
                }
                (Gesto::InAttesa { pos, modifiers, .. }, Event::PointerButton { pressed: false, .. }) => {
                    // Tocco breve: la pressione ora, il rilascio al fotogramma
                    // dopo. Se arrivano insieme egui mette il cursore in fondo
                    // al testo invece che sotto il dito.
                    self.gesto = Gesto::Nessuno;
                    nuovi.push(Event::PointerMoved(pos));
                    nuovi.push(premi(pos, modifiers, true));
                    self.rilascio_in_sospeso.push(premi(pos, modifiers, false));
                    ctx.request_repaint();
                }
                (Gesto::Scorre { velocita, tempo, ultimo }, Event::PointerButton { pressed: false, .. }) => {
                    // Lo scorrimento finisce senza tocchi: se era veloce continua un poco.
                    let fermo_da = adesso - tempo;
                    self.gesto = if velocita.abs() > 300.0 && fermo_da < 0.1 {
                        Gesto::Slancio { velocita, tempo: adesso, punto: ultimo }
                    } else {
                        // Fine dello scorrimento, al fotogramma dopo: insieme
                        // all'ultimo spostamento lo cancellerebbe.
                        self.rilascio_in_sospeso.push(fase_rotella(egui::TouchPhase::End));
                        ctx.request_repaint();
                        Gesto::Nessuno
                    };
                }
                (Gesto::Ignora, Event::PointerButton { pressed: false, .. }) => {
                    self.gesto = Gesto::Nessuno;
                }
                (Gesto::Ignora, Event::PointerButton { .. } | Event::PointerMoved(_)) => {}
                // "Puntatore sparito" dopo un rilascio rimandato: va dopo il rilascio,
                // altrimenti egui annulla il tocco.
                (_, Event::PointerGone) if !self.rilascio_in_sospeso.is_empty() => {
                    self.rilascio_in_sospeso.push(evento);
                }
                (Gesto::Normale, Event::PointerButton { pressed: false, .. }) => {
                    self.gesto = Gesto::Nessuno;
                    nuovi.push(evento);
                }
                _ => nuovi.push(evento),
            }
        }
        // Nella fascia in basso sulla nota il pollice spesso resta fermo un
        // attimo prima di salire: li' si aspetta di piu' a decidere, cosi' anche
        // partendo piano il dito apre la panoramica (e non e' preso per un tocco).
        // Sul testo della nota si aspetta tutta la pressione lunga: un dito che
        // esita e poi si muove fa scorrere la nota, non seleziona. Rimasto fermo,
        // arriva a egui gia' come pressione lunga (parola selezionata e menu).
        let sul_testo = matches!(self.gesto, Gesto::InAttesa { pos, .. } if super::schermo::zona_testo(ctx).is_some_and(|z| z.contains(pos)));
        let attesa = match self.gesto {
            Gesto::InAttesa { .. } if sul_testo => ATTESA_GESTO + PRESSIONE_LUNGA,
            Gesto::InAttesa { pos, .. }
                if !self.app.interfaccia.panoramica
                    && [super::schermo::zona_maniglia(ctx), super::schermo::zona_apri_panoramica(ctx)]
                        .iter()
                        .any(|z| z.is_some_and(|z| z.contains(pos))) =>
            {
                ATTESA_FASCIA
            }
            // Anche sulla nota del cestino, che si tira giu' per tornare alle note.
            Gesto::InAttesa { pos, .. }
                if self.nota_del_cestino_da_chiudere(ctx).is_some()
                    && super::schermo::zona_chiudi_pagina(ctx).is_some_and(|z| z.contains(pos)) =>
            {
                ATTESA_FASCIA
            }
            _ => ATTESA_GESTO,
        };
        match self.gesto {
            Gesto::InAttesa { pos, modifiers, inizio, mosso: false } if adesso - inizio >= attesa => {
                // Il dito resta fermo: e' un tocco normale (forse una pressione lunga).
                self.gesto = Gesto::Normale;
                nuovi.push(Event::PointerMoved(pos));
                nuovi.push(premi(pos, modifiers, true));
                if sul_testo {
                    self.imposta_soglia_lunga(ctx, 0.0);
                    self.dito_su_controllo = true;
                    ctx.request_repaint();
                }
            }
            Gesto::InAttesa { .. } => ctx.request_repaint_after(std::time::Duration::from_millis(20)),
            Gesto::Slancio { velocita, tempo, punto } => {
                let dt = ((adesso - tempo) as f32).clamp(0.0, 0.05);
                // Il puntatore dove era il dito: la rotella va all'area che scorreva.
                nuovi.push(Event::PointerMoved(punto));
                nuovi.push(rotella(velocita * dt, egui::MouseWheelSource::Momentum));
                let velocita = velocita * (-dt * ATTRITO_SLANCIO).exp();
                self.gesto = if velocita.abs() < 40.0 {
                    self.rilascio_in_sospeso.push(fase_rotella(egui::TouchPhase::End));
                    self.rilascio_in_sospeso.push(Event::PointerGone);
                    Gesto::Nessuno
                } else {
                    Gesto::Slancio { velocita, tempo: adesso, punto }
                };
                ctx.request_repaint();
            }
            _ => {}
        }
        raw_input.events = nuovi;
    }

    /// Dopo quanto il dito fermo diventa una pressione lunga, per egui e per
    /// la vibrazione (vedi `soglia_lunga`).
    #[cfg(target_os = "android")]
    fn imposta_soglia_lunga(&mut self, ctx: &egui::Context, soglia: f64) {
        self.soglia_lunga = soglia;
        ctx.options_mut(|o| o.input_options.max_click_duration = soglia);
    }

    /// Si puo' sfogliare partendo da qui? Solo sulla nota aperta (non su menu,
    /// panoramica o pagine), con la tastiera chiusa e senza menu aperti.
    #[cfg(target_os = "android")]
    fn si_sfoglia(&self, ctx: &egui::Context, pos: egui::Pos2) -> bool {
        let interfaccia = &self.app.interfaccia;
        super::schermo::telefono(ctx)
            && !crate::piattaforma::android::tastiera_aperta()
            && interfaccia.pagina == crate::applicazione::Pagina::Nota
            && !interfaccia.panoramica
            && interfaccia.rinomina.is_none()
            && interfaccia.selezionata.is_some()
            && self.app.avviso.is_none()
            && !egui::Popup::is_any_open(ctx)
            && super::schermo::zona_sfoglia_attuale(ctx).is_some_and(|z| z.contains(pos))
    }

    /// La nota segue il dito; dal lato in cui non ci sono altre note si muove
    /// appena, per far capire che li' si e' arrivati in fondo.
    #[cfg(target_os = "android")]
    fn segui_sfoglia(&self, ctx: &egui::Context, dx: f32) {
        let verso = if dx < 0.0 { 1 } else { -1 };
        let c_e = self.app.interfaccia.selezionata.as_deref().is_some_and(|id| {
            self.app.nota_vicina(id, verso).is_some() || (verso == 1 && self.app.pagina_bianca_dopo(id))
        });
        super::schermo::imposta_sfoglia(ctx, Some(if c_e { dx } else { dx * 0.25 }));
        ctx.request_repaint();
    }

    /// Dito sollevato: oltre un terzo della larghezza, o con un gesto veloce,
    /// si passa alla nota accanto; altrimenti la nota torna al suo posto. In
    /// tutti e due i casi lo scorrimento si completa con un'animazione; il
    /// cambio di nota lo fa l'editor quando la nota accanto e' arrivata
    /// (`editor::mostra`, con `schermo::conclusione_sfoglia`).
    #[cfg(target_os = "android")]
    fn fine_sfoglia(&mut self, ctx: &egui::Context, dx: f32, velocita: f32) {
        super::schermo::imposta_sfoglia(ctx, None);
        ctx.request_repaint();
        let larghezza = super::schermo::zona_sfoglia_attuale(ctx).map_or(360.0, |z| z.width());
        let veloce = velocita.abs() > 700.0 && velocita.signum() == dx.signum() && dx.abs() > 30.0;
        let verso = if dx < 0.0 { 1 } else { -1 };
        let c_e = self.app.interfaccia.selezionata.as_deref().is_some_and(|id| {
            self.app.nota_vicina(id, verso).is_some() || (verso == 1 && self.app.pagina_bianca_dopo(id))
        });
        let conclusione = if c_e && (dx.abs() >= larghezza / 3.0 || veloce) {
            (-(verso as f32) * larghezza, verso as i8)
        } else {
            (0.0, 0)
        };
        super::schermo::imposta_conclusione_sfoglia(ctx, Some(conclusione));
    }

    /// Un dito che va di lato porta tra la panoramica e una pagina accanto?
    /// Dalla panoramica verso sinistra le Impostazioni, verso destra il
    /// Cestino; dalla pagina, nel verso opposto, si torna alla panoramica.
    /// Restituisce la pagina e se la si apre.
    #[cfg(target_os = "android")]
    fn verso_laterale(&self, ctx: &egui::Context, dx: f32) -> Option<(Pagina, bool)> {
        let interfaccia = &self.app.interfaccia;
        let libero = super::schermo::telefono(ctx)
            && interfaccia.panoramica
            && !crate::piattaforma::android::tastiera_aperta()
            && interfaccia.rinomina.is_none()
            && self.app.avviso.is_none()
            && !egui::Popup::is_any_open(ctx);
        if !libero {
            return None;
        }
        match interfaccia.pagina {
            Pagina::Nota if dx < 0.0 => Some((Pagina::Impostazioni, true)),
            Pagina::Nota => Some((Pagina::Cestino, true)),
            Pagina::Impostazioni if dx > 0.0 => Some((Pagina::Impostazioni, false)),
            Pagina::Cestino if dx < 0.0 => Some((Pagina::Cestino, false)),
            _ => None,
        }
    }

    /// Da che parte si muove il dito per aprire la pagina: verso sinistra (-1)
    /// le Impostazioni, che stanno a destra; verso destra (1) il Cestino.
    #[cfg(target_os = "android")]
    fn verso_apertura(pagina: Pagina) -> f32 {
        if pagina == Pagina::Impostazioni { -1.0 } else { 1.0 }
    }

    /// La pagina segue il dito, entrando (o uscendo) dal suo bordo.
    #[cfg(target_os = "android")]
    fn segui_laterale(&self, ctx: &egui::Context, pagina: Pagina, apri: bool, dx: f32) {
        let largo = super::schermo::area_libera(ctx).width().max(1.0);
        let s = Self::verso_apertura(pagina);
        let visibile = if apri { s * dx / largo } else { 1.0 + s * dx / largo };
        super::schermo::imposta_scorri_laterale(ctx, Some((pagina, visibile.clamp(0.0, 1.0))));
        ctx.request_repaint();
    }

    /// Dito sollevato: oltre un terzo della larghezza, o con un gesto veloce,
    /// si passa all'altra pagina; altrimenti si torna dov'era. Il resto del
    /// movimento lo fa l'animazione in `principale::telefono`.
    #[cfg(target_os = "android")]
    fn fine_laterale(&mut self, ctx: &egui::Context, pagina: Pagina, apri: bool, dx: f32, velocita: f32) {
        super::schermo::imposta_scorri_laterale(ctx, None);
        ctx.request_repaint();
        let largo = super::schermo::area_libera(ctx).width().max(1.0);
        let s = if apri { Self::verso_apertura(pagina) } else { -Self::verso_apertura(pagina) };
        let percorso = s * dx;
        let veloce = s * velocita > 700.0 && percorso > 30.0;
        if percorso < largo / 3.0 && !veloce {
            return;
        }
        match (pagina, apri) {
            (Pagina::Cestino, true) => super::principale::pagine::apri_cestino(&mut self.app),
            (_, true) => {
                self.app.conferma_rinomina();
                self.app.interfaccia.pagina = pagina;
            }
            (_, false) => super::principale::pagine::torna_alla_panoramica(&mut self.app),
        }
    }

    /// La pagina scende seguendo il dito (`dy` dal punto di partenza).
    #[cfg(target_os = "android")]
    fn segui_chiudi_pagina(&self, ctx: &egui::Context, dy: f32) {
        let alto = super::schermo::area_libera(ctx).height().max(1.0);
        super::schermo::imposta_chiudi_pagina(ctx, Some((dy / alto).clamp(0.0, 1.0)));
        ctx.request_repaint();
    }

    /// Dito sollevato: oltre un terzo dell'altezza, o con un gesto veloce, si
    /// torna alla nota e la pagina finisce di scendere da sola; altrimenti
    /// risale.
    #[cfg(target_os = "android")]
    fn fine_chiudi_pagina(&mut self, ctx: &egui::Context, dy: f32, velocita: f32) {
        super::schermo::imposta_chiudi_pagina(ctx, None);
        ctx.request_repaint();
        let alto = super::schermo::area_libera(ctx).height().max(1.0);
        let veloce = velocita > 700.0 && dy > 30.0;
        if let Some(mut nota) = super::schermo::nota_che_scende(ctx) {
            // La nota del cestino: finisce di scendere o risale
            // (`principale::telefono`, che poi la toglie o la riapre).
            nota.esce = dy >= alto / 3.0 || veloce;
            super::schermo::imposta_nota_che_scende(ctx, Some(nota));
            return;
        }
        if dy >= alto / 3.0 || veloce {
            let quale = self.app.interfaccia.pagina;
            super::principale::pagine::torna_alla_nota(&mut self.app);
            super::schermo::imposta_pagina_uscente(ctx, Some(quale));
        }
    }

    /// La panoramica segue il dito: `dy` e' lo spostamento dal punto di
    /// partenza (negativo verso l'alto).
    #[cfg(target_os = "android")]
    fn segui_pannello(&self, ctx: &egui::Context, apri: bool, dy: f32) {
        let alto = super::schermo::area_libera(ctx).height().max(1.0);
        let progresso = if apri { (-dy / alto).clamp(0.0, 1.0) } else { (1.0 - dy / alto).clamp(0.0, 1.0) };
        super::schermo::imposta_trascina_panoramica(ctx, Some(progresso));
        ctx.request_repaint();
    }

    /// Dito sollevato: oltre un terzo dello schermo, o con un gesto veloce, la
    /// panoramica si apre (o si chiude); altrimenti torna dov'era. Il resto
    /// del movimento lo fa l'animazione in `principale::telefono`.
    #[cfg(target_os = "android")]
    fn fine_pannello(&mut self, ctx: &egui::Context, apri: bool, dy: f32, velocita: f32) {
        super::schermo::imposta_trascina_panoramica(ctx, None);
        ctx.request_repaint();
        let alto = super::schermo::area_libera(ctx).height().max(1.0);
        let percorso = if apri { -dy } else { dy };
        let veloce = if apri { velocita < -700.0 } else { velocita > 700.0 } && percorso > 30.0;
        if percorso >= alto / 3.0 || veloce {
            self.app.interfaccia.panoramica = apri;
        }
    }

    /// Una leggera vibrazione quando il dito, fermo su un controllo, diventa
    /// una pressione lunga: e' il momento in cui compare la descrizione o il
    /// menu, e si puo' sollevare il dito.
    #[cfg(target_os = "android")]
    fn vibra_se_pressione_lunga(&mut self, ctx: &egui::Context) {
        let (giu, inizio, origine, posizione, adesso) = ctx.input(|i| {
            (i.pointer.primary_down(), i.pointer.press_start_time(), i.pointer.press_origin(), i.pointer.latest_pos(), i.time)
        });
        let (Some(inizio), true) = (inizio, giu) else {
            self.dito_su_controllo = false;
            return;
        };
        let fermo = matches!((origine, posizione), (Some(o), Some(p)) if o.distance(p) < 8.0);
        // egui smette di considerare il dito "in uso" proprio quando diventa
        // una pressione lunga: conta se lo era un attimo prima.
        if adesso - inizio < self.soglia_lunga {
            self.dito_su_controllo |= ctx.egui_is_using_pointer();
            // Senza nuovi tocchi l'app non ridisegnerebbe fino al rilascio.
            ctx.request_repaint_after(Duration::from_millis(40));
            return;
        }
        if fermo && self.dito_su_controllo && self.vibrato_per != Some(inizio) {
            self.vibrato_per = Some(inizio);
            crate::piattaforma::android::vibra();
        }
    }

    /// Lascia libero lo spazio di barra di stato e tastiera: restituisce
    /// l'area in cui disegnare.
    #[cfg(target_os = "android")]
    fn area_libera(&mut self, ctx: &egui::Context) -> egui::Rect {
        let coperto = crate::piattaforma::android::spazio_coperto();
        if coperto.basso > self.basso_prima + 50 {
            // La tastiera si e' aperta: la riga che si sta scrivendo resta in vista.
            super::tastiera::mostra_cursore(ctx);
        }
        self.basso_prima = coperto.basso;
        let ppp = ctx.pixels_per_point();
        let mut area = ctx.content_rect();
        area.min.y += coperto.alto as f32 / ppp;
        area.max.y -= coperto.basso as f32 / ppp;
        // In orizzontale la barra di navigazione puo' stare su un lato.
        area.min.x += coperto.sinistra as f32 / ppp;
        area.max.x -= coperto.destra as f32 / ppp;
        // Mai un'area rovesciata (per esempio durante la rotazione, quando gli
        // spazi coperti non sono ancora quelli del nuovo verso).
        area.max.y = area.max.y.max(area.min.y + 1.0);
        area.max.x = area.max.x.max(area.min.x + 1.0);
        super::schermo::imposta_area_libera(ctx, area);
        area
    }

    /// Invia la chiusura definitiva quando l'uscita e' confermata.
    fn invia_chiusura(&mut self, ctx: &egui::Context) {
        if self.app.uscita_confermata && !self.chiusura_inviata {
            self.chiusura_inviata = true;
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close);
        }
    }

    fn scorciatoie(&mut self, ctx: &egui::Context) {
        // Mentre si registra una combinazione nuova, i tasti servono solo a quella.
        if self.app.interfaccia.cattura_scorciatoia {
            return;
        }
        let (nuova, cerca, salva, rinomina, widget) = ctx.input_mut(|i| {
            (
                consuma_comando(i, Key::N),
                consuma_comando(i, Key::F),
                consuma_comando(i, Key::S),
                i.consume_key(Modifiers::NONE, Key::F2),
                consuma_comando(i, Key::D),
            )
        });
        if self.app.cartella.is_none() {
            return;
        }
        // Ctrl+Shift+frecce sposta la scheda selezionata, ma solo fuori dai
        // campi di testo: li' le stesse frecce selezionano parola per parola.
        if !ctx.egui_wants_keyboard_input() {
            let passo = ctx.input_mut(|i| {
                let mut passo = 0;
                i.events.retain(|e| match e {
                    egui::Event::Key { key, pressed: true, modifiers, .. }
                        if modifiers.command && modifiers.shift && !modifiers.alt
                            && matches!(key, Key::ArrowLeft | Key::ArrowRight) =>
                    {
                        passo = if *key == Key::ArrowLeft { -1 } else { 1 };
                        false
                    }
                    _ => true,
                });
                passo
            });
            if passo != 0 && self.app.interfaccia.ricerca.trim().is_empty() {
                if let Some(id) = self.app.interfaccia.selezionata.clone() {
                    self.app.sposta_scheda_di(&id, passo);
                }
            }
        }
        if nuova {
            self.app.nuova_nota();
        }
        if cerca {
            self.app.mostra_finestra();
            // La ricerca sta nel menu di destra: se e' chiuso, si riapre.
            if super::schermo::telefono(ctx) {
                // Telefono: la ricerca e' in cima alla panoramica (delle note,
                // anche da una nota del cestino).
                super::principale::pagine::torna_alla_panoramica(&mut self.app);
            } else if self.app.preferenze.menu_chiuso {
                self.app.preferenze.menu_chiuso = false;
                self.app.salva_preferenze();
            }
            self.app.interfaccia.fuoco_ricerca = true;
        }
        if salva {
            self.app.salva_ora();
        }
        // Ctrl+D (D come desktop): la nota selezionata diventa un widget.
        // Su Android non ci sono widget sul desktop.
        if widget && !cfg!(target_os = "android") {
            let selezionata = self.app.interfaccia.selezionata.clone();
            if let Some(id) = selezionata.filter(|id| self.app.nota(id).is_some_and(|n| !n.nel_cestino)) {
                self.app.apri_widget(&id);
            }
        }
        if rinomina {
            if let Some(id) = self.app.interfaccia.selezionata.clone() {
                if let Some(nota) = self.app.nota(&id).filter(|n| !n.nel_cestino) {
                    let titolo = nota.titolo.clone();
                    self.app.interfaccia.rinomina = Some((id, titolo));
                }
            }
        }
    }

    fn gestisci_chiusura(&mut self, ctx: &egui::Context) {
        let richiesta = ctx.input(|i| i.viewport().close_requested());
        if richiesta && !self.app.uscita_confermata {
            if self.app.cartella.is_some() && !self.app.widget.is_empty() {
                // Chiudere la finestra principale lascia aperti i widget.
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                self.app.conferma_rinomina();
                self.app.salva_ora();
                self.app.finestra_visibile = false;
                self.app.salva_sessione();
            } else if !self.app.prepara_uscita() {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            }
        }
        self.invia_chiusura(ctx);
    }

    fn applica_visibilita(&mut self, ctx: &egui::Context) {
        // I primi fotogrammi restano visibili: i widget devono prima esistere,
        // altrimenti eframe non disegnerebbe nulla con la finestra nascosta.
        if self.fotogrammi < 3 {
            return;
        }
        let voluta = self.app.finestra_visibile || self.app.widget.is_empty();
        if voluta != self.finestra_mostrata || (voluta && std::mem::take(&mut self.porta_davanti)) {
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Visible(voluta));
            if voluta {
                ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Focus);
            }
            self.finestra_mostrata = voluta;
        }
    }

    fn ricorda_geometria(&mut self, ctx: &egui::Context) {
        if !self.finestra_mostrata {
            return;
        }
        let (esterno, interno, minimizzata) =
            ctx.input(|i| (i.viewport().outer_rect, i.viewport().inner_rect, i.viewport().minimized));
        if minimizzata == Some(true) {
            return;
        }
        if let (Some(esterno), Some(interno)) = (esterno, interno) {
            self.app.preferenze.finestra =
                Some([esterno.min.x, esterno.min.y, interno.width(), interno.height()]);
        }
    }
}

/// Consuma Ctrl+`tasto` (Cmd su macOS) solo se Alt non era premuto: egui
/// accetterebbe anche Ctrl+Alt+N come Ctrl+N, ma Ctrl+Alt+... e' per la nota
/// rapida. Conta lo stato dei tasti nell'evento stesso, non quello a fine
/// fotogramma (a quel punto Alt potrebbe essere gia' stato rilasciato).
pub(super) fn consuma_comando(i: &mut egui::InputState, tasto: Key) -> bool {
    let mut trovato = false;
    i.events.retain(|evento| {
        let corrisponde = matches!(
            evento,
            egui::Event::Key { key, pressed: true, modifiers, .. }
                if *key == tasto && modifiers.command && !modifiers.alt
        );
        trovato |= corrisponde;
        !corrisponde
    });
    trovato
}

/// Ctrl + rotella, Ctrl +, Ctrl - e Ctrl 0 sulla finestra in cui si trova il
/// mouse o la tastiera (la principale o un widget).
pub(super) fn gestisci_dimensione_testo(ctx: &egui::Context, app: &mut Applicazione) {
    if app.interfaccia.cattura_scorciatoia {
        return;
    }
    let (rotella, piu, meno, normale) = ctx.input_mut(|i| {
        (
            i.zoom_delta(),
            i.consume_key(Modifiers::COMMAND, Key::Plus) || i.consume_key(Modifiers::COMMAND, Key::Equals),
            i.consume_key(Modifiers::COMMAND, Key::Minus),
            i.consume_key(Modifiers::COMMAND, Key::Num0),
        )
    });
    if (rotella - 1.0).abs() > 0.001 {
        app.cambia_dimensione_testo(Some(rotella));
    }
    if piu {
        app.cambia_dimensione_testo(Some(PASSO_TESTO));
    }
    if meno {
        app.cambia_dimensione_testo(Some(1.0 / PASSO_TESTO));
    }
    if normale {
        app.cambia_dimensione_testo(None);
    }
}

/// Di quanto cambia il testo a ogni Ctrl + / Ctrl -.
const PASSO_TESTO: f32 = 1.1;

impl eframe::App for Programma {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Eseguito anche a finestra nascosta: e' qui che arrivano i comandi
        // dell'area di notifica (Mostra, Nuova nota, Esci).
        self.app.aggiorna();
        // Computer: "Come il sistema" segue il tema di Windows (o del desktop).
        #[cfg(not(target_os = "android"))]
        {
            self.app.interfaccia.tema_sistema = ctx.system_theme().map(|tema| tema == egui::Theme::Dark);
        }
        self.gestisci_area_notifica();
        #[cfg(not(target_os = "android"))]
        self.gestisci_scorciatoia_globale();
        #[cfg(target_os = "android")]
        self.gestisci_android(ctx);
        self.applica_visibilita(ctx);
        self.invia_chiusura(ctx);
    }

    #[cfg(target_os = "android")]
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // Passare a un'altra app (o spegnere lo schermo) toglie il fuoco.
        if raw_input.events.iter().any(|e| matches!(e, egui::Event::WindowFocused(false))) {
            self.salva_tutto();
        }
        self.tocchi(ctx, raw_input);
        // La tastiera da nascondere (chiesto nel fotogramma precedente e non
        // annullato da un "mostra" subito dopo): vedi la copia di android-activity.
        crate::piattaforma::android::applica_nascondi_tastiera();
        self.ponte.prima(ctx, raw_input);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.fotogrammi += 1;
        let t = Tavolozza::per(self.app.tema_scuro_attuale());
        if self.tema_applicato != Some(t.scuro) {
            tema::applica(&ctx, &t);
            self.tema_applicato = Some(t.scuro);
            // Barra di stato e di navigazione nei colori dell'app.
            #[cfg(target_os = "android")]
            crate::piattaforma::android::colora_barre(t.scuro, t.sfondo);
        }

        // Caselle delle liste e barra sopra la tastiera non tolgono il fuoco al testo.
        super::schermo::inizio_fotogramma(&ctx);
        super::tastiera::applica_richieste(&ctx);
        self.scorciatoie(&ctx);
        gestisci_dimensione_testo(&ctx, &mut self.app);
        #[cfg(not(target_os = "android"))]
        if self.app.cartella.is_none() {
            avvio::mostra(ui, &mut self.app, &t);
        } else {
            principale::mostra(ui, &mut self.app, &t);
            widget_fluttuante::mostra_tutti(&ctx, &mut self.app, &t);
        }
        #[cfg(target_os = "android")]
        {
            // Lo sfondo copre tutto, il contenuto evita barra di stato e tastiera.
            ui.painter().rect_filled(ctx.content_rect(), 0.0, t.sfondo);
            self.vibra_se_pressione_lunga(&ctx);
            let area = self.area_libera(&ctx);
            ui.scope_builder(egui::UiBuilder::new().max_rect(area), |ui| {
                if self.app.cartella.is_none() {
                    avvio::mostra(ui, &mut self.app, &t);
                } else {
                    principale::mostra(ui, &mut self.app, &t);
                }
            });
        }
        principale::dialoghi::mostra(&ctx, &mut self.app, &t);
        #[cfg(target_os = "android")]
        {
            self.ponte.dopo(&ctx, super::tastiera::prendi(&ctx));
            // Il gesto del dito ha bisogno di altri fotogrammi anche senza nuovi
            // tocchi (rilascio rimandato, attesa, slancio): chiesto qui, a fine
            // fotogramma, perche' durante la lettura dei tocchi andrebbe perso.
            match self.gesto {
                Gesto::InAttesa { .. } => ctx.request_repaint_after(Duration::from_millis(16)),
                Gesto::Slancio { .. } => ctx.request_repaint(),
                _ => {}
            }
            if !self.rilascio_in_sospeso.is_empty() {
                ctx.request_repaint();
            }
            // Tastiera in attesa (vedi `trattieni_tastiera`): per questo
            // fotogramma egui non la chiede.
            if self.trattieni_tastiera {
                ctx.output_mut(|o| o.ime = None);
            }
            // Copia e Taglia vanno negli appunti di Android.
            let copiati: Vec<String> = ctx.output(|o| {
                o.commands
                    .iter()
                    .filter_map(|c| match c {
                        egui::OutputCommand::CopyText(testo) => Some(testo.clone()),
                        _ => None,
                    })
                    .collect()
            });
            for testo in copiati {
                crate::piattaforma::android::copia_negli_appunti(&testo);
            }
        }

        self.gestisci_chiusura(&ctx);
        self.applica_visibilita(&ctx);
        self.ricorda_geometria(&ctx);

        if let Some(attesa) = self.app.prossima_scadenza() {
            ctx.request_repaint_after(attesa);
        }
        if self.app.rete.servizio.is_some() {
            // Aggiorna periodicamente l'elenco dei dispositivi presenti.
            ctx.request_repaint_after(Duration::from_secs(2));
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Trasparente: i widget disegnano da soli il proprio fondo arrotondato.
        [0.0, 0.0, 0.0, 0.0]
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Uscita forzata dal sistema (es. spegnimento): salvataggio di emergenza.
        if !self.app.uscita_confermata {
            self.app.prepara_uscita();
        }
    }
}
