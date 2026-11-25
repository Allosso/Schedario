//! Schedario per Android: il punto di ingresso chiamato da GameActivity e le
//! funzioni `...Nativo` che la classe Java `it.schedario.app.MainActivity`
//! chiama quando succede qualcosa dal suo lato (Indietro, "Nuova nota"
//! dall'icona, file scelto o salvato, tema chiaro/scuro del telefono cambiato). Tutto il programma e' nella libreria
//! `schedario` (cartella superiore), in `piattaforma::android`.

#![cfg(target_os = "android")]

use jni::sys::{JNIEnv, jboolean, jclass};
use schedario::piattaforma::android::{self, AndroidApp, Evento};

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android::avvia(app);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_it_schedario_app_MainActivity_indietroNativo(_env: *mut JNIEnv, _classe: jclass) {
    android::notifica(Evento::Indietro);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_it_schedario_app_MainActivity_nuovaNotaNativo(_env: *mut JNIEnv, _classe: jclass) {
    android::notifica(Evento::NuovaNota);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_it_schedario_app_MainActivity_fileApertoNativo(_env: *mut JNIEnv, _classe: jclass) {
    android::notifica(Evento::FileAperto(android::file_importato()));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_it_schedario_app_MainActivity_fileSalvatoNativo(
    _env: *mut JNIEnv,
    _classe: jclass,
    riuscito: jboolean,
) {
    android::notifica(Evento::FileSalvato(riuscito));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_it_schedario_app_MainActivity_temaCambiatoNativo(_env: *mut JNIEnv, _classe: jclass) {
    android::notifica(Evento::TemaCambiato);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_it_schedario_app_MainActivity_backupAutomaticoSceltoNativo(_env: *mut JNIEnv, _classe: jclass) {
    android::notifica(Evento::BackupAutomaticoScelto);
}
