//! Finestre di messaggio modali (informazioni, errori, conferme).

use egui::{Id, Modal, RichText, ScrollArea};

use crate::applicazione::{Applicazione, Avviso};
use crate::interfaccia::componenti::{self, pulsante, pulsante_accento};
use crate::interfaccia::tema::Tavolozza;

/// Larghezza di una finestra di messaggio: quella voluta, ma mai piu' larga
/// dello schermo (sul telefono 440 punti non ci stanno).
fn larghezza(ctx: &egui::Context, voluta: f32) -> f32 {
    voluta.min(ctx.content_rect().width() - 72.0).max(120.0)
}

pub fn mostra(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza) {
    if matches!(app.avviso, Some(Avviso::Cronologia { .. })) {
        cronologia(ctx, app, t);
        return;
    }
    if matches!(app.avviso, Some(Avviso::PrendiOrdine { .. })) {
        prendi_ordine(ctx, app, t);
        return;
    }
    if matches!(app.avviso, Some(Avviso::Licenze)) {
        licenze(ctx, app, t);
        return;
    }
    let Some(avviso) = app.avviso.take() else { return };
    let mut chiudi = false;
    let mut conferma = false;
    let mut apri_licenze = false;

    let risposta = Modal::new(Id::unique("avviso")).frame(componenti::riquadro(t).inner_margin(20)).show(ctx, |ui| {
        ui.set_max_width(larghezza(ctx, 440.0));
        // Titolo a `TITOLO_DIALOGO`, rosso per gli errori e per cio' che elimina.
        match &avviso {
            Avviso::Info(testo) => {
                componenti::titolo_dialogo(ui, crate::NOME_APP, t.testo);
                ui.label(testo);
            }
            Avviso::Errore(testo) => {
                componenti::titolo_dialogo(ui, app.tr.t("errore"), t.pericolo);
                ui.label(testo);
            }
            Avviso::ConfermaImportazione(percorso) => {
                componenti::titolo_dialogo(ui, app.tr.t("importa_backup"), t.testo);
                ui.label(app.tr.t("conferma_importa"));
                // Sul telefono il file e' una copia interna: il percorso non dice nulla.
                if !cfg!(target_os = "android") {
                    componenti::nota_attenuata(ui, t, &percorso.display().to_string());
                }
            }
            Avviso::ConfermaEliminazione(id) => {
                let titolo = app.nota(id).map(|n| n.titolo_visibile(app.tr.t("senza_titolo")).to_owned()).unwrap_or_default();
                componenti::titolo_dialogo(ui, app.tr.t("elimina_definitivamente"), t.pericolo);
                ui.label(app.tr.f("conferma_elimina", &[("titolo", &titolo)]));
            }
            Avviso::ConfermaSvuotaCestino => {
                let quante = app.note_cestino_visibili().len();
                componenti::titolo_dialogo(ui, app.tr.t("svuota_cestino"), t.pericolo);
                ui.label(app.tr.f("conferma_svuota", &[("n", &quante.to_string())]));
            }
            Avviso::InvitoBackup => {
                componenti::titolo_dialogo(ui, app.tr.t("invito_backup_titolo"), t.testo);
                ui.label(app.tr.t("invito_backup_testo"));
            }
            Avviso::Cronologia { .. } | Avviso::PrendiOrdine { .. } | Avviso::Licenze => {}
            Avviso::Informazioni => {
                ui.horizontal(|ui| {
                    componenti::logo(ui, 32.0);
                    componenti::titolo_dialogo(ui, crate::NOME_APP, t.testo);
                });
                let cartella = app.cartella.as_ref().map(|c| c.cartella_note().display().to_string()).unwrap_or_default();
                ui.label(app.tr.f("info_testo", &[("versione", crate::VERSIONE), ("cartella", &cartella)]));
            }
        }
        // Pulsanti sempre nello stesso ordine in tutti i dialoghi: prima
        // l'azione, poi Annulla (o Chiudi) a destra.
        ui.add_space(10.0);
        ui.horizontal(|ui| match &avviso {
            Avviso::ConfermaImportazione(_) => {
                if pulsante_accento(ui, t, app.tr.t("importa")).clicked() {
                    conferma = true;
                }
                if pulsante(ui, t, app.tr.t("annulla")).clicked() {
                    chiudi = true;
                }
            }
            Avviso::ConfermaEliminazione(_) | Avviso::ConfermaSvuotaCestino => {
                // L'azione distruttiva e' rossa; quella evidenziata e' Annulla.
                if componenti::pulsante_colorato(ui, t, app.tr.t("elimina"), t.pericolo).clicked() {
                    conferma = true;
                }
                if pulsante_accento(ui, t, app.tr.t("annulla")).clicked() {
                    chiudi = true;
                }
            }
            Avviso::InvitoBackup => {
                if pulsante_accento(ui, t, app.tr.t("backup_automatico_attiva"))
                    .on_hover_text(app.tr.t("aiuto_backup_automatico_attiva"))
                    .clicked()
                {
                    conferma = true;
                }
                if pulsante(ui, t, app.tr.t("piu_tardi")).clicked() {
                    chiudi = true;
                }
            }
            Avviso::Informazioni => {
                if pulsante(ui, t, app.tr.t("licenze")).on_hover_text(app.tr.t("aiuto_licenze")).clicked() {
                    apri_licenze = true;
                }
                if pulsante_accento(ui, t, app.tr.t("ok")).clicked() {
                    chiudi = true;
                }
            }
            _ => {
                if pulsante_accento(ui, t, app.tr.t("ok")).clicked() {
                    chiudi = true;
                }
            }
        });
    });

    if apri_licenze {
        app.avviso = Some(Avviso::Licenze);
        return;
    }
    if conferma {
        match &avviso {
            Avviso::ConfermaImportazione(percorso) => app.importa_backup(percorso),
            Avviso::ConfermaEliminazione(id) => app.elimina_definitivamente(id),
            // Telefono: dopo la conferma anche ANNULLA per 10 secondi (pagina Cestino).
            Avviso::ConfermaSvuotaCestino if crate::interfaccia::schermo::telefono(ctx) => {
                let tutte = app.note_cestino_visibili().into_iter().map(|n| n.id.clone()).collect();
                app.elimina_con_annulla(tutte);
            }
            Avviso::ConfermaSvuotaCestino => app.svuota_cestino(),
            #[cfg(target_os = "android")]
            Avviso::InvitoBackup => crate::interfaccia::backup_automatico::attiva(),
            _ => {}
        }
        return;
    }
    if chiudi || risposta.should_close() {
        return;
    }
    // Ancora aperta: se nel frattempo non e' arrivato un altro avviso, la teniamo.
    if app.avviso.is_none() {
        app.avviso = Some(avviso);
    }
}

/// Testi delle licenze, generati da `script/genera-licenze.ps1`: la licenza di
/// Schedario e quelle di librerie e caratteri inclusi, che chiedono di essere
/// distribuite insieme al programma (anche nell'app per il telefono).
const LICENZE: &str = include_str!("../../../assets/licenze/licenze.txt");

fn licenze(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza) {
    static RIGHE: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    let righe = RIGHE.get_or_init(|| LICENZE.lines().collect());
    let mut chiudi = false;

    let risposta = Modal::new(Id::unique("licenze")).frame(componenti::riquadro(t).inner_margin(20)).show(ctx, |ui| {
        ui.set_width(larghezza(ctx, 640.0));
        componenti::titolo_dialogo(ui, app.tr.t("licenze"), t.testo);
        // Circa 15 000 righe: si dispongono solo quelle visibili. Le righe
        // non vanno a capo (sono testi gia' impaginati): si scorre anche di lato.
        let carattere = egui::FontId::monospace(crate::interfaccia::tema::TESTO_SECONDARIO - 1.0);
        let altezza = ui.fonts_mut(|f| f.row_height(&carattere));
        ScrollArea::both()
            .id_salt("testi_licenze")
            .max_height(ctx.content_rect().height() * 0.6)
            .auto_shrink([false, true])
            .show_rows(ui, altezza, righe.len(), |ui, intervallo| {
                for riga in &righe[intervallo] {
                    ui.add(egui::Label::new(RichText::new(*riga).font(carattere.clone()).color(t.testo)).extend());
                }
            });
        ui.add_space(10.0);
        if pulsante_accento(ui, t, app.tr.t("chiudi")).clicked() {
            chiudi = true;
        }
    });
    if chiudi || risposta.should_close() {
        app.avviso = None;
    }
}

/// "Prendi l'ordine delle schede da…": elenco dei computer collegati, poi
/// Annulla o Conferma. Cambia solo questo computer, mai l'altro.
fn prendi_ordine(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza) {
    let Some(Avviso::PrendiOrdine { mut scelta }) = app.avviso.take() else { return };
    let mut chiudi = false;
    let mut conferma = false;
    // Solo i computer che hanno gia' risposto con la loro identita'.
    let dispositivi: Vec<_> = app.rete.dispositivi().into_iter().filter(|d| !d.id.starts_with("manuale:")).collect();

    let risposta = Modal::new(Id::unique("prendi_ordine")).frame(componenti::riquadro(t).inner_margin(20)).show(ctx, |ui| {
        ui.set_width(larghezza(ctx, 440.0));
        componenti::titolo_dialogo(ui, app.tr.t("prendi_ordine_titolo"), t.testo);
        if dispositivi.is_empty() {
            ui.label(app.tr.t("rete_nessun_dispositivo"));
        } else {
            ui.label(app.tr.t("prendi_ordine_scegli"));
            ui.add_space(4.0);
            for d in &dispositivi {
                let raggiungibile = d.raggiungibile != Some(false);
                let stato = match (d.raggiungibile, d.ultima_sincronizzazione) {
                    (Some(false), _) => app.tr.t("rete_fallito").to_owned(),
                    (_, Some(quando)) => app.tr.f("rete_ok", &[("ora", &crate::piattaforma::ora_locale::ora(quando))]),
                    _ => app.tr.t("rete_mai").to_owned(),
                };
                let etichetta = format!("{} · {stato}", d.nome);
                let voce = ui
                    .add_enabled(raggiungibile, egui::Button::selectable(scelta.as_deref() == Some(d.id.as_str()), etichetta))
                    .on_hover_text(app.tr.t("aiuto_prendi_ordine_computer"))
                    .on_disabled_hover_text(app.tr.f("prendi_ordine_non_raggiungibile", &[("da", &d.nome)]));
                if voce.clicked() {
                    scelta = Some(d.id.clone());
                }
            }
        }
        let nome_scelto = scelta.as_ref().and_then(|s| dispositivi.iter().find(|d| &d.id == s)).map(|d| d.nome.clone());
        if let Some(nome) = &nome_scelto {
            ui.add_space(6.0);
            ui.label(app.tr.f("prendi_ordine_conferma", &[("da", nome)]));
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let pronto = nome_scelto.is_some();
            let conferma_pulsante = ui
                .add_enabled_ui(pronto, |ui| pulsante_accento(ui, t, app.tr.t("conferma")))
                .inner
                .on_disabled_hover_text(app.tr.t("aiuto_prendi_ordine_scegli"));
            if conferma_pulsante.clicked() {
                conferma = true;
            }
            if pulsante(ui, t, app.tr.t("annulla")).clicked() {
                chiudi = true;
            }
        });
    });

    if conferma {
        if let Some(id) = scelta {
            app.prendi_ordine_da(&id);
        }
        return;
    }
    if chiudi || risposta.should_close() {
        return;
    }
    if app.avviso.is_none() {
        app.avviso = Some(Avviso::PrendiOrdine { scelta });
    }
}

/// Cronologia di una nota: elenco delle versioni a sinistra, anteprima a
/// destra, "Ripristina questa versione" in basso.
fn cronologia(ctx: &egui::Context, app: &mut Applicazione, t: &Tavolozza) {
    let Some(Avviso::Cronologia { id, versioni, mut scelta }) = app.avviso.take() else { return };
    let mut chiudi = false;
    let mut ripristina = false;
    let titolo_nota = app.nota(&id).map(|n| n.titolo_visibile(app.tr.t("senza_titolo")).to_owned()).unwrap_or_default();

    let risposta = Modal::new(Id::unique("cronologia")).frame(componenti::riquadro(t).inner_margin(20)).show(ctx, |ui| {
        ui.set_width(larghezza(ctx, 640.0));
        componenti::titolo_dialogo(ui, &app.tr.f("cronologia_di", &[("titolo", &titolo_nota)]), t.testo);
        if versioni.is_empty() {
            componenti::nota_attenuata(ui, t, app.tr.t("cronologia_vuota"));
        } else {
            componenti::nota_attenuata(ui, t, app.tr.t("cronologia_aiuto"));
            ui.add_space(6.0);
            const ALTEZZA: f32 = 320.0;
            ui.allocate_ui(egui::vec2(ui.available_width(), ALTEZZA), |ui| {
                ui.horizontal_top(|ui| {
                    crate::interfaccia::schermo::scorrevole(ScrollArea::vertical()).id_salt("cronologia_elenco").max_height(ALTEZZA).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(190.0);
                            for (i, versione) in versioni.iter().enumerate() {
                                let data = crate::piattaforma::ora_locale::data_ora(versione.quando, app.tr.codice());
                                let nota = crate::modello::Nota { testo: versione.testo.clone(), ..crate::modello::Nota::nuova("") };
                                let parole = app.tr.parole(nota.conta_parole());
                                if ui.selectable_label(scelta == i, format!("{data}\n{parole}")).clicked() {
                                    scelta = i;
                                }
                            }
                        });
                    });
                    ui.separator();
                    if let Some(versione) = versioni.get(scelta) {
                        crate::interfaccia::schermo::scorrevole(ScrollArea::vertical()).id_salt("cronologia_anteprima").max_height(ALTEZZA).show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width());
                                let titolo =
                                    if versione.titolo.trim().is_empty() { app.tr.t("senza_titolo") } else { &versione.titolo };
                                ui.label(RichText::new(titolo).strong().size(15.0));
                                ui.add(egui::Label::new(RichText::new(&versione.testo).size(14.0)).wrap());
                            });
                        });
                    }
                });
            });
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if !versioni.is_empty()
                && pulsante_accento(ui, t, app.tr.t("cronologia_ripristina"))
                    .on_hover_text(app.tr.t("aiuto_cronologia_ripristina"))
                    .clicked()
            {
                ripristina = true;
            }
            if pulsante(ui, t, app.tr.t("chiudi")).clicked() {
                chiudi = true;
            }
        });
    });

    if ripristina {
        if let Some(versione) = versioni.get(scelta) {
            app.ripristina_versione(&id, versione);
        }
        return;
    }
    if chiudi || risposta.should_close() || app.nota(&id).is_none() {
        return;
    }
    if app.avviso.is_none() {
        app.avviso = Some(Avviso::Cronologia { id, versioni, scelta });
    }
}
