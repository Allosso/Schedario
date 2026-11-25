//! Android: avvio, cartelle dell'app, collegamento con la parte Java
//! (`android/progetto/app/src/main/java/it/schedario/app/MainActivity.java`)
//! e tastiera del telefono (`tastiera`).
//!
//! La parte Java fa solo cio' che da Rust non si puo' fare: appunti, finestre
//! di Android per salvare e aprire file, tasto Indietro, scorciatoia "Nuova
//! nota" sull'icona, permesso di ricevere gli annunci della rete locale.
//! Comunica con Rust in due modi:
//! - Rust chiama metodi di `MainActivity` (JNI, `con_java`);
//! - Java chiama le funzioni `...Nativo` della libreria `android/src/lib.rs`,
//!   che mettono un `Evento` in coda e chiedono un fotogramma nuovo.

pub mod tastiera;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use jni::objects::{JObject, JString, JValue};
use jni::{jni_sig, jni_str};

pub use winit::platform::android::activity::AndroidApp;

use crate::piattaforma::argomenti::Argomenti;

static APP: OnceLock<AndroidApp> = OnceLock::new();
static CONTESTO: OnceLock<egui::Context> = OnceLock::new();
static EVENTI: Mutex<VecDeque<Evento>> = Mutex::new(VecDeque::new());

/// Cose successe dal lato Java, da gestire al prossimo fotogramma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evento {
    /// Tasto (o gesto) Indietro.
    Indietro,
    /// Scorciatoia "Nuova nota" tenendo premuta l'icona dell'app.
    NuovaNota,
    /// L'utente ha scelto un file da importare: eccone una copia.
    FileAperto(PathBuf),
    /// Esito di "salva con nome" (`false`: annullato o non riuscito).
    FileSalvato(bool),
    /// Il telefono e' passato dal tema chiaro a quello scuro, o viceversa.
    TemaCambiato,
}

/// Punto di ingresso, chiamato da `android_main` (android/src/lib.rs).
pub fn avvia(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("schedario"),
    );
    let base = cartella_app_di(&app);
    let _ = APP.set(app.clone());
    // Note e preferenze nella cartella privata dell'app: le altre app non la
    // vedono; per avere i file si usa "Esporta backup".
    let argomenti = Argomenti {
        cartella_note: Some(base.join("Note")),
        cartella_dati: Some(base.join("Dati")),
        mostra_finestra: true,
        aiuto: false,
    };
    if let Err(errore) = crate::interfaccia::avvia_android(argomenti, app) {
        log::error!("avvio non riuscito: {errore}");
    }
}

fn cartella_app_di(app: &AndroidApp) -> PathBuf {
    app.internal_data_path().unwrap_or_else(|| PathBuf::from("/data/local/tmp/schedario"))
}

/// Cartella privata dell'app (`getFilesDir()` in Java).
pub fn cartella_app() -> PathBuf {
    APP.get().map(cartella_app_di).unwrap_or_else(|| PathBuf::from("/data/local/tmp/schedario"))
}

/// Dove la parte Java copia il file scelto con "Importa backup".
pub fn file_importato() -> PathBuf {
    cartella_app().join("Scambio").join("importato.json")
}

/// Dove si prepara un file prima di salvarlo con la finestra di Android.
pub fn file_in_uscita(nome: &str) -> PathBuf {
    cartella_app().join("Scambio").join(nome)
}

/// Collega il contesto dell'interfaccia: serve per ridisegnare quando arriva
/// qualcosa da Java o dalla tastiera.
pub fn imposta_contesto(ctx: &egui::Context) {
    let _ = CONTESTO.set(ctx.clone());
    winit::platform::android::activity::set_text_event_hook_schedario(Box::new(ridisegna));
}

fn ridisegna() {
    if let Some(ctx) = CONTESTO.get() {
        ctx.request_repaint();
    }
}

/// Chiamata dal lato Java (attraverso android/src/lib.rs).
pub fn notifica(evento: Evento) {
    if let Ok(mut coda) = EVENTI.lock() {
        coda.push_back(evento);
    }
    ridisegna();
}

/// Gli eventi arrivati da Java dall'ultima volta.
pub fn eventi() -> Vec<Evento> {
    EVENTI.lock().map(|mut coda| coda.drain(..).collect()).unwrap_or_default()
}

/// Spazio coperto ai bordi dello schermo, in pixel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Coperto {
    pub alto: i32,
    pub basso: i32,
    pub sinistra: i32,
    pub destra: i32,
}

/// Spazio coperto: in alto la barra di stato, in basso la tastiera se e'
/// aperta, altrimenti la barra di navigazione. In orizzontale la barra di
/// navigazione puo' stare a sinistra o a destra.
pub fn spazio_coperto() -> Coperto {
    let Some(app) = APP.get() else { return Coperto::default() };
    let barre = app.window_insets_schedario(false);
    let tastiera = app.window_insets_schedario(true);
    Coperto {
        alto: barre.top,
        basso: barre.bottom.max(tastiera.bottom),
        sinistra: barre.left,
        destra: barre.right,
    }
}

/// La tastiera e' aperta (occupa spazio in basso)?
pub fn tastiera_aperta() -> bool {
    APP.get().is_some_and(|app| app.window_insets_schedario(true).bottom > 0)
}

/// Esegue il "nascondi tastiera" rimasto in sospeso dal fotogramma precedente.
pub fn applica_nascondi_tastiera() {
    if let Some(app) = APP.get() {
        app.applica_nascondi_tastiera_schedario();
    }
}

/// Il telefono usa il tema scuro (Impostazioni → Display)?
pub fn tema_scuro_di_sistema() -> Option<bool> {
    con_java(|env, attivita| env.call_method(attivita, jni_str!("temaScuro"), jni_sig!("()Z"), &[])?.z())
}

/// Colora barra di stato e barra di navigazione come l'app: fondo `colore`
/// (ARGB) e icone scure sul tema chiaro, chiare sul tema scuro.
pub fn colora_barre(scuro: bool, colore: egui::Color32) {
    let [r, g, b, _] = colore.to_array();
    let argb = i32::from_be_bytes([0xFF, r, g, b]);
    con_java(|env, attivita| {
        env.call_method(attivita, jni_str!("coloraBarre"), jni_sig!("(ZI)V"), &[JValue::Bool(scuro), JValue::Int(argb)])?;
        Ok(())
    });
}

/// Esegue `f` con l'ambiente Java e l'oggetto `MainActivity`.
fn con_java<T>(f: impl FnOnce(&mut jni::Env, &JObject) -> jni::errors::Result<T>) -> Option<T> {
    let app = APP.get()?;
    // Sicurezza: puntatori validi finche' l'app e' viva (li fornisce android-activity).
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) };
    let attivita = app.activity_as_ptr() as jni::sys::jobject;
    let esito = vm.attach_current_thread(|env| -> jni::errors::Result<T> {
        let attivita = unsafe { JObject::from_raw(env, attivita) };
        f(env, &attivita)
    });
    match esito {
        Ok(valore) => Some(valore),
        Err(errore) => {
            log::warn!("chiamata Java non riuscita: {errore:?}");
            None
        }
    }
}

fn stringa_java(env: &mut jni::Env, oggetto: JObject) -> jni::errors::Result<Option<String>> {
    if oggetto.is_null() {
        return Ok(None);
    }
    let testo = env.cast_local::<JString>(oggetto)?;
    Ok(Some(testo.try_to_string(env)?))
}

/// Copia il testo negli appunti di Android.
pub fn copia_negli_appunti(testo: &str) {
    con_java(|env, attivita| {
        let testo = JString::new(env, testo)?;
        env.call_method(attivita, jni_str!("copiaNegliAppunti"), jni_sig!("(Ljava/lang/String;)V"), &[JValue::Object(&testo)])?;
        Ok(())
    });
}

/// Il testo negli appunti di Android, se ce n'e'.
pub fn leggi_appunti() -> Option<String> {
    con_java(|env, attivita| {
        let oggetto = env.call_method(attivita, jni_str!("leggiAppunti"), jni_sig!("()Ljava/lang/String;"), &[])?.l()?;
        stringa_java(env, oggetto)
    })
    .flatten()
}

/// Nome del telefono come lo mostra Android (Impostazioni → Info sul
/// telefono), oppure il modello.
pub fn nome_dispositivo() -> Option<String> {
    con_java(|env, attivita| {
        let oggetto = env.call_method(attivita, jni_str!("nomeDispositivo"), jni_sig!("()Ljava/lang/String;"), &[])?.l()?;
        stringa_java(env, oggetto)
    })
    .flatten()
    .filter(|nome| !nome.trim().is_empty())
}

/// Apre la finestra di Android "salva con nome" per il file gia' pronto in
/// `percorso`; l'esito arriva come `Evento::FileSalvato`.
pub fn salva_file(percorso: &std::path::Path, nome: &str, tipo: &str) {
    let percorso = percorso.display().to_string();
    con_java(|env, attivita| {
        let percorso = JString::new(env, &percorso)?;
        let nome = JString::new(env, nome)?;
        let tipo = JString::new(env, tipo)?;
        env.call_method(
            attivita,
            jni_str!("salvaFile"),
            jni_sig!("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V"),
            &[JValue::Object(&percorso), JValue::Object(&nome), JValue::Object(&tipo)],
        )?;
        Ok(())
    });
}

/// Apre la finestra di Android per scegliere un file da importare; la copia
/// arriva come `Evento::FileAperto`.
pub fn apri_file() {
    let destinazione = file_importato().display().to_string();
    con_java(|env, attivita| {
        let destinazione = JString::new(env, &destinazione)?;
        env.call_method(attivita, jni_str!("apriFile"), jni_sig!("(Ljava/lang/String;)V"), &[JValue::Object(&destinazione)])?;
        Ok(())
    });
}

/// Una vibrazione leggera, come quella delle altre app sulla pressione lunga.
pub fn vibra() {
    con_java(|env, attivita| {
        env.call_method(attivita, jni_str!("vibra"), jni_sig!("()V"), &[])?;
        Ok(())
    });
}

/// Manda l'app in secondo piano (come il tasto Home), senza chiuderla.
pub fn in_secondo_piano() {
    con_java(|env, attivita| {
        env.call_method(attivita, jni_str!("vaiInSecondoPiano"), jni_sig!("()V"), &[])?;
        Ok(())
    });
}
