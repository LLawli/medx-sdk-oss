//! Configuração lida das variáveis de ambiente, e a escolha de como se
//! conectar à MedX a partir dela e da sessão salva.

use medx::Session;

/// Credenciais de login (`MEDX_LOGIN_CREDENTIAL` e `MEDX_PASSWORD_CREDENTIAL`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

/// Configuração do servidor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    /// Host da MedX (origem, sem `/api`) vindo de `MEDX_BASE_URL`, já
    /// normalizado. `None` quando a variável falta ou está vazia.
    pub host: Option<String>,
    /// Presentes só quando as duas variáveis têm valor.
    pub credentials: Option<Credentials>,
    /// `MEDX_MCP_ALLOW_WRITE`: liga as ferramentas de escrita.
    pub allow_write: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("MEDX_BASE_URL deve começar com https:// ou http:// (recebido: {0})")]
    BaseUrl(String),
    #[error("MEDX_MCP_ALLOW_WRITE aceita 1, true, 0 ou false (recebido: {0})")]
    AllowWrite(String),
}

impl Config {
    /// Lê a configuração por `var`, que devolve o valor de uma variável de
    /// ambiente: `std::env::var` no binário, um mapa nos testes.
    ///
    /// - `MEDX_BASE_URL`: aparada; vazia conta como ausente; precisa começar
    ///   com `https://` ou `http://`; é normalizada para a origem
    ///   (`medx::client::normalize_host`).
    /// - Credenciais: só quando as duas variáveis existem e não estão vazias
    ///   depois de aparadas (o valor guardado não é aparado).
    /// - `MEDX_MCP_ALLOW_WRITE`: aparada, sem diferenciar maiúsculas; ausente,
    ///   vazia, `0` ou `false` desligam; `1` ou `true` ligam; qualquer outro
    ///   valor é erro, para um erro de digitação não mudar o modo em silêncio.
    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let host = match var("MEDX_BASE_URL").map(|v| v.trim().to_owned()) {
            Some(url) if !url.is_empty() => {
                if !(url.starts_with("https://") || url.starts_with("http://")) {
                    return Err(ConfigError::BaseUrl(url));
                }
                Some(medx::client::normalize_host(&url))
            }
            _ => None,
        };

        let filled = |name: &str| var(name).filter(|v| !v.trim().is_empty());
        let credentials = match (
            filled("MEDX_LOGIN_CREDENTIAL"),
            filled("MEDX_PASSWORD_CREDENTIAL"),
        ) {
            (Some(email), Some(password)) => Some(Credentials { email, password }),
            _ => None,
        };

        let allow_write = match var("MEDX_MCP_ALLOW_WRITE") {
            None => false,
            Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
                "" | "0" | "false" => false,
                "1" | "true" => true,
                _ => return Err(ConfigError::AllowWrite(raw)),
            },
        };

        Ok(Self {
            host,
            credentials,
            allow_write,
        })
    }
}

/// Como abrir o cliente da MedX.
#[derive(Debug, Clone)]
pub enum ConnectionPlan {
    /// Reabre a sessão salva em `base_url` (`<host>/api`), com re-login em
    /// 401 quando há credenciais.
    Session {
        session: Session,
        base_url: String,
        credentials: Option<Credentials>,
    },
    /// Sem sessão salva: faz login em `host`.
    Login {
        host: String,
        credentials: Credentials,
    },
    /// Sem sessão salva e sem credenciais: não há como conectar.
    Missing,
}

/// Escolhe como conectar, com a mesma regra de host do medx-cli:
/// `MEDX_BASE_URL` vence; sem ela, o host em que o token da sessão foi
/// emitido; sem sessão, `medx::client::DEFAULT_HOST`.
pub fn plan_connection(config: &Config, session: Option<Session>) -> ConnectionPlan {
    match session {
        Some(session) => {
            let host = config.host.as_deref().unwrap_or(&session.host);
            let base_url = medx::client::api_base_url(host);
            ConnectionPlan::Session {
                session,
                base_url,
                credentials: config.credentials.clone(),
            }
        }
        None => match &config.credentials {
            Some(credentials) => ConnectionPlan::Login {
                host: config
                    .host
                    .clone()
                    .unwrap_or_else(|| medx::client::DEFAULT_HOST.to_owned()),
                credentials: credentials.clone(),
            },
            None => ConnectionPlan::Missing,
        },
    }
}
