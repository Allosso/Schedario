//! Barra laterale con sezioni retraibili: Cestino, Aspetto, Cartella e
//! backup, Rete locale, Assistente AI, Applicazione. Lo stato aperto/chiuso
//! viene ricordato. Sul telefono le stesse sezioni (senza Cestino e
//! Assistente AI) sono la pagina Impostazioni.

use egui::{Id, RichText, ScrollArea, TextEdit, Ui};

use crate::applicazione::{Applicazione, Avviso};
use crate::archivio::backup;
#[cfg(not(target_os = "android"))]
use crate::archivio::preferenze::Combinazione;
use crate::archivio::preferenze::DIMENSIONE_TESTO_PREDEFINITA;
#[cfg(not(target_os = "android"))]
use crate::interfaccia::scorciatoia_globale::tasto_supportato;
use crate::interfaccia::tastiera;
use crate::interfaccia::componenti::{
    self, Icona, nota_attenuata, pulsante, pulsante_icona, pulsante_largo, pulsante_largo_colorato,
};
use crate::interfaccia::tema::{SPAZIO, TESTO_SECONDARIO, TESTO_VOCE, Tavolozza};
use crate::lingue::LINGUE;
use crate::piattaforma::ora_locale;
use crate::rete::cifratura;

/// Margine interno dei campi di testo: alti come i pulsanti e come i campi
/// di ricerca (`ricerca::campo_cestino`), con lo stesso margine a sinistra.
const MARGINE_CAMPO: egui::Vec2 = egui::vec2(10.0, (componenti::ALTEZZA_PULSANTE - 18.0) / 2.0);

pub fn mostra(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    componenti::riquadro(t).show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        // Sul telefono il menu e' gia' stretto: niente logo, subito le sezioni.
        if !crate::interfaccia::schermo::telefono(ui.ctx()) {
            intestazione(ui, app, t);
            ui.add_space(4.0);
        }
        crate::interfaccia::schermo::scorrevole(ScrollArea::vertical()).auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = SPAZIO;
            // Prima il Cestino (le note), poi le impostazioni: Rete locale e
            // Assistente AI vicine, perche' funzionano allo stesso modo
            // (interruttore, codice, Mostra · Copia · Nuovo codice).
            sezione(ui, app, t, "cestino", "sezione_cestino", true, cestino);
            sezione(ui, app, t, "aspetto", "sezione_aspetto", true, aspetto);
            sezione(ui, app, t, "file", "sezione_file", false, file);
            sezione(ui, app, t, "rete", "sezione_rete", false, rete);
            // Il server per gli assistenti AI c'e' solo sui computer.
            if !cfg!(target_os = "android") {
                sezione(ui, app, t, "mcp", "sezione_mcp", false, assistente);
            }
            sezione(ui, app, t, "applicazione", "sezione_applicazione", false, applicazione);
        });
    });
}

/// Telefono: la pagina Impostazioni, con le sezioni del menu tranne il
/// Cestino (che si apre dalla panoramica).
/// `true` se la pagina e' scorsa in cima (allora trascinandola verso il
/// basso si torna alla nota, vedi `pagine`).
pub fn impostazioni(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) -> bool {
    let uscita = crate::interfaccia::schermo::scorrevole(ScrollArea::vertical()).auto_shrink(false).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = SPAZIO;
        // Lo stesso ordine del menu del computer.
        sezione(ui, app, t, "aspetto", "sezione_aspetto", true, aspetto);
        sezione(ui, app, t, "file", "sezione_file", false, file);
        sezione(ui, app, t, "rete", "sezione_rete", false, rete);
        sezione(ui, app, t, "applicazione", "sezione_applicazione", false, applicazione);
    });
    uscita.state.clamped_offset().y <= 1.0
}

/// Logo e nome del programma in cima al menu.
fn intestazione(ui: &mut Ui, app: &Applicazione, t: &Tavolozza) {
    let risposta = ui
        .horizontal(|ui| {
            ui.add_space(4.0);
            componenti::logo(ui, 22.0);
            ui.label(RichText::new("schedario").size(17.0).color(t.testo));
        })
        .response;
    ui.interact(risposta.rect, Id::unique("logo_menu"), egui::Sense::hover()).on_hover_text(app.tr.t("motto"));
}

fn sezione(
    ui: &mut Ui,
    app: &mut Applicazione,
    t: &Tavolozza,
    chiave: &str,
    titolo: &str,
    aperta_inizialmente: bool,
    contenuto: fn(&mut Ui, &mut Applicazione, &Tavolozza),
) {
    let aperta = app.interfaccia.sezioni.get(chiave).copied().unwrap_or(aperta_inizialmente);
    if componenti::titolo_sezione(ui, t, app.tr.t(titolo), aperta) {
        app.interfaccia.sezioni.insert(chiave.to_owned(), !aperta);
    }
    if aperta {
        ui.indent(chiave, |ui| {
            ui.add_space(2.0);
            contenuto(ui, app, t);
            ui.add_space(6.0);
        });
    }
}

fn cestino(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let Some(cartella) = &app.cartella else { return };
    if cartella.note_cestino().is_empty() {
        nota_attenuata(ui, t, app.tr.t("cestino_vuoto"));
        return;
    }
    // In cima la ricerca del Cestino: cerca solo tra le note cancellate
    // (quella sopra il menu cerca solo tra le note).
    super::ricerca::campo_cestino(ui, app, t);
    let Some(cartella) = &app.cartella else { return };
    let segnaposto = app.tr.t("senza_titolo");
    let ricerca = app.interfaccia.ricerca_cestino.clone();
    let tutte = cartella.note_cestino();
    let totale = tutte.len();
    // Con una ricerca in corso il cestino mostra solo le note che corrispondono.
    let note: Vec<(String, String)> = tutte
        .into_iter()
        .filter(|n| n.corrisponde(&ricerca))
        .map(|n| (n.id.clone(), n.titolo_visibile(segnaposto).to_owned()))
        .collect();
    if ricerca.trim().is_empty() {
        nota_attenuata(ui, t, &app.tr.f("cestino_conteggio", &[("n", &totale.to_string())]));
    } else {
        let testo = app.tr.f("cestino_trovate", &[("n", &note.len().to_string()), ("totale", &totale.to_string())]);
        nota_attenuata(ui, t, &testo);
    }
    crate::interfaccia::schermo::scorrevole(ScrollArea::vertical()).id_salt("elenco_cestino").max_height(150.0).show(ui, |ui| {
        for (id, titolo) in &note {
            let scelta = app.interfaccia.selezionata.as_deref() == Some(id.as_str());
            if ui.selectable_label(scelta, titolo).on_hover_text(app.tr.t("aiuto_nota_cestino")).clicked() {
                app.conferma_rinomina();
                app.interfaccia.selezionata = Some(id.clone());
            }
        }
    });
    // Ripristina ed Elimina definitivamente stanno accanto alla nota aperta
    // nell'editor: qui solo l'elenco e Svuota cestino.
    nota_attenuata(ui, t, app.tr.t("cestino_istruzioni"));
    // In rosso come le altre azioni che cancellano per sempre.
    let svuota = pulsante_largo_colorato(ui, t, app.tr.t("svuota_cestino"), t.pericolo);
    if svuota.on_hover_text(app.tr.t("aiuto_svuota_cestino")).clicked() {
        app.avviso = Some(Avviso::ConfermaSvuotaCestino);
    }
}

fn aspetto(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    // Tema: come il sistema (o il telefono), chiaro o scuro. Uguale sul
    // computer e sul telefono; sul computer vale anche per i widget.
    let (automatico, scuro) = (app.preferenze.tema_automatico, app.preferenze.tema_scuro);
    componenti::etichetta(ui, app.tr.t("tema"));
    let voci = [
        (app.tr.t("tema_sistema"), app.tr.t("aiuto_tema_sistema"), automatico),
        (app.tr.t("tema_chiaro"), app.tr.t("aiuto_tema_chiaro"), !automatico && !scuro),
        (app.tr.t("tema_scuro"), app.tr.t("aiuto_tema_scuro"), !automatico && scuro),
    ];
    match componenti::scelte(ui, t, &voci) {
        Some(0) => app.imposta_tema(true, scuro),
        Some(1) => app.imposta_tema(false, false),
        Some(2) => app.imposta_tema(false, true),
        _ => {}
    }
    if !cfg!(target_os = "android") {
        nota_attenuata(ui, t, app.tr.t("tema_descrizione"));
    }

    // Dimensione del testo delle note: anche con Ctrl + rotella o Ctrl +/-.
    ui.add_space(4.0);
    componenti::etichetta(ui, app.tr.t("dimensione_testo"));
    ui.horizontal(|ui| {
        if pulsante(ui, t, "A-").on_hover_text(app.tr.t("testo_piu_piccolo")).clicked() {
            app.cambia_dimensione_testo(Some(1.0 / 1.1));
        }
        let percentuale = app.preferenze.dimensione_testo / DIMENSIONE_TESTO_PREDEFINITA * 100.0;
        let etichetta = ui.add(
            egui::Label::new(RichText::new(format!("{percentuale:.0}%")).size(TESTO_VOCE)).sense(egui::Sense::click()),
        );
        if etichetta.on_hover_text(app.tr.t("testo_normale")).clicked() {
            app.cambia_dimensione_testo(None);
        }
        if pulsante(ui, t, "A+").on_hover_text(app.tr.t("testo_piu_grande")).clicked() {
            app.cambia_dimensione_testo(Some(1.1));
        }
    });
    nota_attenuata(ui, t, app.tr.t("dimensione_testo_aiuto"));
}

/// Rete locale, nello stesso ordine dell'Assistente AI: in cima
/// l'interruttore con lo stato, poi nome e codice (Mostra · Copia · Nuovo
/// codice), i dispositivi collegati e, in fondo, gli indirizzi aggiunti a mano.
fn rete(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let mut attiva = app.rete.impostazioni.attiva;
    if ui.checkbox(&mut attiva, app.tr.t("rete_attiva")).on_hover_text(app.tr.t("aiuto_rete_attiva")).changed() {
        app.applica_impostazioni_rete(attiva);
    }
    if let Some(errore) = &app.rete.errore {
        componenti::errore(ui, t, errore);
    }
    match &app.rete.servizio {
        None => nota_attenuata(ui, t, app.tr.t("rete_spenta")),
        Some(servizio) => {
            let ip = crate::rete::scoperta::indirizzo_locale().map(|ip| format!("{ip} · ")).unwrap_or_default();
            nota_attenuata(ui, t, &format!("{ip}{}", app.tr.f("rete_in_ascolto", &[("porta", &servizio.porta().to_string())])));
        }
    }

    // Nome e codice si salvano da soli quando si lascia il campo o si preme
    // Invio: l'unico comando per accendere e spegnere e' la casella sopra.
    ui.add_space(4.0);
    componenti::etichetta(ui, app.tr.t("rete_nome"));
    let uscita = TextEdit::singleline(&mut app.rete.bozza_nome).id(Id::unique("rete_nome")).margin(MARGINE_CAMPO).min_size(egui::vec2(0.0, componenti::ALTEZZA_PULSANTE)).desired_width(f32::INFINITY).show(ui);
    tastiera::campo(ui, &uscita, &app.rete.bozza_nome, false, false);
    let nome = uscita.response.response.on_hover_text(app.tr.t("aiuto_rete_nome"));
    componenti::etichetta(ui, app.tr.t("rete_codice"));
    let uscita = TextEdit::singleline(&mut app.rete.bozza_codice).margin(MARGINE_CAMPO).min_size(egui::vec2(0.0, componenti::ALTEZZA_PULSANTE))
        .id(Id::unique("rete_codice"))
        .password(!app.rete.mostra_codice)
        .desired_width(f32::INFINITY)
        .show(ui);
    tastiera::campo(ui, &uscita, &app.rete.bozza_codice, false, true);
    let codice = uscita.response.response.on_hover_text(app.tr.t("aiuto_rete_codice"));
    if nome.lost_focus() || codice.lost_focus() {
        app.conferma_bozze_rete();
    }
    // Come nell'Assistente AI: Mostra · Copia · Nuovo codice.
    ui.horizontal_wrapped(|ui| {
        let mostra = app.rete.mostra_codice;
        if componenti::scelta(ui, t, app.tr.t("rete_mostra"), mostra).on_hover_text(app.tr.t("aiuto_rete_mostra")).clicked() {
            app.rete.mostra_codice = !mostra;
        }
        let vuoto = app.rete.bozza_codice.is_empty();
        let copia = ui
            .add_enabled_ui(!vuoto, |ui| pulsante(ui, t, app.tr.t("rete_copia")))
            .inner
            .on_hover_text(app.tr.t("aiuto_rete_copia"))
            .on_disabled_hover_text(app.tr.t("aiuto_rete_copia_vuoto"));
        if copia.clicked() {
            ui.ctx().copy_text(app.rete.bozza_codice.clone());
        }
        if pulsante(ui, t, app.tr.t("rete_genera")).on_hover_text(app.tr.t("aiuto_rete_genera")).clicked() {
            app.rete.bozza_codice = cifratura::genera_codice();
            app.rete.mostra_codice = true;
            app.conferma_bozze_rete();
        }
    });
    let codice = app.rete.bozza_codice.trim();
    if !codice.is_empty() && !cifratura::codice_robusto(codice) {
        componenti::errore(ui, t, app.tr.t("rete_codice_debole"));
    }
    nota_attenuata(ui, t, app.tr.t("rete_codice_aiuto"));

    if app.rete.servizio.is_some() {
        ui.add_space(4.0);
        componenti::etichetta(ui, app.tr.t("rete_dispositivi"));
        let dispositivi = app.rete.dispositivi();
        if dispositivi.is_empty() {
            nota_attenuata(ui, t, app.tr.t("rete_nessun_dispositivo"));
        }
        for d in dispositivi {
            let stato = match (d.raggiungibile, d.ultima_sincronizzazione) {
                (Some(false), _) => app.tr.t("rete_fallito").to_owned(),
                (_, Some(quando)) => app.tr.f("rete_ok", &[("ora", &ora_locale::ora(quando))]),
                _ => app.tr.t("rete_mai").to_owned(),
            };
            ui.label(RichText::new(&d.nome).size(TESTO_VOCE));
            nota_attenuata(ui, t, &format!("{} · {stato}", d.indirizzo));
        }
        if pulsante(ui, t, app.tr.t("rete_sincronizza")).on_hover_text(app.tr.t("aiuto_rete_sincronizza")).clicked() {
            app.sincronizza_ora();
        }
        // L'ordine delle schede resta di ciascun computer; qui lo si prende,
        // una volta, da un altro computer collegato.
        if let Some(nome) = &app.rete.ordine_in_attesa {
            nota_attenuata(ui, t, &app.tr.f("prendi_ordine_in_attesa", &[("da", nome)]));
        } else if pulsante_largo(ui, t, app.tr.t("prendi_ordine")).on_hover_text(app.tr.t("aiuto_prendi_ordine")).clicked() {
            app.avviso = Some(Avviso::PrendiOrdine { scelta: None });
        }
    }

    ui.add_space(4.0);
    componenti::etichetta(ui, app.tr.t("rete_manuali"));
    ui.horizontal(|ui| {
        let larghezza = ui.available_width() - 86.0;
        let suggerimento = app.tr.t("rete_indirizzo").to_owned();
        let uscita = TextEdit::singleline(&mut app.rete.bozza_indirizzo).margin(MARGINE_CAMPO).min_size(egui::vec2(0.0, componenti::ALTEZZA_PULSANTE))
            .id(Id::unique("rete_indirizzo"))
            .hint_text(suggerimento)
            .desired_width(larghezza)
            .show(ui);
        tastiera::campo(ui, &uscita, &app.rete.bozza_indirizzo, false, false);
        let campo = uscita.response.response.on_hover_text(app.tr.t("aiuto_rete_indirizzo"));
        let invio = campo.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if pulsante(ui, t, app.tr.t("rete_aggiungi")).on_hover_text(app.tr.t("aiuto_rete_aggiungi")).clicked() || invio {
            app.aggiungi_indirizzo_manuale();
        }
    });
    let mut da_rimuovere = None;
    for indirizzo in &app.rete.impostazioni.indirizzi_manuali {
        ui.horizontal(|ui| {
            if pulsante_icona(ui, t, Icona::Chiudi, app.tr.t("rete_rimuovi"), true).clicked() {
                da_rimuovere = Some(indirizzo.clone());
            }
            ui.label(RichText::new(indirizzo).size(TESTO_VOCE));
        });
    }
    if let Some(indirizzo) = da_rimuovere {
        app.rimuovi_indirizzo_manuale(&indirizzo);
    }
}

#[cfg(not(target_os = "android"))]
fn file(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    if let Some(cartella) = &app.cartella {
        nota_attenuata(ui, t, &cartella.radice().display().to_string());
    }
    if pulsante_largo(ui, t, app.tr.t("cambia_cartella")).on_hover_text(app.tr.t("aiuto_cambia_cartella")).clicked() {
        let iniziale = app.cartella.as_ref().map(|c| c.radice().to_path_buf());
        let mut dialogo = rfd::FileDialog::new().set_title(app.tr.t("cambia_cartella"));
        if let Some(cartella) = iniziale.as_ref().and_then(|p| p.parent()) {
            dialogo = dialogo.set_directory(cartella);
        }
        if let Some(scelta) = dialogo.pick_folder() {
            if let Err(errore) = app.apri_cartella(&scelta) {
                app.avviso = Some(Avviso::Errore(errore));
            }
        }
    }
    if pulsante_largo(ui, t, app.tr.t("esporta_backup")).on_hover_text(app.tr.t("aiuto_esporta_backup")).clicked() {
        if let Some(percorso) = rfd::FileDialog::new()
            .set_title(app.tr.t("esporta_backup"))
            .set_file_name("Schedario-backup.json")
            .add_filter(app.tr.t("file_backup"), &["json"])
            .save_file()
        {
            app.esporta_backup(&percorso);
        }
    }
    if pulsante_largo(ui, t, app.tr.t("importa_backup")).on_hover_text(app.tr.t("aiuto_importa_backup")).clicked() {
        if let Some(percorso) = rfd::FileDialog::new()
            .set_title(app.tr.t("importa_backup"))
            .add_filter(app.tr.t("file_backup"), &["json"])
            .pick_file()
        {
            app.avviso = Some(Avviso::ConfermaImportazione(percorso));
        }
    }
    nota_attenuata(ui, t, app.tr.t("solo_dispositivi"));
}

/// Esporta una nota come file di testo: si sceglie dove salvarla. Dal menu
/// della nota (clic destro sulla scheda, o tenendola premuta nella panoramica).
#[cfg(not(target_os = "android"))]
pub fn esporta_nota(app: &mut Applicazione, id: &str) {
    let titolo = app.nota(id).map(|n| n.titolo_visibile(app.tr.t("senza_titolo")).to_owned()).unwrap_or_default();
    let nome = format!("{}.txt", backup::nome_sicuro(&titolo, "Nota"));
    if let Some(percorso) = rfd::FileDialog::new()
        .set_title(app.tr.t("esporta_nota"))
        .set_file_name(&nome)
        .add_filter(app.tr.t("file_testo"), &["txt"])
        .save_file()
    {
        app.esporta_nota(id, &percorso);
    }
}

/// Android: il file si prepara nella cartella dell'app, poi Android chiede
/// dove salvarlo (per esempio in Download o in un servizio cloud).
#[cfg(target_os = "android")]
pub fn esporta_nota(app: &mut Applicazione, id: &str) {
    let titolo = app.nota(id).map(|n| n.titolo_visibile(app.tr.t("senza_titolo")).to_owned()).unwrap_or_default();
    let nome = format!("{}.txt", backup::nome_sicuro(&titolo, "Nota"));
    let id = id.to_owned();
    salva_su_android(app, &nome, "text/plain", &|app, p| app.esporta_nota(&id, p));
}

/// Android: prepara il file con `prepara`, poi Android chiede dove salvarlo.
#[cfg(target_os = "android")]
fn salva_su_android(app: &mut Applicazione, nome: &str, tipo: &str, prepara: &dyn Fn(&mut Applicazione, &std::path::Path)) {
    use crate::piattaforma::android;
    let percorso = android::file_in_uscita(nome);
    if let Some(cartella) = percorso.parent() {
        let _ = std::fs::create_dir_all(cartella);
    }
    prepara(app, &percorso);
    if matches!(app.avviso, Some(Avviso::Info(_))) {
        app.avviso = None;
        android::salva_file(&percorso, nome, tipo);
    }
}

/// Android: le note stanno nella cartella privata dell'app (non si cambia);
/// i backup si salvano e si aprono con le finestre di Android, per esempio
/// in Download o in un servizio cloud.
#[cfg(target_os = "android")]
fn file(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    nota_attenuata(ui, t, app.tr.t("cartella_privata_android"));
    if pulsante_largo(ui, t, app.tr.t("esporta_backup")).on_hover_text(app.tr.t("aiuto_esporta_backup")).clicked() {
        salva_su_android(app, "Schedario-backup.json", "application/json", &|app, p| app.esporta_backup(p));
    }
    if pulsante_largo(ui, t, app.tr.t("importa_backup")).on_hover_text(app.tr.t("aiuto_importa_backup")).clicked() {
        crate::piattaforma::android::apri_file();
    }
    nota_attenuata(ui, t, app.tr.t("solo_dispositivi"));
    backup_automatico(ui, app, t);
}

/// Android: backup automatico in un file fuori dall'app (vedi
/// `interfaccia::backup_automatico`). Spento: spiegazione e Attiva; acceso:
/// file, ultimo aggiornamento, Cambia file e Disattiva.
#[cfg(target_os = "android")]
fn backup_automatico(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    use crate::interfaccia::backup_automatico as backup;
    ui.add_space(6.0);
    componenti::etichetta(ui, app.tr.t("backup_automatico"));
    let Some(attivo) = app.preferenze.backup_automatico.clone() else {
        nota_attenuata(ui, t, app.tr.t("backup_automatico_spiegazione"));
        if pulsante_largo(ui, t, app.tr.t("backup_automatico_attiva"))
            .on_hover_text(app.tr.t("aiuto_backup_automatico_attiva"))
            .clicked()
        {
            backup::attiva();
        }
        return;
    };
    ui.label(RichText::new(app.tr.f("backup_automatico_file", &[("nome", &attivo.nome)])).size(TESTO_VOCE));
    let quando = match attivo.ultimo {
        Some(ms) => app.tr.f(
            "backup_automatico_ultimo",
            &[("quando", &crate::piattaforma::ora_locale::data_ora(ms, app.tr.codice()))],
        ),
        None => app.tr.t("backup_automatico_mai").to_owned(),
    };
    nota_attenuata(ui, t, &quando);
    if let Some(errore) = app.errore_backup_automatico.clone() {
        componenti::errore(ui, t, &errore);
    }
    let scelta = componenti::scelte(
        ui,
        t,
        &[
            (app.tr.t("backup_automatico_cambia"), app.tr.t("aiuto_backup_automatico_cambia"), false),
            (app.tr.t("backup_automatico_disattiva"), app.tr.t("aiuto_backup_automatico_disattiva"), false),
        ],
    );
    match scelta {
        Some(0) => backup::attiva(),
        Some(1) => backup::disattiva(app),
        _ => {}
    }
}

/// Assistente AI (MCP): accesso, permesso di modificare, codice e comando
/// per collegare Claude Code. Tutto spento all'inizio.
fn assistente(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    // Come la Rete locale: interruttore, eventuale errore e stato, poi il resto.
    let mut attivo = app.preferenze.mcp_attivo;
    if ui.checkbox(&mut attivo, app.tr.t("mcp_attiva")).on_hover_text(app.tr.t("aiuto_mcp_attiva")).changed() {
        app.imposta_mcp(attivo);
    }
    if app.preferenze.mcp_attivo {
        if let Some(errore) = &app.mcp.errore {
            componenti::errore(ui, t, errore);
        }
        if let Some(servizio) = &app.mcp.servizio {
            let stato = match servizio.ultimo_accesso() {
                Some(quando) => app.tr.f(
                    "mcp_ultimo_accesso",
                    &[("ora", &ora_locale::ora(quando)), ("n", &servizio.richieste().to_string())],
                ),
                None => app.tr.t("mcp_nessun_accesso").to_owned(),
            };
            let porta = servizio.porta().to_string();
            nota_attenuata(ui, t, &format!("{} · {stato}", app.tr.f("mcp_in_ascolto", &[("porta", &porta)])));
        }
    }
    nota_attenuata(ui, t, app.tr.t("mcp_aiuto"));
    if !app.preferenze.mcp_attivo {
        return;
    }
    ui.add_space(4.0);

    let mut modifiche = app.preferenze.mcp_modifiche;
    if ui.checkbox(&mut modifiche, app.tr.t("mcp_modifiche")).on_hover_text(app.tr.t("aiuto_mcp_modifiche")).changed() {
        app.preferenze.mcp_modifiche = modifiche;
        app.salva_preferenze();
    }
    let spiegazione = if modifiche { "mcp_modifiche_si" } else { "mcp_modifiche_no" };
    nota_attenuata(ui, t, app.tr.t(spiegazione));

    ui.add_space(4.0);
    componenti::etichetta(ui, app.tr.t("mcp_codice"));
    let codice = if app.mcp.mostra_codice {
        app.preferenze.mcp_codice.clone()
    } else {
        "•".repeat(app.preferenze.mcp_codice.chars().count().min(24))
    };
    ui.label(RichText::new(codice).monospace().size(TESTO_SECONDARIO)).on_hover_text(app.tr.t("aiuto_mcp_codice"));
    ui.horizontal_wrapped(|ui| {
        let mostra = app.mcp.mostra_codice;
        if componenti::scelta(ui, t, app.tr.t("rete_mostra"), mostra).on_hover_text(app.tr.t("aiuto_rete_mostra")).clicked() {
            app.mcp.mostra_codice = !mostra;
        }
        if pulsante(ui, t, app.tr.t("rete_copia")).on_hover_text(app.tr.t("aiuto_mcp_copia")).clicked() {
            ui.ctx().copy_text(app.preferenze.mcp_codice.clone());
        }
        if pulsante(ui, t, app.tr.t("mcp_nuovo_codice")).on_hover_text(app.tr.t("aiuto_mcp_nuovo_codice")).clicked() {
            app.rigenera_codice_mcp();
        }
    });
    if pulsante_largo(ui, t, app.tr.t("mcp_copia_comando")).on_hover_text(app.tr.t("aiuto_mcp_copia_comando")).clicked() {
        ui.ctx().copy_text(app.comando_claude_code());
    }
    nota_attenuata(ui, t, app.tr.t("mcp_comando_aiuto"));
}

fn applicazione(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    // Lingua: scelte come quelle del Tema.
    componenti::etichetta(ui, app.tr.t("lingua"));
    let aiuto = app.tr.t("aiuto_lingua");
    let voci: Vec<(&str, &str, bool)> = LINGUE.iter().map(|(codice, nome)| (*nome, aiuto, app.tr.codice() == *codice)).collect();
    if let Some(i) = componenti::scelte(ui, t, &voci) {
        let codice = LINGUE[i].0;
        app.imposta_lingua(codice);
    }
    // Avvio automatico e nota rapida da qualsiasi programma: solo sui computer.
    #[cfg(not(target_os = "android"))]
    {
        let mut avvio = app.avvio_automatico;
        if ui.checkbox(&mut avvio, app.tr.t("avvio_automatico")).on_hover_text(app.tr.t("aiuto_avvio_automatico")).changed() {
            app.imposta_avvio_automatico(avvio);
        }
        nota_attenuata(ui, t, app.tr.t("avvio_automatico_aiuto"));

        scorciatoia_globale(ui, app, t);
    }

    if pulsante_largo(ui, t, app.tr.t("informazioni")).on_hover_text(app.tr.t("aiuto_informazioni")).clicked() {
        app.avviso = Some(Avviso::Informazioni);
    }
    // Su Android le app non si chiudono a mano: Indietro manda Schedario in
    // secondo piano e il sistema lo chiude quando serve. Sul computer Esci
    // chiude anche i widget e l'icona nell'area di notifica.
    if !cfg!(target_os = "android")
        && pulsante_largo(ui, t, app.tr.t("esci")).on_hover_text(app.tr.t("aiuto_esci")).clicked()
    {
        app.prepara_uscita();
    }
}

/// Nota rapida: attivazione e scelta della combinazione di tasti.
/// "Cambia combinazione" aspetta il prossimo tasto premuto con Ctrl, Alt
/// (o Cmd su macOS); Esc annulla.
#[cfg(not(target_os = "android"))]
fn scorciatoia_globale(ui: &mut Ui, app: &mut Applicazione, t: &Tavolozza) {
    let descrizione = app.preferenze.combinazione.descrizione();
    let mut attiva = app.preferenze.scorciatoia_globale;
    let etichetta = app.tr.f("scorciatoia_globale", &[("tasti", &descrizione)]);
    if ui.checkbox(&mut attiva, etichetta).on_hover_text(app.tr.t("scorciatoia_globale_aiuto")).changed() {
        app.preferenze.scorciatoia_globale = attiva;
        app.salva_preferenze();
    }

    if app.interfaccia.cattura_scorciatoia {
        ui.label(RichText::new(app.tr.t("scorciatoia_premi")).color(t.accento).strong().size(TESTO_SECONDARIO));
        if let Some(avviso) = &app.interfaccia.avviso_cattura {
            componenti::errore(ui, t, avviso);
        }
        if pulsante(ui, t, app.tr.t("annulla")).on_hover_text(app.tr.t("aiuto_annulla_cattura")).clicked() {
            fine_cattura(app);
        }
        cattura_combinazione(ui, app);
        return;
    }

    ui.add_enabled_ui(attiva, |ui| {
        let pulsante = pulsante(ui, t, app.tr.t("scorciatoia_cambia"))
            .on_hover_text(app.tr.t("aiuto_scorciatoia_cambia"))
            .on_disabled_hover_text(app.tr.t("aiuto_scorciatoia_spenta"));
        if pulsante.clicked() {
            app.interfaccia.cattura_scorciatoia = true;
            app.interfaccia.avviso_cattura = None;
            // Nessun campo di testo deve ricevere i tasti nel frattempo.
            ui.memory_mut(|m| m.stop_text_input());
        }
    });
    match &app.errore_scorciatoia {
        Some(errore) if attiva => {
            let testo = app.tr.f("scorciatoia_errore", &[("errore", errore)]);
            componenti::errore(ui, t, &testo);
        }
        _ => nota_attenuata(ui, t, app.tr.t("scorciatoia_globale_aiuto")),
    }
}

#[cfg(not(target_os = "android"))]
fn fine_cattura(app: &mut Applicazione) {
    app.interfaccia.cattura_scorciatoia = false;
    app.interfaccia.avviso_cattura = None;
}

/// Legge il primo tasto premuto in questo fotogramma e, se va bene, lo salva.
#[cfg(not(target_os = "android"))]
fn cattura_combinazione(ui: &mut Ui, app: &mut Applicazione) {
    // Ctrl, Alt, Shift... arrivano anche come tasti a se': si aspetta il tasto vero.
    let solo_modificatore = |tasto: egui::Key| {
        let nome = tasto.name();
        ["Control", "Shift", "Alt", "Meta", "Super", "Command", "OS", "Hyper", "Fn", "CapsLock"]
            .iter()
            .any(|m| nome.starts_with(m))
    };
    let premuto = ui.input(|i| {
        i.events.iter().find_map(|e| match e {
            egui::Event::Key { key, pressed: true, modifiers, .. } if !solo_modificatore(*key) => Some((*key, *modifiers)),
            _ => None,
        })
    });
    let Some((tasto, modificatori)) = premuto else {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        return;
    };
    if tasto == egui::Key::Escape {
        fine_cattura(app);
        return;
    }
    let nome = tasto.name();
    let nuova = Combinazione {
        ctrl: modificatori.ctrl,
        alt: modificatori.alt,
        maiuscolo: modificatori.shift,
        comando: cfg!(target_os = "macos") && modificatori.mac_cmd,
        tasto: nome.to_owned(),
    };
    if !nuova.valida() {
        app.interfaccia.avviso_cattura = Some(app.tr.t("scorciatoia_serve_modificatore").to_owned());
    } else if !tasto_supportato(nome) {
        app.interfaccia.avviso_cattura = Some(app.tr.f("scorciatoia_tasto_non_valido", &[("tasto", nome)]));
    } else {
        app.preferenze.combinazione = nuova;
        app.salva_preferenze();
        fine_cattura(app);
    }
}
