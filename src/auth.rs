use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::{
    client::{normalize_host, DEFAULT_HOST},
    crypto::{rsa_oaep_encrypt, rsa_public_key_from_xml},
    error::MedxError,
    session::{self, Session},
    util::encode_query_value,
};

// ── DTOs de resposta ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct KeysResponse {
    #[serde(rename = "KeyId")]
    key_id: String,
    #[serde(rename = "PublicKey")]
    public_key: String,
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(rename = "Message")]
    message: Option<String>,
}

// ── DTO de envio ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct LoginDto {
    #[serde(rename = "AssymetricKeyId")]
    assymetric_key_id: String,
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "Senha")]
    senha: String,
    ip: String,
    #[serde(rename = "Mobile")]
    mobile: u8,
    #[serde(rename = "dbId")]
    db_id: String,
}

// ── Helpers internos ─────────────────────────────────────────────────────────

/// Etapa 1 – resolve SoftwareId/dbId a partir do e-mail.
fn resolve_db_id(client: &Client, host: &str, email: &str) -> Result<String, MedxError> {
    let url = format!(
        "{host}/api/LoginUnificado/VerificaEmailCripto?Email={}&dbId=",
        encode_query_value(email)
    );

    let resp = client.get(&url).send()?;
    let status = resp.status().as_u16();
    let body = resp.text()?;

    if status != 200 {
        return Err(MedxError::Api {
            status,
            message: body,
        });
    }

    // Resposta: "Ok:Success:SoftwareId:<id>" — o id pode ser base64 ou numérico
    // A string pode vir com aspas externas (JSON string) ou sem
    let body = body.trim().trim_matches('"').to_string();
    if let Some(id) = body.strip_prefix("Ok:Success:SoftwareId:") {
        return Ok(id.trim().to_string());
    }

    // Resposta: "Ok:JSON:[{...}]" (múltiplas clínicas) — sem suporte nesta versão
    if body.contains("JSON:") {
        return Err(MedxError::UnexpectedResponse(
            "Múltiplas clínicas vinculadas ao e-mail — selecione um dbId manualmente.".into(),
        ));
    }

    Err(MedxError::UnexpectedResponse(body))
}

/// Etapa 2 – busca a chave RSA pública do servidor.
fn fetch_rsa_keys(client: &Client, host: &str) -> Result<KeysResponse, MedxError> {
    let resp = client.get(format!("{host}/api/security/getkeys")).send()?;

    let status = resp.status().as_u16();
    if status != 200 {
        return Err(MedxError::Api {
            status,
            message: "Falha ao buscar chaves RSA".into(),
        });
    }
    Ok(resp.json::<KeysResponse>()?)
}

/// Etapa 3 – tenta o POST de login e retorna o token ou um erro estruturado.
fn post_login(client: &Client, host: &str, dto: &LoginDto) -> Result<String, MedxError> {
    let resp = client
        .post(format!("{host}/api/LoginUnificado/loginV3"))
        .json(dto)
        .send()?;

    let status = resp.status().as_u16();

    if status == 200 {
        // Retorna o token como string pura
        let token = resp.text()?.trim_matches('"').to_string();
        return Ok(token);
    }

    // Tenta ler o body como JSON de erro
    let body_text = resp.text().unwrap_or_default();
    let message = serde_json::from_str::<ErrorBody>(&body_text)
        .ok()
        .and_then(|b| b.message)
        .unwrap_or_else(|| body_text.clone());

    if status == 401 {
        return Err(MedxError::InvalidCredentials);
    }

    Err(MedxError::Api { status, message })
}

/// Invalida uma sessão activa via token antigo.
fn remove_active_session(client: &Client, host: &str, old_token: &str) -> Result<(), MedxError> {
    let resp = client
        .post(format!(
            "{host}/api/security/removetokeninuse?token={}",
            encode_query_value(old_token)
        ))
        .send()?;

    let status = resp.status().as_u16();
    if status != 200 {
        return Err(MedxError::Api {
            status,
            message: "Falha ao remover sessão ativa".into(),
        });
    }
    Ok(())
}

/// Extrai o token antigo da mensagem de erro "usuário já logado".
/// Formato: "usuário já logado, <info>, token  <token>"
fn extract_old_token(message: &str) -> Option<String> {
    let parts: Vec<&str> = message.splitn(3, ',').collect();
    parts
        .get(2)
        .map(|s| {
            s.replace("token  ", "")
                .replace("token ", "")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
}

// ── API pública ───────────────────────────────────────────────────────────────

/// Faz login na plataforma MedX, no host padrão.
///
/// Atalho para [`login_at`] com [`DEFAULT_HOST`].
pub fn login(email: &str, password: &str) -> Result<Session, MedxError> {
    login_at(DEFAULT_HOST, email, password)
}

/// Etapa do login, relatada por [`login_at_with_progress`] a quem quiser
/// mostrar o progresso. A biblioteca não escreve nada no terminal: quem
/// decide o que mostrar é o programa (o medx-cli, por exemplo).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoginStep {
    /// Vai consultar o `dbId` do e-mail.
    VerifyingEmail,
    /// O `dbId` do e-mail foi resolvido.
    DbIdResolved(String),
    /// Vai buscar a chave pública RSA do servidor.
    FetchingKey,
    /// Vai enviar o login.
    Authenticating,
    /// O servidor recusou porque a conta já tem uma sessão ativa.
    ActiveSessionDetected,
    /// A sessão anterior foi invalidada; o login vai ser repetido.
    PreviousSessionRemoved,
    /// A mensagem de sessão ativa não trazia o token antigo; o login vai ser
    /// repetido sem invalidar a sessão anterior.
    OldTokenNotFound,
    /// Login feito e sessão salva.
    Done,
}

/// Faz login como [`login_at`], chamando `progress` a cada etapa, na ordem
/// em que acontecem.
pub fn login_at_with_progress(
    host: &str,
    email: &str,
    password: &str,
    progress: &mut dyn FnMut(LoginStep),
) -> Result<Session, MedxError> {
    let host = normalize_host(host);
    let client = Client::builder().user_agent("medx-sdk/0.1").build()?;

    // 1. Descobre o dbId
    progress(LoginStep::VerifyingEmail);
    let db_id = resolve_db_id(&client, &host, email)?;
    progress(LoginStep::DbIdResolved(db_id.clone()));

    // 2. Busca chaves RSA
    progress(LoginStep::FetchingKey);
    let keys = fetch_rsa_keys(&client, &host)?;

    // 3. Encripta a senha
    let public_key = rsa_public_key_from_xml(&keys.public_key)?;
    let encrypted_password = rsa_oaep_encrypt(&public_key, password)?;

    // 4. Monta e envia o DTO de login
    let dto = LoginDto {
        assymetric_key_id: keys.key_id,
        email: email.to_string(),
        senha: encrypted_password,
        ip: String::new(),
        mobile: 0,
        db_id: db_id.clone(),
    };

    progress(LoginStep::Authenticating);
    match post_login(&client, &host, &dto) {
        Ok(token) => {
            let session = Session {
                token,
                email: email.to_string(),
                db_id,
                host,
            };
            session::save(&session)?;
            progress(LoginStep::Done);
            return Ok(session);
        }

        Err(MedxError::Api {
            status: 400,
            ref message,
        }) if message.contains("usuário já logado") => {
            progress(LoginStep::ActiveSessionDetected);

            if let Some(old_token) = extract_old_token(message) {
                remove_active_session(&client, &host, &old_token)?;
                progress(LoginStep::PreviousSessionRemoved);
            } else {
                progress(LoginStep::OldTokenNotFound);
            }

            // Retry após invalidar
            let token = post_login(&client, &host, &dto)?;
            let session = Session {
                token,
                email: email.to_string(),
                db_id,
                host,
            };
            session::save(&session)?;
            progress(LoginStep::Done);
            Ok(session)
        }

        Err(e) => Err(e),
    }
}

/// Faz login na plataforma MedX em `host` (com ou sem `/api` no fim), sem
/// escrever nada no terminal.
///
/// - Resolve automaticamente o `dbId` a partir do e-mail.
/// - Encripta a senha com RSA-OAEP (SHA-1) usando a chave pública do servidor.
/// - Se houver sessão ativa, **invalida-a automaticamente** e tenta de novo.
/// - Persiste o token em `session.json`, no diretório de [`session::config_dir`].
///
/// Retorna a `Session` com o token salvo e o host em que ele foi emitido.
pub fn login_at(host: &str, email: &str, password: &str) -> Result<Session, MedxError> {
    login_at_with_progress(host, email, password, &mut |_| {})
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_old_token_formato_esperado() {
        let msg = "usuário já logado, dispositivo: web, token  ABCDEF123";
        let tok = extract_old_token(msg);
        assert_eq!(tok.as_deref(), Some("ABCDEF123"));
    }

    #[test]
    fn extract_old_token_com_duplo_espaco() {
        let msg = "usuário já logado, info, token  tok_xyz_789";
        assert_eq!(extract_old_token(msg).as_deref(), Some("tok_xyz_789"));
    }

    #[test]
    fn extract_old_token_sem_terceira_parte_retorna_none() {
        let msg = "usuário já logado, info";
        assert!(extract_old_token(msg).is_none());
    }

    #[test]
    fn extract_old_token_string_vazia_retorna_none() {
        assert!(extract_old_token("").is_none());
    }
}
