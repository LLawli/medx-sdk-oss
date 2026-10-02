//! Servidor HTTP fake da MedX, só com `std::net`.
//!
//! Responde rotas registradas pelo teste, com fixtures sintéticas (nunca dado
//! de paciente), e registra cada request para o teste conferir o que o
//! servidor MCP mandou. Cada conexão é atendida numa thread própria, para que
//! chamadas em paralelo cheguem em paralelo e `max_in_flight` signifique algo.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use medx::Session;
use medx_mcp::config::{Config, Credentials};
use serde_json::Value;

/// Chave pública RSA de 1024 bits gerada só para os testes. O fake não
/// decifra a senha: a chave só precisa ser válida para o SDK cifrar.
const PUBLIC_KEY_XML: &str = "<RSAKeyValue><Modulus>nQmVeUUOCy8WOa2rTuRl27GEYcBFLroA6W2BVHBmnlDrRemIQMCoN3pOHUiZZmo+3AZUFjjQj6yms4907FsJjVVcpX3i3Y1vpwVee68xzQk+pfQGSnbOBQksovif32EAMADRRkblFSnvN1pmCLxJUekjxrQX1vVFf1/+PzhHThM=</Modulus><Exponent>AQAB</Exponent></RSAKeyValue>";

/// Token que o login do fake emite.
pub const NEW_TOKEN: &str = "tok_novo";
/// Token da sessão que os testes em processo usam.
pub const VALID_TOKEN: &str = "tok_valido";
/// Credenciais sintéticas.
pub const EMAIL: &str = "medico@example.invalid";
pub const PASSWORD: &str = "senha-de-teste";

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    /// Caminho com a query string.
    pub path: String,
    pub body: String,
    pub authorization: Option<String>,
}

impl Request {
    /// Caminho sem a query string.
    pub fn route(&self) -> &str {
        self.path.split('?').next().unwrap_or("")
    }

    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or_else(|err| {
            panic!("corpo de {} não é JSON ({err}): {}", self.path, self.body)
        })
    }
}

#[derive(Debug, Clone)]
struct Route {
    method: String,
    path: String,
    status: u16,
    body: String,
}

#[derive(Default)]
struct State {
    routes: Mutex<Vec<Route>>,
    requests: Mutex<Vec<Request>>,
    in_flight: AtomicUsize,
    max_in_flight: AtomicUsize,
    delay: Mutex<Duration>,
    required_token: Mutex<Option<String>>,
}

pub struct FakeMedx {
    url: String,
    state: Arc<State>,
}

impl FakeMedx {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("porta local");
        let url = format!("http://{}", listener.local_addr().expect("endereço local"));
        let state = Arc::new(State::default());
        let shared = Arc::clone(&state);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let state = Arc::clone(&shared);
                std::thread::spawn(move || handle(stream, &state));
            }
        });
        Self { url, state }
    }

    /// Origem do fake (`http://127.0.0.1:<porta>`), sem `/api`.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Registra a resposta de `method` em `path` (sem a query string, com o
    /// prefixo `/api`). O registro mais recente para a mesma rota vence.
    pub fn on(&self, method: &str, path: &str, status: u16, body: impl Into<String>) -> &Self {
        self.state.routes.lock().unwrap().push(Route {
            method: method.to_owned(),
            path: path.to_owned(),
            status,
            body: body.into(),
        });
        self
    }

    /// Atalho para uma resposta 200 com JSON.
    pub fn json(&self, method: &str, path: &str, value: Value) -> &Self {
        self.on(method, path, 200, value.to_string())
    }

    /// Rotas de login da MedX: o login sempre passa e emite [`NEW_TOKEN`].
    pub fn with_login(&self) -> &Self {
        self.on(
            "GET",
            "/api/LoginUnificado/VerificaEmailCripto",
            200,
            "\"Ok:Success:SoftwareId:4242\"",
        );
        self.json(
            "GET",
            "/api/security/getkeys",
            serde_json::json!({ "KeyId": "k1", "PublicKey": PUBLIC_KEY_XML }),
        );
        self.on(
            "POST",
            "/api/LoginUnificado/loginV3",
            200,
            format!("\"{NEW_TOKEN}\""),
        )
    }

    /// Fora das rotas de login, responde 401 a quem não mandar este token.
    pub fn require_token(&self, token: &str) -> &Self {
        *self.state.required_token.lock().unwrap() = Some(token.to_owned());
        self
    }

    /// Atraso antes de cada resposta.
    pub fn set_delay(&self, delay: Duration) -> &Self {
        *self.state.delay.lock().unwrap() = delay;
        self
    }

    pub fn requests(&self) -> Vec<Request> {
        self.state.requests.lock().unwrap().clone()
    }

    /// Requests a uma rota (sem a query string).
    pub fn requests_to(&self, route: &str) -> Vec<Request> {
        self.requests()
            .into_iter()
            .filter(|request| request.route() == route)
            .collect()
    }

    /// Maior número de requests atendidas ao mesmo tempo.
    pub fn max_in_flight(&self) -> usize {
        self.state.max_in_flight.load(Ordering::SeqCst)
    }

    /// Sessão salva válida para este fake, com [`VALID_TOKEN`].
    pub fn session(&self) -> Session {
        Session {
            token: VALID_TOKEN.to_owned(),
            email: EMAIL.to_owned(),
            db_id: "4242".to_owned(),
            host: self.url.clone(),
        }
    }

    /// Configuração apontada para este fake, sem credenciais e sem escrita.
    pub fn config(&self) -> Config {
        Config {
            host: Some(self.url.clone()),
            credentials: None,
            allow_write: false,
        }
    }

    /// Como [`FakeMedx::config`], com a escrita ligada.
    pub fn config_with_write(&self) -> Config {
        Config {
            allow_write: true,
            ..self.config()
        }
    }

    pub fn credentials() -> Credentials {
        Credentials {
            email: EMAIL.to_owned(),
            password: PASSWORD.to_owned(),
        }
    }
}

fn handle(stream: TcpStream, state: &State) {
    let Ok(read_half) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(read_half);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_owned();
    let path = parts.next().unwrap_or("").to_owned();

    let mut content_length = 0usize;
    let mut authorization = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) <= 2 {
            break;
        }
        let (name, value) = line.split_once(':').unwrap_or((&line, ""));
        let value = value.trim().to_owned();
        match name.trim().to_ascii_lowercase().as_str() {
            "content-length" => content_length = value.parse().unwrap_or(0),
            "authorization" => authorization = Some(value),
            _ => {}
        }
    }
    let mut body = vec![0u8; content_length];
    let _ = reader.read_exact(&mut body);

    let request = Request {
        method,
        path,
        body: String::from_utf8_lossy(&body).into_owned(),
        authorization,
    };
    state.requests.lock().unwrap().push(request.clone());

    let now = state.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
    state.max_in_flight.fetch_max(now, Ordering::SeqCst);
    let delay = *state.delay.lock().unwrap();
    if !delay.is_zero() {
        std::thread::sleep(delay);
    }
    let (status, body) = respond(&request, state);
    state.in_flight.fetch_sub(1, Ordering::SeqCst);

    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "Error",
    };
    let mut stream = stream;
    let _ = stream.write_all(
        format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    );
}

fn respond(request: &Request, state: &State) -> (u16, String) {
    let route = request.route();
    let is_login = route.starts_with("/api/LoginUnificado/") || route == "/api/security/getkeys";
    if !is_login
        && let Some(token) = state.required_token.lock().unwrap().as_deref()
        && request.authorization.as_deref() != Some(&format!("Bearer {token}"))
    {
        return (401, String::new());
    }
    let routes = state.routes.lock().unwrap();
    routes
        .iter()
        .rev()
        .find(|r| r.method == request.method && r.path == route)
        .map(|r| (r.status, r.body.clone()))
        .unwrap_or((404, String::new()))
}
