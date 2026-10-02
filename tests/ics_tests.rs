//! Calendário ICS da agenda.
//!
//! O webapp chama `GET ICS/GetLocalizador`, que devolve só o localizador do
//! feed como string JSON; o link do feed é `<origem>/api/ics/getics?id=<loc>`
//! e abre sem autenticação. O endpoint antigo do SDK
//! (`localizadorICS/GetLocalizadorICS`) responde 404 na MedX real.
//!
//! Sem rede externa: um servidor HTTP local responde as rotas do teste.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use medx::{MedxClient, Session};

struct Mock {
    url: String,
    paths: Arc<Mutex<Vec<String>>>,
}

impl Mock {
    /// Responde 200 com `body` em `/api/ICS/GetLocalizador` e 404 no resto.
    fn start(body: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let paths = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&paths);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    continue;
                }
                let mut line = String::new();
                while reader.read_line(&mut line).map(|n| n > 2).unwrap_or(false) {
                    line.clear();
                }
                let path = request_line.split_whitespace().nth(1).unwrap_or("").to_string();
                seen.lock().unwrap().push(path.clone());
                let (status, body) = if path == "/api/ICS/GetLocalizador" {
                    ("200 OK", body)
                } else {
                    ("404 Not Found", "")
                };
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                );
            }
        });
        Mock { url, paths }
    }

    fn client(&self) -> MedxClient {
        let session = Session {
            token: "tok".to_string(),
            email: "ics@example.invalid".to_string(),
            db_id: "1".to_string(),
            host: self.url.clone(),
        };
        MedxClient::from_session(session)
    }
}

#[test]
fn ics_locator_le_o_localizador_do_webapp() {
    let mock = Mock::start("\"ABCDEFGHIJKLMNO\"");

    let locator = mock.client().ics_locator().expect("localizador");

    assert_eq!(locator, "ABCDEFGHIJKLMNO");
    assert_eq!(mock.paths.lock().unwrap().as_slice(), ["/api/ICS/GetLocalizador"]);
}

#[test]
fn ics_config_monta_o_link_do_feed() {
    let mock = Mock::start("\"ABCDEFGHIJKLMNO\"");

    let cfg = mock.client().ics_config().expect("configuração do ICS");

    assert_eq!(cfg.url, format!("{}/api/ics/getics?id=ABCDEFGHIJKLMNO", mock.url));
    assert_eq!(cfg.token, "ABCDEFGHIJKLMNO");
    assert!(cfg.active());
}

#[test]
fn sem_localizador_o_feed_fica_inativo() {
    for body in ["\"\"", "null", ""] {
        let mock = Mock::start(body);

        let cfg = mock.client().ics_config().expect("configuração do ICS");

        assert_eq!(cfg.url, "", "corpo {body:?}");
        assert_eq!(cfg.token, "", "corpo {body:?}");
        assert!(!cfg.active(), "corpo {body:?}");
    }
}
