//! La nota: titolo, testo e metadati necessari a salvarla e sincronizzarla.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::tempo;

/// Quante modifiche ha fatto ciascun computer a una nota ("vettore di versioni").
/// Permette di capire se una versione deriva dall'altra o se sono nate in
/// parallelo su due computer diversi (conflitto).
pub type Versioni = BTreeMap<String, u64>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nota {
    /// Identificativo stabile (32 cifre esadecimali). Non cambia mai,
    /// nemmeno rinominando la nota: da' il nome al file di testo.
    pub id: String,
    pub titolo: String,
    pub testo: String,
    /// Istante di creazione, in millisecondi. Ordina le schede.
    pub creata: i64,
    /// Istante dell'ultima modifica, in millisecondi.
    pub modificata: i64,
    /// Dispositivo che ha fatto l'ultima modifica.
    #[serde(default)]
    pub origine: String,
    /// `true` se la nota e' nel cestino.
    #[serde(default)]
    pub nel_cestino: bool,
    /// `true` se la nota e' stata eliminata definitivamente. Resta solo questo
    /// segnaposto vuoto, perche' gli altri computer sappiano di eliminarla.
    #[serde(default)]
    pub eliminata: bool,
    /// Se la nota e' la copia di una versione in conflitto: id dell'originale.
    #[serde(default)]
    pub conflitto_di: Option<String>,
    #[serde(default)]
    pub versioni: Versioni,
    /// Colore della nota (uno di `COLORI`), vuoto se non ne ha. E' un testo e
    /// non un elenco chiuso: un colore sconosciuto, arrivato da una versione
    /// futura del programma, viene semplicemente mostrato come "nessuno".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub colore: String,
    /// Le note fissate stanno prima delle altre.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fissata: bool,
}

/// Colori che si possono dare a una nota, nell'ordine in cui sono proposti.
pub const COLORI: [&str; 6] = ["giallo", "arancio", "rosa", "viola", "azzurro", "verde"];

/// `true` se `colore` e' vuoto (nessun colore) o e' uno di `COLORI`.
pub fn colore_valido(colore: &str) -> bool {
    colore.is_empty() || COLORI.contains(&colore)
}

/// Rapporto tra due versioni della stessa nota.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relazione {
    /// Sono la stessa versione.
    Uguale,
    /// La prima e' piu' vecchia: la seconda contiene tutte le sue modifiche.
    Precede,
    /// La prima e' piu' recente e contiene tutte le modifiche della seconda.
    Segue,
    /// Nate in parallelo su computer diversi: e' un conflitto.
    Concorrente,
}

/// Confronta due vettori di versioni. A parita' di vettore (per esempio note
/// create prima dell'introduzione dei vettori) decide l'istante di modifica.
pub fn relazione(a: (&Versioni, i64, &str), b: (&Versioni, i64, &str)) -> Relazione {
    let (mut minore, mut maggiore) = (false, false);
    for chiave in a.0.keys().chain(b.0.keys()) {
        let (x, y) = (a.0.get(chiave).copied().unwrap_or(0), b.0.get(chiave).copied().unwrap_or(0));
        minore |= x < y;
        maggiore |= x > y;
    }
    match (minore, maggiore) {
        (true, true) => Relazione::Concorrente,
        (true, false) => Relazione::Precede,
        (false, true) => Relazione::Segue,
        (false, false) => match (a.1, a.2).cmp(&(b.1, b.2)) {
            Ordering::Equal => Relazione::Uguale,
            Ordering::Less => Relazione::Precede,
            Ordering::Greater => Relazione::Segue,
        },
    }
}

impl Nota {
    /// Crea una nota vuota, modificata ora dal dispositivo indicato.
    pub fn nuova(dispositivo: &str) -> Self {
        let ora = tempo::adesso_ms();
        Self {
            id: nuovo_id(),
            titolo: String::new(),
            testo: String::new(),
            creata: ora,
            modificata: ora,
            origine: dispositivo.to_owned(),
            nel_cestino: false,
            eliminata: false,
            conflitto_di: None,
            versioni: Versioni::from([(dispositivo.to_owned(), 1)]),
            colore: String::new(),
            fissata: false,
        }
    }

    /// Registra una modifica locale: aggiorna istante, dispositivo e versione.
    pub fn segna_modificata(&mut self, dispositivo: &str) {
        self.modificata = tempo::successivo(self.modificata);
        self.origine = dispositivo.to_owned();
        *self.versioni.entry(dispositivo.to_owned()).or_insert(0) += 1;
    }

    /// Una nota e' visibile se non e' stata eliminata definitivamente.
    pub fn visibile(&self) -> bool {
        !self.eliminata
    }

    /// Titolo da mostrare: quello scritto, oppure la prima riga del testo,
    /// oppure il segnaposto indicato (per esempio "Senza titolo").
    pub fn titolo_visibile<'a>(&'a self, segnaposto: &'a str) -> &'a str {
        let titolo = self.titolo.trim();
        if !titolo.is_empty() {
            return titolo;
        }
        // Una voce di lista nella prima riga: "[ ] Pane" diventa "Pane".
        self.testo
            .lines()
            .map(|riga| {
                let riga = riga.trim();
                ["[ ] ", "[x] ", "[X] "].iter().find_map(|segno| riga.strip_prefix(segno)).unwrap_or(riga).trim()
            })
            .find(|riga| !riga.is_empty() && !matches!(*riga, "[ ]" | "[x]" | "[X]"))
            .unwrap_or(segnaposto)
    }

    /// Parole del testo. I segni delle liste (`[ ]`, `[x]`) e i trattini
    /// isolati non contano: spuntare una voce non cambia il conteggio.
    pub fn conta_parole(&self) -> usize {
        self.testo
            .split_whitespace()
            .filter(|parola| !matches!(*parola, "[" | "]" | "[x]" | "[X]" | "-" | "*"))
            .count()
    }

    /// Ricerca senza distinzione tra maiuscole e minuscole in titolo e testo.
    pub fn corrisponde(&self, ricerca: &str) -> bool {
        let ricerca = ricerca.trim().to_lowercase();
        ricerca.is_empty()
            || self.titolo.to_lowercase().contains(&ricerca)
            || self.testo.to_lowercase().contains(&ricerca)
    }

    /// Dove compare la ricerca nel testo, senza distinguere maiuscole e
    /// minuscole: intervalli di byte, per evidenziare le parole trovate.
    pub fn occorrenze(testo: &str, ricerca: &str) -> Vec<std::ops::Range<usize>> {
        let cercato: Vec<char> = ricerca.trim().to_lowercase().chars().collect();
        if cercato.is_empty() {
            return Vec::new();
        }
        // Ogni carattere del testo con la sua forma minuscola (che puo' essere
        // piu' lunga di un carattere, per esempio "İ").
        let caratteri: Vec<(usize, Vec<char>)> = testo.char_indices().map(|(b, c)| (b, c.to_lowercase().collect())).collect();
        let mut trovate = Vec::new();
        let mut inizio = 0;
        while inizio < caratteri.len() {
            let mut posizione = 0;
            let mut fine = inizio;
            while posizione < cercato.len() && fine < caratteri.len() {
                let minuscolo = &caratteri[fine].1;
                if cercato[posizione..].starts_with(minuscolo) {
                    posizione += minuscolo.len();
                    fine += 1;
                } else {
                    break;
                }
            }
            if posizione == cercato.len() {
                let da = caratteri[inizio].0;
                let a = caratteri.get(fine).map_or(testo.len(), |c| c.0);
                trovate.push(da..a);
                inizio = fine;
            } else {
                inizio += 1;
            }
        }
        trovate
    }

    /// La prima riga del testo in cui compare la ricerca, accorciata attorno
    /// alla parola trovata (circa `lunghezza` caratteri), con le parti da
    /// evidenziare. Serve all'elenco delle note trovate sul telefono: cercando
    /// "pane" in una lista lunga si legge "… comprare il pane e il latte …".
    /// I segni delle liste (`[ ]`, `[x]`) all'inizio della riga si tolgono.
    pub fn estratto(testo: &str, ricerca: &str, lunghezza: usize) -> Option<(String, Vec<std::ops::Range<usize>>)> {
        let riga = testo.lines().find(|riga| !Self::occorrenze(riga, ricerca).is_empty())?;
        let riga = riga.trim();
        let riga = ["[ ] ", "[x] ", "[X] "].iter().find_map(|segno| riga.strip_prefix(segno)).unwrap_or(riga).trim();
        let caratteri: Vec<char> = riga.chars().collect();
        let primo = Self::occorrenze(riga, ricerca).first()?.start;
        let posizione = riga[..primo].chars().count();
        // Un terzo dello spazio prima della parola, il resto dopo.
        let inizio = posizione.saturating_sub(lunghezza / 3).min(caratteri.len().saturating_sub(lunghezza));
        let fine = (inizio + lunghezza).min(caratteri.len());
        let mut estratto: String = caratteri[inizio..fine].iter().collect();
        if inizio > 0 {
            estratto.insert_str(0, "… ");
        }
        if fine < caratteri.len() {
            estratto.push_str(" …");
        }
        let parti = Self::occorrenze(&estratto, ricerca);
        Some((estratto, parti))
    }

    /// Rapporto tra questa versione e `altra` (stessa nota).
    pub fn relazione_con(&self, altra: &Nota) -> Relazione {
        relazione(
            (&self.versioni, self.modificata, &self.origine),
            (&altra.versioni, altra.modificata, &altra.origine),
        )
    }

    /// `true` se questa versione e' stata salvata dopo `altra` (solo l'orario).
    /// A parita' vince il dispositivo con identificativo maggiore, cosi'
    /// tutti i computer arrivano alla stessa scelta.
    pub fn piu_recente_di(&self, altra: &Nota) -> bool {
        (self.modificata, self.origine.as_str()) > (altra.modificata, altra.origine.as_str())
    }

    /// `true` se titolo e testo coincidono.
    pub fn stesso_testo(&self, altra: &Nota) -> bool {
        self.titolo == altra.titolo && self.testo == altra.testo
    }

    /// Trasforma la nota in un segnaposto di eliminazione definitiva.
    pub fn elimina(&mut self, dispositivo: &str) {
        self.titolo.clear();
        self.testo.clear();
        self.nel_cestino = true;
        self.eliminata = true;
        self.conflitto_di = None;
        self.colore.clear();
        self.fissata = false;
        self.segna_modificata(dispositivo);
    }

    /// Copia di questa versione da conservare dopo un conflitto.
    ///
    /// L'identificativo deriva dal contenuto della versione: due computer che
    /// risolvono lo stesso conflitto creano la stessa identica copia, invece di
    /// due copie diverse.
    pub fn copia_in_conflitto(&self) -> Nota {
        let mut impronta = Sha256::new();
        impronta.update(b"conflitto\0");
        impronta.update(self.id.as_bytes());
        for (dispositivo, numero) in &self.versioni {
            impronta.update(dispositivo.as_bytes());
            impronta.update(numero.to_be_bytes());
        }
        impronta.update(self.modificata.to_be_bytes());
        let id = impronta.finalize().iter().take(16).map(|b| format!("{b:02x}")).collect();
        Nota {
            id,
            nel_cestino: false,
            eliminata: false,
            conflitto_di: Some(self.conflitto_di.clone().unwrap_or_else(|| self.id.clone())),
            ..self.clone()
        }
    }
}

/// Unisce due vettori prendendo, per ogni computer, il numero piu' alto.
pub fn unisci_versioni(a: &Versioni, b: &Versioni) -> Versioni {
    let mut unite = a.clone();
    for (dispositivo, numero) in b {
        let voce = unite.entry(dispositivo.clone()).or_insert(0);
        *voce = (*voce).max(*numero);
    }
    unite
}

/// Identificativo casuale di 32 cifre esadecimali.
pub fn nuovo_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

/// Un identificativo valido e' fatto solo di 32 cifre esadecimali minuscole.
/// Protegge i nomi dei file da caratteri pericolosi arrivati da backup o rete.
pub fn id_valido(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn occorrenze_senza_maiuscole() {
        let testo = "Dentista lunedì, poi DENTISTA di nuovo";
        let trovate = Nota::occorrenze(testo, " dentista ");
        assert_eq!(trovate.len(), 2);
        assert_eq!(&testo[trovate[0].clone()], "Dentista");
        assert_eq!(&testo[trovate[1].clone()], "DENTISTA");
        // Lettere accentate: gli intervalli restano su confini di carattere.
        let trovate = Nota::occorrenze(testo, "LUNEDÌ");
        assert_eq!(&testo[trovate[0].clone()], "lunedì");
        assert!(Nota::occorrenze(testo, "").is_empty());
        assert!(Nota::occorrenze(testo, "gatto").is_empty());
        // Le occorrenze non si sovrappongono.
        assert_eq!(Nota::occorrenze("aaaa", "aa").len(), 2);
    }

    #[test]
    fn estratto_attorno_alla_parola() {
        let testo = "Spesa\n[ ] Pane\nRicordarsi di comprare il pane e il latte prima di sera, poi passare in farmacia";
        // La prima riga che contiene la parola, senza il segno della lista.
        let (estratto, parti) = Nota::estratto(testo, "pane", 40).unwrap();
        assert_eq!(estratto, "Pane");
        assert_eq!(&estratto[parti[0].clone()], "Pane");
        // Riga lunga: accorciata attorno alla parola, con i puntini.
        let (estratto, parti) = Nota::estratto(testo, "latte", 20).unwrap();
        assert!(estratto.starts_with("… ") && estratto.ends_with(" …"), "{estratto}");
        assert_eq!(&estratto[parti[0].clone()], "latte");
        // Solo nel titolo, o da nessuna parte: nessun estratto.
        assert!(Nota::estratto(testo, "gatto", 40).is_none());
    }

    #[test]
    fn titolo_visibile_usa_la_prima_riga() {
        let mut nota = Nota::nuova("a");
        assert_eq!(nota.titolo_visibile("Senza titolo"), "Senza titolo");
        nota.testo = "\n  spesa  \nlatte".into();
        assert_eq!(nota.titolo_visibile("Senza titolo"), "spesa");
        // Il segno di una voce di lista non fa parte del titolo.
        nota.testo = "[ ] Pane\n[x] Latte".into();
        assert_eq!(nota.titolo_visibile("Senza titolo"), "Pane");
        nota.testo = "[ ]\n[x] Latte".into();
        assert_eq!(nota.titolo_visibile("Senza titolo"), "Latte");
        nota.titolo = "Lista".into();
        assert_eq!(nota.titolo_visibile("Senza titolo"), "Lista");
    }

    #[test]
    fn una_modifica_successiva_segue_la_precedente() {
        let originale = Nota::nuova("a");
        let mut dopo = originale.clone();
        dopo.segna_modificata("b");
        assert_eq!(dopo.relazione_con(&originale), Relazione::Segue);
        assert_eq!(originale.relazione_con(&dopo), Relazione::Precede);
        assert_eq!(dopo.relazione_con(&dopo.clone()), Relazione::Uguale);
    }

    #[test]
    fn due_modifiche_in_parallelo_sono_concorrenti() {
        let originale = Nota::nuova("a");
        let mut su_a = originale.clone();
        let mut su_b = originale.clone();
        su_a.segna_modificata("a");
        su_b.segna_modificata("b");
        assert_eq!(su_a.relazione_con(&su_b), Relazione::Concorrente);
    }

    #[test]
    fn senza_vettori_decide_l_orario() {
        let mut a = Nota::nuova("aaa");
        a.versioni.clear();
        let mut b = a.clone();
        b.origine = "bbb".into();
        assert_eq!(b.relazione_con(&a), Relazione::Segue);
        assert!(b.piu_recente_di(&a));
    }

    #[test]
    fn la_copia_in_conflitto_e_deterministica() {
        let mut nota = Nota::nuova("a");
        nota.testo = "versione persa".into();
        let (c1, c2) = (nota.copia_in_conflitto(), nota.copia_in_conflitto());
        assert_eq!(c1, c2);
        assert!(id_valido(&c1.id));
        assert_ne!(c1.id, nota.id);
        assert_eq!(c1.conflitto_di.as_deref(), Some(nota.id.as_str()));
    }

    #[test]
    fn il_colore_e_facoltativo_e_tollerante() {
        // Una nota salvata prima dei colori si legge ancora, senza colore.
        let mut vecchia = serde_json::to_value(Nota::nuova("a")).unwrap();
        assert!(vecchia.get("colore").is_none(), "senza colore il campo non viene scritto");
        vecchia.as_object_mut().unwrap().remove("colore");
        let letta: Nota = serde_json::from_value(vecchia).unwrap();
        assert_eq!(letta.colore, "");

        let mut nota = Nota::nuova("a");
        nota.colore = "verde".into();
        let riletta: Nota = serde_json::from_str(&serde_json::to_string(&nota).unwrap()).unwrap();
        assert_eq!(riletta.colore, "verde");
        assert!(colore_valido("verde") && colore_valido(""));
        assert!(!colore_valido("fucsia-futuro"));

        nota.elimina("a");
        assert_eq!(nota.colore, "", "un segnaposto di eliminazione non porta dati");
    }

    #[test]
    fn i_segni_delle_liste_non_sono_parole() {
        let mut nota = Nota::nuova("a");
        nota.testo = "Spesa:\n[ ] latte\n- [x] pane fresco".into();
        assert_eq!(nota.conta_parole(), 4);
        nota.testo = nota.testo.replace("[ ]", "[x]");
        assert_eq!(nota.conta_parole(), 4);
    }

    #[test]
    fn id_generati_sono_validi() {
        assert!(id_valido(&nuovo_id()));
        assert!(!id_valido("../../etc/passwd"));
    }
}
