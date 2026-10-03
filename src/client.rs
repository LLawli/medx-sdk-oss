//! Cliente HTTP autenticado central do SDK.
//!
//! Toda request à API passa por aqui, garantindo:
//! - Injeção automática do `Authorization: Bearer <token>`
//! - Retry transparente em 401 (re-login com credenciais guardadas)
//! - Mapeamento uniforme de status HTTP → `MedxError`

use reqwest::blocking::{Client, RequestBuilder};
use reqwest::Method;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Mutex;

use crate::{auth, error::MedxError, session::Session};

/// Endereço padrão da plataforma. É a única ocorrência do host no código:
/// `tests/host_tests.rs` reprova qualquer outra.
macro_rules! default_host {
    () => {
        "https://v65.medx.med.br"
    };
}

/// Host padrão (origem, sem `/api`).
pub const DEFAULT_HOST: &str = default_host!();

/// Base da API no host padrão.
pub const DEFAULT_BASE_URL: &str = concat!(default_host!(), "/api");

/// Reduz um host ou uma base URL à origem: sem barra final e sem `/api`.
///
/// O MedX serve a API na mesma origem do webapp (`<origem>/api/`), e o webapp
/// alterna entre hosts sem aviso. Aceitar as duas formas deixa quem configura
/// passar o endereço que tiver em mãos.
pub fn normalize_host(url: &str) -> String {
    let url = url.trim().trim_end_matches('/');
    url.strip_suffix("/api")
        .unwrap_or(url)
        .trim_end_matches('/')
        .to_string()
}

/// Base da API (`<host>/api`) para um host.
pub fn api_base_url(host: &str) -> String {
    format!("{}/api", normalize_host(host))
}

// ── Struct principal ──────────────────────────────────────────────────────────

pub struct MedxClient {
    http: Client,
    pub base_url: String,
    /// Sessão atual protegida por Mutex para permitir re-login interno.
    session: Mutex<Session>,
    /// Credenciais armazenadas para retry automático em 401.
    credentials: Option<(String, String)>,
}

// ── Construtores ──────────────────────────────────────────────────────────────

impl MedxClient {
    fn new(session: Session, base_url: String, credentials: Option<(String, String)>) -> Self {
        MedxClient {
            http: Client::builder()
                .user_agent("medx-sdk/0.1")
                .build()
                .expect("falha ao construir reqwest::Client"),
            base_url,
            session: Mutex::new(session),
            credentials,
        }
    }

    /// Cria um cliente a partir de uma sessão já existente (sem retry em 401).
    ///
    /// Usa o host em que o token foi emitido (`session.host`).
    pub fn from_session(session: Session) -> Self {
        let base_url = api_base_url(&session.host);
        Self::from_session_at(session, base_url)
    }

    /// Versão com base_url customizada (ex.: `https://care-app65…/api`, ou um servidor de teste).
    pub fn from_session_at(session: Session, base_url: String) -> Self {
        Self::new(session, base_url, None)
    }

    /// Cria um cliente a partir de uma sessão existente com credenciais para retry em 401.
    ///
    /// Útil quando a sessão pode expirar e o cliente precisa renovar automaticamente
    /// (ex: CLI com credenciais disponíveis via variáveis de ambiente). Usa o host
    /// em que o token foi emitido (`session.host`).
    pub fn from_session_with_credentials(
        session: Session,
        email: String,
        password: String,
    ) -> Self {
        let base_url = api_base_url(&session.host);
        Self::from_session_with_credentials_at(session, base_url, email, password)
    }

    /// Versão de [`from_session_with_credentials`](Self::from_session_with_credentials)
    /// com base_url customizada. O re-login vai para o host dessa base_url.
    pub fn from_session_with_credentials_at(
        session: Session,
        base_url: String,
        email: String,
        password: String,
    ) -> Self {
        Self::new(session, base_url, Some((email, password)))
    }

    /// Faz login no host padrão e retorna um cliente com retry automático em 401.
    ///
    /// As credenciais são guardadas em memória para que o cliente possa
    /// renovar silenciosamente a sessão quando ela expirar.
    pub fn login(email: &str, password: &str) -> Result<Self, MedxError> {
        Self::login_at(DEFAULT_HOST, email, password)
    }

    /// Faz login em `host` (com ou sem `/api`) e retorna um cliente com retry
    /// automático em 401 no mesmo host.
    pub fn login_at(host: &str, email: &str, password: &str) -> Result<Self, MedxError> {
        let session = auth::login_at(host, email, password)?;
        let base_url = api_base_url(&session.host);
        Ok(Self::new(
            session,
            base_url,
            Some((email.to_string(), password.to_string())),
        ))
    }

    /// Host (origem, sem `/api`) para onde o cliente envia as requests.
    pub fn host(&self) -> String {
        normalize_host(&self.base_url)
    }

    /// Retorna um clone da sessão atual (token, email, db_id).
    pub fn session(&self) -> Session {
        self.session.lock().unwrap().clone()
    }
}

// ── Métodos públicos HTTP ─────────────────────────────────────────────────────

impl MedxClient {
    /// GET autenticado → desserializa o body em `T`.
    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, MedxError> {
        let resp = self.execute(path, Method::GET, None)?;
        Ok(resp.json()?)
    }

    /// GET autenticado → retorna o body como `String` pura.
    pub fn get_text(&self, path: &str) -> Result<String, MedxError> {
        let resp = self.execute(path, Method::GET, None)?;
        Ok(resp.text()?)
    }

    /// POST autenticado com JSON body → desserializa resposta em `T`.
    pub fn post<B, T>(&self, path: &str, body: &B) -> Result<T, MedxError>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        let json = serde_json::to_value(body)?;
        let resp = self.execute(path, Method::POST, Some(json))?;
        Ok(resp.json()?)
    }

    /// POST autenticado sem body → retorna body como `String`.
    pub fn post_empty(&self, path: &str) -> Result<String, MedxError> {
        let resp = self.execute(path, Method::POST, None)?;
        Ok(resp.text()?)
    }

    /// PUT autenticado com JSON body → desserializa resposta em `T`.
    pub fn put<B, T>(&self, path: &str, body: &B) -> Result<T, MedxError>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        let json = serde_json::to_value(body)?;
        let resp = self.execute(path, Method::PUT, Some(json))?;
        Ok(resp.json()?)
    }

    /// DELETE autenticado → espera 200/204 e descarta o body.
    pub fn delete(&self, path: &str) -> Result<(), MedxError> {
        self.execute(path, Method::DELETE, None)?;
        Ok(())
    }

    /// DELETE autenticado → retorna o body como `String` pura.
    ///
    /// Alguns endpoints respondem `200` com um corpo textual que indica recusa
    /// (ex.: `"negado"`), em vez de um status de erro. Use esta variante quando
    /// o corpo precisa ser inspecionado.
    pub fn delete_text(&self, path: &str) -> Result<String, MedxError> {
        let resp = self.execute(path, Method::DELETE, None)?;
        Ok(resp.text()?)
    }
}

// ── Núcleo de execução ────────────────────────────────────────────────────────

impl MedxClient {
    /// Monta e dispara a request com token atual.
    /// Em caso de 401, tenta re-login e repete até `MAX_RETRIES` vezes.
    fn execute(
        &self,
        path: &str,
        method: Method,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::blocking::Response, MedxError> {
        const MAX_RETRIES: u8 = 5;
        let mut resp = self.send_once(path, &method, body.as_ref())?;

        for _ in 0..MAX_RETRIES {
            if resp.status().as_u16() != 401 {
                return Self::check_status(resp);
            }
            if !self.try_relogin()? {
                return Err(MedxError::InvalidCredentials);
            }
            resp = self.send_once(path, &method, body.as_ref())?;
        }

        Self::check_status(resp)
    }

    /// Envia uma única request com o token atual (sem retry).
    fn send_once(
        &self,
        path: &str,
        method: &Method,
        body: Option<&serde_json::Value>,
    ) -> Result<reqwest::blocking::Response, MedxError> {
        let token = self.session.lock().unwrap().token.clone();
        let url = self.full_url(path);

        let mut builder: RequestBuilder = match method.as_str() {
            "GET" => self.http.get(&url),
            "POST" => self.http.post(&url),
            "PUT" => self.http.put(&url),
            "DELETE" => self.http.delete(&url),
            _ => self.http.request(method.clone(), &url),
        };

        builder = builder.bearer_auth(&token);

        if let Some(json) = body {
            builder = builder.json(json);
        }

        Ok(builder.send()?)
    }

    /// Tenta re-login com credenciais guardadas e atualiza a sessão interna.
    /// Retorna `true` se conseguiu, `false` se não havia credenciais.
    fn try_relogin(&self) -> Result<bool, MedxError> {
        let Some((ref email, ref password)) = self.credentials else {
            return Ok(false);
        };

        eprintln!("[medx-sdk] token expirado — renovando sessão...");
        // Mesmo host das requests: o token só vale na origem em que foi emitido.
        let new_session = auth::login_at(&self.host(), email, password)?;
        *self.session.lock().unwrap() = new_session;
        eprintln!("[medx-sdk] sessão renovada com sucesso.");
        Ok(true)
    }

    /// Mapeia status HTTP de erro para `MedxError`.
    fn check_status(
        resp: reqwest::blocking::Response,
    ) -> Result<reqwest::blocking::Response, MedxError> {
        let status = resp.status().as_u16();
        match status {
            200..=299 => Ok(resp),
            401 => Err(MedxError::InvalidCredentials),
            s => {
                let message = resp
                    .json::<serde_json::Value>()
                    .ok()
                    .and_then(|v| {
                        v.get("Message")
                            .or_else(|| v.get("message"))
                            .and_then(|m| m.as_str())
                            .map(str::to_string)
                    })
                    .unwrap_or_else(|| format!("HTTP {s}"));
                Err(MedxError::Api { status: s, message })
            }
        }
    }

    /// Monta a URL completa a partir do base_url e de um path relativo.
    pub fn full_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Session;

    fn dummy_session() -> Session {
        Session {
            token: "tok_test_123".to_string(),
            email: "test@example.com".to_string(),
            db_id: "db999".to_string(),
            host: "https://host.example.com".to_string(),
        }
    }

    #[test]
    fn normalize_host_aceita_origem_e_base_da_api() {
        for url in [
            "https://h.example.com",
            "https://h.example.com/",
            "https://h.example.com/api",
            "https://h.example.com/api/",
            "  https://h.example.com/api  ",
        ] {
            assert_eq!(
                normalize_host(url),
                "https://h.example.com",
                "entrada: {url:?}"
            );
        }
    }

    #[test]
    fn default_base_url_e_o_host_padrao_mais_api() {
        assert_eq!(DEFAULT_BASE_URL, api_base_url(DEFAULT_HOST));
    }

    #[test]
    fn from_session_usa_o_host_da_sessao() {
        let client = MedxClient::from_session(dummy_session());
        assert_eq!(client.base_url, "https://host.example.com/api");
        assert_eq!(client.host(), "https://host.example.com");
    }

    #[test]
    fn full_url_sem_barra_final() {
        let client =
            MedxClient::from_session_at(dummy_session(), "https://api.example.com".to_string());
        assert_eq!(
            client.full_url("security/getcurrentuser"),
            "https://api.example.com/security/getcurrentuser"
        );
    }

    #[test]
    fn full_url_com_barra_final_no_base() {
        let client =
            MedxClient::from_session_at(dummy_session(), "https://api.example.com/".to_string());
        assert_eq!(
            client.full_url("security/getcurrentuser"),
            "https://api.example.com/security/getcurrentuser"
        );
    }

    #[test]
    fn full_url_com_barra_inicial_no_path() {
        let client =
            MedxClient::from_session_at(dummy_session(), "https://api.example.com".to_string());
        assert_eq!(
            client.full_url("/security/getcurrentuser"),
            "https://api.example.com/security/getcurrentuser"
        );
    }

    #[test]
    fn session_retorna_clone_correto() {
        let session = dummy_session();
        let client = MedxClient::from_session(session.clone());
        let got = client.session();
        assert_eq!(got.token, session.token);
        assert_eq!(got.email, session.email);
        assert_eq!(got.db_id, session.db_id);
    }
}
