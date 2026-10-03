//! Codificação dos valores de texto na query string.
//!
//! Todo valor de texto que vai para a query passa por
//! `util::encode_query_value` (codificação de formulário: espaço vira `+`,
//! só `A-Z a-z 0-9 - _ . ~` ficam como estão, o resto vira `%XX` dos bytes
//! UTF-8). Sem isso, um `&` no nome de um paciente cortava a busca, um `#`
//! virava fragmento e um `+` num e-mail (`fulano+clinica@...`) chegava ao
//! servidor como espaço.
//!
//! Sem rede externa: um servidor HTTP local registra o caminho de cada
//! request; o teste decodifica a query e exige o valor original de volta.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use medx::util::encode_query_value;
use medx::{MedxClient, Session};

/// Valor com tudo o que quebrava a query: espaço, `&`, `#`, `?`, `=`, `+`,
/// `/`, `%` e acento.
const NASTY: &str = "ANA & FILHA #1 ?x=y+z/50% é";

/// Chave pública RSA de 1024 bits gerada só para os testes (o mock não
/// decifra nada; a chave só precisa ser válida para o SDK cifrar).
const PUBLIC_KEY_XML: &str = "<RSAKeyValue><Modulus>nQmVeUUOCy8WOa2rTuRl27GEYcBFLroA6W2BVHBmnlDrRemIQMCoN3pOHUiZZmo+3AZUFjjQj6yms4907FsJjVVcpX3i3Y1vpwVee68xzQk+pfQGSnbOBQksovif32EAMADRRkblFSnvN1pmCLxJUekjxrQX1vVFf1/+PzhHThM=</Modulus><Exponent>AQAB</Exponent></RSAKeyValue>";

// ── encode_query_value ───────────────────────────────────────────────────────

#[test]
fn espaco_vira_mais() {
    assert_eq!(encode_query_value("ANA SOUZA"), "ANA+SOUZA");
}

#[test]
fn caracteres_reservados_viram_percentual() {
    assert_eq!(encode_query_value("a+b@x.com"), "a%2Bb%40x.com");
    assert_eq!(encode_query_value("&#?=/%"), "%26%23%3F%3D%2F%25");
}

#[test]
fn acento_vira_bytes_utf8() {
    assert_eq!(encode_query_value("JOÃO"), "JO%C3%83O");
}

#[test]
fn nao_reservados_ficam_como_estao() {
    assert_eq!(encode_query_value("abc-_.~XYZ019"), "abc-_.~XYZ019");
    assert_eq!(encode_query_value(""), "");
}

// ── Servidor local ────────────────────────────────────────────────────────────

struct Mock {
    url: String,
    paths: Arc<Mutex<Vec<String>>>,
}

impl Mock {
    /// Responde `routes[caminho sem query]` (status, corpo); o resto, 200 `[]`.
    fn start(routes: &[(&'static str, u16, &'static str)]) -> Self {
        let routes: HashMap<&'static str, (u16, &'static str)> = routes
            .iter()
            .map(|(path, status, body)| (*path, (*status, *body)))
            .collect();
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
                let mut length = 0usize;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) <= 2 {
                        break;
                    }
                    if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        length = v.trim().parse().unwrap_or(0);
                    }
                }
                let mut body = vec![0u8; length];
                let _ = reader.read_exact(&mut body);
                let path = request_line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_string();
                let route = path.split('?').next().unwrap_or("").to_string();
                seen.lock().unwrap().push(path);
                let (status, body) = routes.get(route.as_str()).copied().unwrap_or((200, "[]"));
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                );
            }
        });
        Mock { url, paths }
    }

    fn client(&self) -> MedxClient {
        MedxClient::from_session(Session {
            token: "tok".to_string(),
            email: "u@example.invalid".to_string(),
            db_id: "1".to_string(),
            host: self.url.clone(),
        })
    }

    /// Parâmetros decodificados da última request a `route`.
    fn query(&self, route: &str) -> HashMap<String, String> {
        let paths = self.paths.lock().unwrap();
        let path = paths
            .iter()
            .rev()
            .find(|p| p.split('?').next() == Some(route))
            .unwrap_or_else(|| panic!("nenhuma request a {route}; requests: {paths:?}"));
        let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
        query
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
                (form_decode(k), form_decode(v))
            })
            .collect()
    }
}

/// Decodificação de formulário: `+` vira espaço, `%XX` vira o byte.
fn form_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap();
                out.push(u8::from_str_radix(hex, 16).expect("percentual válido"));
                i += 2;
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8(out).expect("UTF-8 válido")
}

fn keys(query: &HashMap<String, String>) -> Vec<&str> {
    let mut keys: Vec<&str> = query.keys().map(String::as_str).collect();
    keys.sort_unstable();
    keys
}

// ── Cada método que leva texto na query ───────────────────────────────────────

#[test]
fn busca_de_pacientes() {
    let mock = Mock::start(&[]);
    mock.client()
        .search_contacts(NASTY, medx::ContactSearchGroup::All, 1)
        .unwrap();
    let q = mock.query("/api/contatos/GetContatosGridBySearch");
    assert_eq!(keys(&q), ["Group", "GroupValue", "Name"]);
    assert_eq!(q["Name"], NASTY);
}

#[test]
fn pacientes_homonimos() {
    let mock = Mock::start(&[]);
    mock.client()
        .homonym_contacts(NASTY, "F&x", "1990-01-01#z")
        .unwrap();
    let q = mock.query("/api/contatos/GetContatosHomonimos");
    assert_eq!(keys(&q), ["Birth", "Gender", "Name"]);
    assert_eq!(q["Name"], NASTY);
    assert_eq!(q["Gender"], "F&x");
    assert_eq!(q["Birth"], "1990-01-01#z");
}

#[test]
fn link_do_arquivo_no_azure() {
    let mock = Mock::start(&[(
        "/api/azure/getfileurl",
        200,
        "\"https://medxdata.blob.core.windows.net/x\"",
    )]);
    let client = mock.client();

    client.azure_file_url(NASTY).unwrap();
    assert_eq!(mock.query("/api/azure/getfileurl")["blobname"], NASTY);

    client.resolve_file_url(NASTY).unwrap();
    assert_eq!(mock.query("/api/azure/getfileurl")["blobname"], NASTY);
}

#[test]
fn lista_de_atendimentos() {
    let mock = Mock::start(&[]);
    mock.client()
        .all_attendances(NASTY, "Pendências & x")
        .unwrap();
    let q = mock.query("/api/atendimentos/GetAllAtendimentos");
    assert_eq!(keys(&q), ["filter", "filterstring"]);
    assert_eq!(q["filter"], NASTY);
    assert_eq!(q["filterstring"], "Pendências & x");
}

#[test]
fn busca_no_prontuario_e_registros_de_modulo() {
    let mock = Mock::start(&[]);
    let client = mock.client();

    client.search_medical_records(900, NASTY).unwrap();
    let q = mock.query("/api/prontuario/GetProntuarioBusca");
    assert_eq!(keys(&q), ["PacId", "busca"]);
    assert_eq!(q["busca"], NASTY);

    client.module_records(900, NASTY).unwrap();
    let q = mock.query("/api/modulos/GetRecords");
    assert_eq!(keys(&q), ["modulo", "pacid"]);
    assert_eq!(q["modulo"], NASTY);
}

#[test]
fn pastas_de_documentos() {
    let mock = Mock::start(&[]);
    mock.client().doc_folders(NASTY).unwrap();
    let q = mock.query("/api/autodocs/getfoldersdocs");
    assert_eq!(keys(&q), ["filter"]);
    assert_eq!(q["filter"], NASTY);
}

#[test]
fn agenda_do_dia() {
    let mock = Mock::start(&[]);
    mock.client().daily_agenda(42, "2026-10-02&Id=1").unwrap();
    let q = mock.query("/api/hoje/GetAgendaDiaUsuario");
    assert_eq!(keys(&q), ["Dt", "Id"]);
    assert_eq!(q["Id"], "42");
    assert_eq!(q["Dt"], "2026-10-02&Id=1");
}

#[test]
fn link_do_feed_ics() {
    let mock = Mock::start(&[("/api/ICS/GetLocalizador", 200, "\"ab+c/d=e\"")]);
    let cfg = mock.client().ics_config().unwrap();
    assert_eq!(
        cfg.url,
        format!("{}/api/ics/getics?id=ab%2Bc%2Fd%3De", mock.url)
    );
    assert_eq!(cfg.token, "ab+c/d=e");
}

// ── Login ─────────────────────────────────────────────────────────────────────

#[test]
fn email_com_mais_no_login() {
    // O 500 encerra o login logo na primeira etapa: só interessa a request.
    let mock = Mock::start(&[("/api/LoginUnificado/VerificaEmailCripto", 500, "")]);
    let _ = medx::auth::login_at(&mock.url, "fulano+clinica@example.invalid", "x");
    let q = mock.query("/api/LoginUnificado/VerificaEmailCripto");
    assert_eq!(keys(&q), ["Email", "dbId"]);
    assert_eq!(q["Email"], "fulano+clinica@example.invalid");
}

#[test]
fn token_antigo_com_mais_e_barra_ao_invalidar_sessao() {
    // Tokens podem ter `+`, `/` e `=`; sem codificar, o `+` chegava como
    // espaço e a sessão antiga não era derrubada.
    let keys_body: &'static str = Box::leak(
        serde_json::json!({ "KeyId": "k1", "PublicKey": PUBLIC_KEY_XML })
            .to_string()
            .into_boxed_str(),
    );
    let mock = Mock::start(&[
        (
            "/api/LoginUnificado/VerificaEmailCripto",
            200,
            "\"Ok:Success:SoftwareId:4242\"",
        ),
        ("/api/security/getkeys", 200, keys_body),
        (
            "/api/LoginUnificado/loginV3",
            400,
            "{\"Message\":\"usuário já logado, dispositivo: web, token  ab+c/d=e\"}",
        ),
        ("/api/security/removetokeninuse", 200, ""),
    ]);
    let dir = std::env::temp_dir().join(format!("medx-urlencode-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // O login repete o loginV3 depois de invalidar e recebe 400 de novo; o
    // resultado não importa, só a request de invalidação.
    std::env::set_var("MEDX_CONFIG_DIR", &dir);
    let _ = medx::auth::login_at(&mock.url, "u@example.invalid", "x");
    std::env::remove_var("MEDX_CONFIG_DIR");
    let _ = std::fs::remove_dir_all(&dir);

    let q = mock.query("/api/security/removetokeninuse");
    assert_eq!(keys(&q), ["token"]);
    assert_eq!(q["token"], "ab+c/d=e");
}
