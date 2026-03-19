use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{rsa_oaep_encrypt, rsa_public_key_from_xml},
    error::MedxError,
    session::{self, Session},
};

const BASE: &str = "https://v65.medx.med.br";

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
fn resolve_db_id(client: &Client, email: &str) -> Result<String, MedxError> {
    let url = format!(
        "{BASE}/api/LoginUnificado/VerificaEmailCripto?Email={}&dbId=",
        urlencoding_simple(email)
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
fn fetch_rsa_keys(client: &Client) -> Result<KeysResponse, MedxError> {
    let resp = client
        .get(format!("{BASE}/api/security/getkeys"))
        .send()?;

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
fn post_login(client: &Client, dto: &LoginDto) -> Result<String, MedxError> {
    let resp = client
        .post(format!("{BASE}/api/LoginUnificado/loginV3"))
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
fn remove_active_session(client: &Client, old_token: &str) -> Result<(), MedxError> {
    let resp = client
        .post(format!(
            "{BASE}/api/security/removetokeninuse?token={old_token}"
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
        .map(|s| s.replace("token  ", "").replace("token ", "").trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Encode URL simples para o e-mail (só substitui @, . etc. necessários).
pub(crate) fn urlencoding_simple(s: &str) -> String {
    s.replace('@', "%40")
}

// ── API pública ───────────────────────────────────────────────────────────────

/// Faz login na plataforma MedX.
///
/// - Resolve automaticamente o `dbId` a partir do e-mail.
/// - Encripta a senha com RSA-OAEP (SHA-1) usando a chave pública do servidor.
/// - Se houver sessão ativa, **invalida-a automaticamente** e tenta de novo.
/// - Persiste o token em `~/.config/medx-sdk/session.json`.
///
/// Retorna a `Session` com o token salvo.
pub fn login(email: &str, password: &str) -> Result<Session, MedxError> {
    let client = Client::builder()
        .user_agent("medx-sdk/0.1")
        .build()?;

    // 1. Descobre o dbId
    println!("→ Verificando e-mail...");
    let db_id = resolve_db_id(&client, email)?;
    println!("  dbId: {db_id}");

    // 2. Busca chaves RSA
    println!("→ Buscando chave pública RSA...");
    let keys = fetch_rsa_keys(&client)?;

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

    println!("→ Autenticando...");
    match post_login(&client, &dto) {
        Ok(token) => {
            let session = Session {
                token,
                email: email.to_string(),
                db_id,
            };
            session::save(&session)?;
            println!("✓ Login realizado. Token salvo.");
            return Ok(session);
        }

        Err(MedxError::Api { status: 400, ref message }) if message.contains("usuário já logado") => {
            println!("! Sessão ativa detectada. Invalidando...");

            if let Some(old_token) = extract_old_token(message) {
                remove_active_session(&client, &old_token)?;
                println!("  Sessão anterior removida. Tentando novamente...");
            } else {
                eprintln!("  Aviso: não foi possível extrair o token antigo da mensagem.");
            }

            // Retry após invalidar
            let token = post_login(&client, &dto)?;
            let session = Session {
                token,
                email: email.to_string(),
                db_id,
            };
            session::save(&session)?;
            println!("✓ Login realizado. Token salvo.");
            Ok(session)
        }

        Err(e) => Err(e),
    }
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencoding_substitui_arroba() {
        assert_eq!(urlencoding_simple("user@example.com"), "user%40example.com");
    }

    #[test]
    fn urlencoding_preserva_resto() {
        assert_eq!(urlencoding_simple("plain.text+filter"), "plain.text+filter");
    }

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
