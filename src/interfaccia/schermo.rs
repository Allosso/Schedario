//! Telefono o computer: quale disposizione usare e quanto grandi fare i
//! controlli.
//!
//! - **Tocco** (`TOCCO`): su Android i controlli sono piu' alti, perche' si
//!   usano con il dito; vale anche con il telefono in orizzontale o su un tablet.
//! - **Disposizione da telefono** (`telefono`): schermo stretto, come un
//!   telefono in verticale. Le schede usano tutta la larghezza, in basso c'e'
//!   una barra con Impostazioni, Cerca, Note e Nuova, e impostazioni, ricerca e
//!   cestino sono pagine a schermo intero. Con almeno `LARGHEZZA_COMPUTER` punti (telefono
//!   in orizzontale, tablet) si usa la disposizione del computer: lavoro a
//!   sinistra, menu sempre aperto a destra.
//!
//! Per provare la disposizione da telefono sul computer, nella versione di
//! sviluppo: variabile d'ambiente `SCHEDARIO_TELEFONO=1`.

/// Si usa con il dito (Android).
pub const TOCCO: bool = cfg!(target_os = "android");

/// Da questa larghezza in su (in punti) si usa la disposizione del computer.
/// Un telefono in verticale ne ha circa 360, in orizzontale circa 780.
pub const LARGHEZZA_COMPUTER: f32 = 720.0;

/// Disposizione da telefono per questo schermo?
pub fn telefono(ctx: &egui::Context) -> bool {
    forzato() || TOCCO && ctx.content_rect().width() < LARGHEZZA_COMPUTER
}

/// Versione di sviluppo con `SCHEDARIO_TELEFONO=1`.
fn forzato() -> bool {
    cfg!(debug_assertions) && std::env::var_os("SCHEDARIO_TELEFONO").is_some_and(|v| v == "1")
}

/// Zone in cui un tocco o un clic non toglie il fuoco al campo di testo:
/// le caselle delle liste e la barra sopra la tastiera. Per esempio, mentre
/// scrivi una lista puoi spuntare una voce senza che la tastiera si chiuda e
/// senza perdere il punto in cui stavi scrivendo.
///
/// Ogni zona si registra mentre viene disegnata e vale dal fotogramma
/// successivo; `inizio_fotogramma` decide se il puntatore e' in una zona.
pub fn proteggi_fuoco(ctx: &egui::Context, zona: egui::Rect) {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Vec<egui::Rect>>(chiave_zone()).push(zona));
}

/// Da chiamare all'inizio di ogni fotogramma, prima di disegnare.
pub fn inizio_fotogramma(ctx: &egui::Context) {
    let zone: Vec<egui::Rect> = ctx.data_mut(|d| d.remove_temp(chiave_zone())).unwrap_or_default();
    let puntatore = ctx.input(|i| i.pointer.interact_pos().or(i.pointer.latest_pos()));
    let dentro = puntatore.is_some_and(|p| zone.iter().any(|z| z.contains(p)));
    ctx.options_mut(|o| {
        o.input_options.surrender_focus_on =
            if dentro { egui::SurrenderFocusOn::Never } else { egui::SurrenderFocusOn::Clicks };
    });
}

fn chiave_zone() -> egui::Id {
    egui::Id::unique("zone_che_tengono_il_fuoco")
}

/// Un'area che scorre: sul telefono anche trascinandola con il dito. egui lo
/// fa da solo solo se riconosce uno schermo touch, e su Android non sempre
/// ci riesce: qui lo si chiede sempre.
pub fn scorrevole(area: egui::ScrollArea) -> egui::ScrollArea {
    if TOCCO {
        area.scroll_source(egui::scroll_area::ScrollSource {
            drag: egui::scroll_area::DragScroll::Always,
            ..Default::default()
        })
    } else {
        area
    }
}

/// Misura di un controllo: quella del computer, oppure quella per il dito.
/// Per esempio un pulsante alto 30 punti sul telefono ne e' alto 42: con lo
/// spazio attorno si arriva ai 48 dp che Android consiglia per un tocco.
pub const fn misura(computer: f32, tocco: f32) -> f32 {
    if TOCCO { tocco } else { computer }
}

/// Telefono: la zona della nota in cui trascinare il dito di lato passa alla
/// nota accanto (vedi `Programma::smista_gesti`). L'editor la registra a ogni
/// fotogramma; vale solo mentre si vede la nota, non per menu o panoramica.
pub fn zona_sfoglia(ctx: &egui::Context, zona: Option<egui::Rect>) {
    ctx.data_mut(|d| match zona {
        Some(z) => {
            d.insert_temp(chiave_zona_sfoglia(), z);
        }
        None => {
            d.remove::<egui::Rect>(chiave_zona_sfoglia());
        }
    });
}

/// La zona registrata dall'ultimo fotogramma disegnato.
pub fn zona_sfoglia_attuale(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp(chiave_zona_sfoglia()))
}

/// Di quanto il dito ha spostato la nota di lato (`None`: nessuno spostamento).
pub fn sfoglia(ctx: &egui::Context) -> Option<f32> {
    ctx.data(|d| d.get_temp(chiave_sfoglia()))
}

pub fn imposta_sfoglia(ctx: &egui::Context, spostamento: Option<f32>) {
    ctx.data_mut(|d| match spostamento {
        Some(dx) => {
            d.insert_temp(chiave_sfoglia(), dx);
        }
        None => {
            d.remove::<f32>(chiave_sfoglia());
        }
    });
}

fn chiave_zona_sfoglia() -> egui::Id {
    egui::Id::unique("zona_sfoglia")
}

fn chiave_sfoglia() -> egui::Id {
    egui::Id::unique("sfoglia_spostamento")
}

/// Telefono: l'area non coperta da barra di stato e tastiera (sul computer
/// tutta la finestra). Serve ai menu, che altrimenti finirebbero sotto la tastiera.
pub fn area_libera(ctx: &egui::Context) -> egui::Rect {
    ctx.data(|d| d.get_temp(chiave_area_libera())).unwrap_or_else(|| ctx.content_rect())
}

pub fn imposta_area_libera(ctx: &egui::Context, area: egui::Rect) {
    ctx.data_mut(|d| d.insert_temp(chiave_area_libera(), area));
}

/// Telefono: il pulsante Cestino (riga della data o panoramica), dove si puo' lasciare
/// una scheda trascinata (registrato a ogni fotogramma dalla barra).
pub fn zona_cestino(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp(chiave_zona_cestino()))
}

pub fn imposta_zona_cestino(ctx: &egui::Context, zona: Option<egui::Rect>) {
    ctx.data_mut(|d| match zona {
        Some(z) => {
            d.insert_temp(chiave_zona_cestino(), z);
        }
        None => {
            d.remove::<egui::Rect>(chiave_zona_cestino());
        }
    });
}

/// Si sta trascinando una nota (scheda o casella della panoramica): il
/// Cestino in basso si colora per dire che la si puo' lasciare li'.
pub fn si_trascina_nota(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp(chiave_trascina_nota())).unwrap_or(false)
}

pub fn imposta_trascina_nota(ctx: &egui::Context, si: bool) {
    ctx.data_mut(|d| d.insert_temp(chiave_trascina_nota(), si));
}

fn chiave_area_libera() -> egui::Id {
    egui::Id::unique("area_libera")
}

fn chiave_zona_cestino() -> egui::Id {
    egui::Id::unique("zona_cestino_barra")
}

fn chiave_trascina_nota() -> egui::Id {
    egui::Id::unique("si_trascina_nota")
}

/// Telefono: la maniglia in basso al centro (trascinata verso l'alto apre
/// tutte le note) e la barra sopra la tastiera (trascinata verso il basso
/// chiude la tastiera). Le registra chi le disegna; le legge
/// `Programma::smista_gesti`, che decide il gesto prima di egui.
pub fn zona_maniglia(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp(egui::Id::unique("zona_maniglia")))
}

pub fn imposta_zona_maniglia(ctx: &egui::Context, zona: Option<egui::Rect>) {
    imposta_zona(ctx, egui::Id::unique("zona_maniglia"), zona);
}

/// Telefono: quale pagina accanto alla panoramica (Impostazioni a destra,
/// Cestino a sinistra) il dito sta tirando di lato, e quanto si vede (0 solo
/// la panoramica, 1 solo la pagina). `None` quando nessun dito trascina:
/// allora `principale::telefono` la porta da sola a 0 o a 1 con
/// un'animazione.
pub fn scorri_laterale(ctx: &egui::Context) -> Option<(crate::applicazione::Pagina, f32)> {
    ctx.data(|d| d.get_temp(egui::Id::unique("scorri_laterale")))
}

pub fn imposta_scorri_laterale(ctx: &egui::Context, valore: Option<(crate::applicazione::Pagina, f32)>) {
    let chiave = egui::Id::unique("scorri_laterale");
    ctx.data_mut(|d| match valore {
        Some(v) => {
            d.insert_temp(chiave, v);
        }
        None => {
            d.remove::<(crate::applicazione::Pagina, f32)>(chiave);
        }
    });
}

/// Telefono: dove si tira verso il basso Impostazioni o Cestino per tornare
/// alla nota (la testata; tutta la pagina quando e' scorsa in cima).
pub fn zona_chiudi_pagina(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp(egui::Id::unique("zona_chiudi_pagina")))
}

pub fn imposta_zona_chiudi_pagina(ctx: &egui::Context, zona: Option<egui::Rect>) {
    imposta_zona(ctx, egui::Id::unique("zona_chiudi_pagina"), zona);
}

/// Telefono: di quanto (0-1 dell'altezza) il dito ha tirato giu' la pagina.
pub fn chiudi_pagina(ctx: &egui::Context) -> Option<f32> {
    ctx.data(|d| d.get_temp(egui::Id::unique("chiudi_pagina")))
}

pub fn imposta_chiudi_pagina(ctx: &egui::Context, valore: Option<f32>) {
    let chiave = egui::Id::unique("chiudi_pagina");
    ctx.data_mut(|d| match valore {
        Some(v) => {
            d.insert_temp(chiave, v);
        }
        None => {
            d.remove::<f32>(chiave);
        }
    });
}

/// Telefono: la nota del cestino tirata giu' dalla sua testata. Sotto c'e'
/// gia' la nota a cui si torna; `esce` dice se, sollevato il dito, finisce di
/// scendere (si torna alle note) o risale (resta aperta lei).
#[derive(Clone, Debug, PartialEq)]
pub struct NotaCheScende {
    pub id: String,
    pub esce: bool,
}

pub fn nota_che_scende(ctx: &egui::Context) -> Option<NotaCheScende> {
    ctx.data(|d| d.get_temp(egui::Id::unique("nota_che_scende")))
}

pub fn imposta_nota_che_scende(ctx: &egui::Context, nota: Option<NotaCheScende>) {
    let chiave = egui::Id::unique("nota_che_scende");
    ctx.data_mut(|d| match nota {
        Some(n) => {
            d.insert_temp(chiave, n);
        }
        None => {
            d.remove::<NotaCheScende>(chiave);
        }
    });
}

/// Telefono: la pagina lasciata andare verso il basso, che finisce di
/// scendere da sola sopra la nota.
pub fn pagina_uscente(ctx: &egui::Context) -> Option<crate::applicazione::Pagina> {
    ctx.data(|d| d.get_temp(egui::Id::unique("pagina_uscente")))
}

pub fn imposta_pagina_uscente(ctx: &egui::Context, pagina: Option<crate::applicazione::Pagina>) {
    let chiave = egui::Id::unique("pagina_uscente");
    ctx.data_mut(|d| match pagina {
        Some(p) => {
            d.insert_temp(chiave, p);
        }
        None => {
            d.remove::<crate::applicazione::Pagina>(chiave);
        }
    });
}

/// Telefono: dove si vede il testo della nota. Li' trascinare fa sempre
/// scorrere e si seleziona solo tenendo premuto (`Programma::smista_gesti`).
/// La registra `testo_nota::testo`.
pub fn zona_testo(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp(egui::Id::unique("zona_testo")))
}

pub fn imposta_zona_testo(ctx: &egui::Context, zona: Option<egui::Rect>) {
    imposta_zona(ctx, egui::Id::unique("zona_testo"), zona);
}

/// Telefono: le maniglie della selezione nel testo (dove le si prende con il
/// dito): li' il tocco arriva subito a egui, per trascinarle.
pub fn maniglie_selezione(ctx: &egui::Context) -> Vec<egui::Rect> {
    ctx.data(|d| d.get_temp::<Vec<egui::Rect>>(egui::Id::unique("maniglie_selezione"))).unwrap_or_default()
}

pub fn imposta_maniglie_selezione(ctx: &egui::Context, zone: Vec<egui::Rect>) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("maniglie_selezione"), zone));
}

/// Telefono: la meta' bassa della nota quando il testo non puo' scorrere piu'
/// giu' (nota corta, o letta fino in fondo). Da li', trascinando verso
/// l'alto, si apre la panoramica come dalla fascia della maniglia; altrimenti
/// lo stesso gesto fa scorrere il testo. La registra `editor`.
pub fn zona_apri_panoramica(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp(egui::Id::unique("zona_apri_panoramica")))
}

pub fn imposta_zona_apri_panoramica(ctx: &egui::Context, zona: Option<egui::Rect>) {
    imposta_zona(ctx, egui::Id::unique("zona_apri_panoramica"), zona);
}

pub fn zona_barra_tastiera(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp(egui::Id::unique("zona_barra_tastiera")))
}

pub fn imposta_zona_barra_tastiera(ctx: &egui::Context, zona: Option<egui::Rect>) {
    imposta_zona(ctx, egui::Id::unique("zona_barra_tastiera"), zona);
}

fn imposta_zona(ctx: &egui::Context, chiave: egui::Id, zona: Option<egui::Rect>) {
    ctx.data_mut(|d| match zona {
        Some(z) => {
            d.insert_temp(chiave, z);
        }
        None => {
            d.remove::<egui::Rect>(chiave);
        }
    });
}

/// Telefono: l'area su cui si allineano i pulsanti fluttuanti ("+", la
/// maniglia, ☐), cosi' stanno alla stessa altezza e alla stessa distanza dai
/// bordi. La registra `principale::telefono` a ogni fotogramma.
pub fn area_pulsanti(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp(egui::Id::unique("area_pulsanti")))
}

pub fn imposta_area_pulsanti(ctx: &egui::Context, area: Option<egui::Rect>) {
    imposta_zona(ctx, egui::Id::unique("area_pulsanti"), area);
}

/// Telefono: la testata della panoramica; trascinata verso il basso la chiude.
pub fn zona_chiudi_panoramica(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.data(|d| d.get_temp(egui::Id::unique("zona_chiudi_panoramica")))
}

pub fn imposta_zona_chiudi_panoramica(ctx: &egui::Context, zona: Option<egui::Rect>) {
    imposta_zona(ctx, egui::Id::unique("zona_chiudi_panoramica"), zona);
}

/// Telefono: quanto e' aperta la panoramica mentre il dito la trascina (0
/// chiusa, 1 aperta del tutto). `None` quando nessun dito la trascina: allora
/// `principale::telefono` la porta da sola a 0 o a 1 con un'animazione.
pub fn trascina_panoramica(ctx: &egui::Context) -> Option<f32> {
    ctx.data(|d| d.get_temp(egui::Id::unique("trascina_panoramica")))
}

pub fn imposta_trascina_panoramica(ctx: &egui::Context, progresso: Option<f32>) {
    ctx.data_mut(|d| match progresso {
        Some(p) => {
            d.insert_temp(egui::Id::unique("trascina_panoramica"), p);
        }
        None => {
            d.remove::<f32>(egui::Id::unique("trascina_panoramica"));
        }
    });
}

/// Telefono: dopo aver sollevato il dito sfogliando, dove deve arrivare la
/// nota (spostamento) e verso quale nota si va (+1 la successiva, -1 la
/// precedente, 0 si resta). Lo usa `editor::mostra` per l'animazione.
pub fn conclusione_sfoglia(ctx: &egui::Context) -> Option<(f32, i8)> {
    ctx.data(|d| d.get_temp(egui::Id::unique("conclusione_sfoglia")))
}

pub fn imposta_conclusione_sfoglia(ctx: &egui::Context, conclusione: Option<(f32, i8)>) {
    ctx.data_mut(|d| match conclusione {
        Some(c) => {
            d.insert_temp(egui::Id::unique("conclusione_sfoglia"), c);
        }
        None => {
            d.remove::<(f32, i8)>(egui::Id::unique("conclusione_sfoglia"));
        }
    });
}
