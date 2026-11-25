//! Lo scambio delle note tra due istanze, via TCP.
//!
//! ```text
//! chi chiama                         chi risponde
//!   Offerta(timbri) ───────────────▶
//!                   ◀─────────────── Risposta(note piu' recenti, richieste)
//!   Invio(note richieste) ─────────▶
//! ```
//!
//! Alla fine entrambi hanno tutte le versioni necessarie; i conflitti
//! vengono risolti da ciascun lato con le stesse regole (`fusione`).

use std::collections::HashSet;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::fusione;
use super::protocollo::{Messaggio, VERSIONE_PROTOCOLLO, leggi_messaggio, scrivi_messaggio};
use super::servizio::{Condiviso, Evento};
use crate::modello::Nota;

const TEMPO_COLLEGAMENTO: Duration = Duration::from_secs(3);
const TEMPO_LETTURA: Duration = Duration::from_secs(15);

pub(crate) fn ciclo_server(ascolto: TcpListener, c: Arc<Condiviso>) {
    while !c.fermato() {
        match ascolto.accept() {
            Ok((flusso, mittente)) => {
                let c = c.clone();
                let _ = thread::Builder::new()
                    .name("rete-richiesta".into())
                    .spawn(move || {
                        // Gli errori (codice diverso, rete caduta) non riguardano l'utente:
                        // la richiesta viene semplicemente ignorata.
                        let _ = rispondi(flusso, mittente, &c);
                    });
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(150)),
            Err(_) => thread::sleep(Duration::from_millis(500)),
        }
    }
}

fn prepara(flusso: &TcpStream) -> Result<(), String> {
    flusso.set_nonblocking(false).map_err(|e| e.to_string())?;
    flusso.set_read_timeout(Some(TEMPO_LETTURA)).map_err(|e| e.to_string())?;
    flusso.set_write_timeout(Some(TEMPO_LETTURA)).map_err(|e| e.to_string())?;
    let _ = flusso.set_nodelay(true);
    Ok(())
}

fn rispondi(mut flusso: TcpStream, mittente: SocketAddr, c: &Condiviso) -> Result<(), String> {
    prepara(&flusso)?;
    let chiave = c.chiave();
    let (versione, id_dispositivo, nome, porta, timbri) = match leggi_messaggio(&mut flusso, chiave)? {
        Messaggio::Offerta { versione, id_dispositivo, nome, porta, timbri } => (versione, id_dispositivo, nome, porta, timbri),
        // Un altro computer chiede il nostro ordine delle schede: si risponde e basta.
        Messaggio::RichiestaOrdine { id_dispositivo } if id_dispositivo != c.id_dispositivo => {
            let ordine = c.ordine.lock().unwrap().clone();
            return scrivi_messaggio(&mut flusso, chiave, &Messaggio::Ordine { ordine }).map_err(|e| e.to_string());
        }
        _ => return Err("sequenza inattesa".into()),
    };
    if versione != VERSIONE_PROTOCOLLO || id_dispositivo == c.id_dispositivo {
        return Err("versione diversa o richiesta da se stessi".into());
    }

    let locali = c.istantanea.lock().unwrap().clone();
    let (note, richieste) = fusione::confronta(&locali, &timbri);
    scrivi_messaggio(
        &mut flusso,
        chiave,
        &Messaggio::Risposta {
            id_dispositivo: c.id_dispositivo.clone(),
            nome: c.nome.clone(),
            note,
            richieste: richieste.clone(),
        },
    )
    .map_err(|e| e.to_string())?;

    let Messaggio::Invio { note } = leggi_messaggio(&mut flusso, chiave)? else {
        return Err("sequenza inattesa".into());
    };
    let richieste: HashSet<String> = richieste.into_iter().collect();
    let ricevute: Vec<Nota> = note.into_iter().filter(|n| richieste.contains(&n.id)).collect();

    // Chi ci ha chiamato e' raggiungibile: lo aggiungiamo all'elenco, cosi' la
    // sincronizzazione funziona anche se la scoperta va in una sola direzione.
    c.registra_dispositivo(&id_dispositivo, &nome, SocketAddr::new(mittente.ip(), porta));
    c.segna_esito(&id_dispositivo, true);
    if !ricevute.is_empty() {
        c.invia_evento(Evento::NoteRicevute { note: ricevute, da: nome });
    }
    Ok(())
}

pub(crate) struct Esito {
    pub id_dispositivo: String,
    pub nome: String,
    pub ricevute: Vec<Nota>,
}

pub(crate) fn sincronizza_con(indirizzo: SocketAddr, c: &Condiviso) -> Result<Esito, String> {
    let chiave = c.chiave();
    let mut flusso = TcpStream::connect_timeout(&indirizzo, TEMPO_COLLEGAMENTO).map_err(|e| e.to_string())?;
    prepara(&flusso)?;
    let locali = c.istantanea.lock().unwrap().clone();
    scrivi_messaggio(
        &mut flusso,
        chiave,
        &Messaggio::Offerta {
            versione: VERSIONE_PROTOCOLLO,
            id_dispositivo: c.id_dispositivo.clone(),
            nome: c.nome.clone(),
            porta: c.porta,
            timbri: fusione::timbri(&locali),
        },
    )
    .map_err(|e| e.to_string())?;

    let Messaggio::Risposta { id_dispositivo, nome, note, richieste } = leggi_messaggio(&mut flusso, chiave)?
    else {
        return Err("sequenza inattesa".into());
    };
    if id_dispositivo == c.id_dispositivo {
        return Err("l’indirizzo è questo stesso dispositivo".into());
    }
    let richieste: HashSet<String> = richieste.into_iter().collect();
    let da_inviare: Vec<Nota> = locali.into_iter().filter(|n| richieste.contains(&n.id)).collect();
    scrivi_messaggio(&mut flusso, chiave, &Messaggio::Invio { note: da_inviare }).map_err(|e| e.to_string())?;

    Ok(Esito { id_dispositivo, nome, ricevute: note })
}

/// Perche' non e' stato possibile avere l'ordine delle schede.
#[derive(Debug, PartialEq)]
pub enum ErroreOrdine {
    /// Il collegamento non e' riuscito.
    NonRaggiungibile,
    /// Collegato, ma l'altro computer non ha risposto come previsto: usa una
    /// versione precedente di Schedario.
    VersionePrecedente,
}

/// Chiede l'ordine delle schede al computer all'indirizzo indicato.
pub(crate) fn chiedi_ordine(indirizzo: SocketAddr, c: &Condiviso) -> Result<Vec<String>, ErroreOrdine> {
    let chiave = c.chiave();
    let mut flusso =
        TcpStream::connect_timeout(&indirizzo, TEMPO_COLLEGAMENTO).map_err(|_| ErroreOrdine::NonRaggiungibile)?;
    prepara(&flusso).map_err(|_| ErroreOrdine::NonRaggiungibile)?;
    scrivi_messaggio(&mut flusso, chiave, &Messaggio::RichiestaOrdine { id_dispositivo: c.id_dispositivo.clone() })
        .map_err(|_| ErroreOrdine::NonRaggiungibile)?;
    match leggi_messaggio(&mut flusso, chiave) {
        Ok(Messaggio::Ordine { ordine }) => Ok(ordine),
        _ => Err(ErroreOrdine::VersionePrecedente),
    }
}
