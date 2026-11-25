//! Scoperta automatica dei computer con lo stesso codice di gruppo.
//!
//! Ogni 3 secondi un breve pacchetto UDP viene inviato in broadcast sulla
//! rete locale. Il contenuto e' cifrato: chi non conosce il codice vede
//! solo un'etichetta casuale, non il nome del computer.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, Socket, Type};

use super::protocollo::{Annuncio, VERSIONE_PROTOCOLLO, codifica_annuncio, decodifica_annuncio};
use super::servizio::{Condiviso, Evento};

const INTERVALLO_ANNUNCI: Duration = Duration::from_secs(3);

pub(crate) fn ciclo(c: Arc<Condiviso>) {
    let socket = match crea_socket(c.porta_scoperta) {
        Ok(socket) => socket,
        Err(errore) => {
            c.invia_evento(Evento::Errore(format!(
                "scoperta automatica non disponibile ({errore}); è possibile aggiungere gli indirizzi manualmente"
            )));
            return;
        }
    };
    let chiave = c.chiave();
    let annuncio = codifica_annuncio(
        chiave,
        &Annuncio {
            versione: VERSIONE_PROTOCOLLO,
            id_dispositivo: c.id_dispositivo.clone(),
            nome: c.nome.clone(),
            porta: c.porta,
        },
    );
    let destinazioni = destinazioni_broadcast(c.porta_scoperta);
    let mut ultimo_annuncio: Option<Instant> = None;
    let mut buffer = [0u8; 4096];

    while !c.fermato() {
        if ultimo_annuncio.is_none_or(|t| t.elapsed() >= INTERVALLO_ANNUNCI) {
            for destinazione in &destinazioni {
                let _ = socket.send_to(&annuncio, destinazione);
            }
            ultimo_annuncio = Some(Instant::now());
        }
        // Il timeout di lettura (500 ms) scandisce il ciclo.
        if let Ok((letti, mittente)) = socket.recv_from(&mut buffer) {
            let Some(arrivato) = decodifica_annuncio(chiave, &buffer[..letti]) else { continue };
            if arrivato.id_dispositivo == c.id_dispositivo {
                continue;
            }
            let indirizzo = SocketAddr::new(mittente.ip(), arrivato.porta);
            if c.registra_dispositivo(&arrivato.id_dispositivo, &arrivato.nome, indirizzo) {
                c.richiedi_sincronizzazione();
            }
        }
    }
}

fn crea_socket(porta: u16) -> std::io::Result<UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    // Permette a piu' istanze sullo stesso computer di ricevere gli annunci.
    socket.set_reuse_address(true)?;
    socket.set_broadcast(true)?;
    socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, porta)).into())?;
    let socket: UdpSocket = socket.into();
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;
    Ok(socket)
}

/// Broadcast generale piu' quello della sottorete principale (ipotizzata /24),
/// che alcuni sistemi inoltrano in modo piu' affidabile.
fn destinazioni_broadcast(porta: u16) -> Vec<SocketAddr> {
    let mut elenco = vec![SocketAddr::from((Ipv4Addr::BROADCAST, porta))];
    if let Some(IpAddr::V4(locale)) = indirizzo_locale() {
        let [a, b, c, _] = locale.octets();
        elenco.push(SocketAddr::from((Ipv4Addr::new(a, b, c, 255), porta)));
    }
    elenco
}

/// Indirizzo IP con cui questo computer esce in rete. "Collegare" un socket
/// UDP non invia alcun pacchetto: serve solo a far scegliere l'interfaccia.
pub fn indirizzo_locale() -> Option<IpAddr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect((Ipv4Addr::new(192, 168, 255, 255), 9)).ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_unspecified()).then_some(ip)
}
