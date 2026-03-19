use thiserror::Error;

#[derive(Error, Debug)]
pub enum MedxError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Credenciais inválidas")]
    InvalidCredentials,

    #[error("Erro de RSA: {0}")]
    Rsa(String),

    #[error("Formato de chave inválido: {0}")]
    KeyFormat(String),

    #[error("Erro de base64: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Erro de IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro de JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Erro da API: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Resposta inesperada da API: {0}")]
    UnexpectedResponse(String),
}
