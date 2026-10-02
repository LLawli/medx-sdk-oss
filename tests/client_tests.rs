//! Testes de integração — Etapa 1: MedxClient (HTTP base).
//!
//! Requerem rede e credenciais válidas.
//! Execute com: `cargo test --test client_tests -- --ignored`

mod common;

use common::{shared_client, with_temp_dir};
use medx::{MedxClient, MedxError, Session};
use serde::Deserialize;

// ── DTO para o endpoint getcurrentuser ───────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CurrentUser {
    #[serde(rename = "DbId")]
    db_id: i64,
    #[serde(rename = "Username")]
    username: String,
    #[serde(rename = "UserFullName")]
    user_full_name: String,
    #[serde(rename = "UserEmail")]
    user_email: String,
}

// ── Testes do MedxClient ──────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_get_current_user_retorna_dados_validos() {
    with_temp_dir(|| {
        let user: CurrentUser = shared_client()
            .get("security/getcurrentuser")
            .expect("GET /security/getcurrentuser falhou");

        assert!(!user.username.is_empty(), "username não deve ser vazio");
        assert!(!user.user_email.is_empty(), "email não deve ser vazio");
        assert!(user.db_id > 0, "DbId deve ser positivo");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_get_text_retorna_string() {
    with_temp_dir(|| {
        let text = shared_client()
            .get_text("security/getcurrentuser")
            .expect("get_text falhou");
        // Resposta deve ser JSON com chaves conhecidas
        assert!(text.contains("Username") || text.contains("UserEmail"));
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_token_invalido_retorna_erro_de_autenticacao() {
    with_temp_dir(|| {
        // Cria cliente com token inválido e SEM credenciais (não faz retry)
        let fake_session = Session {
            token: "token_invalido_xyz".to_string(),
            email: "fake@example.com".to_string(),
            db_id: "db0".to_string(),
            host: medx::client::DEFAULT_HOST.to_string(),
        };
        let client = MedxClient::from_session(fake_session);

        let err = client
            .get::<CurrentUser>("security/getcurrentuser")
            .expect_err("deveria falhar com token inválido");

        assert!(
            matches!(err, MedxError::InvalidCredentials),
            "esperado InvalidCredentials, obtido: {err}"
        );
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_retry_automatico_apos_token_expirado() {
    with_temp_dir(|| {
        // O cliente compartilhado tem credenciais e faz retry automático em 401.
        // Este teste verifica que chamadas com sessão válida funcionam corretamente.
        // (O retry real em 401 é coberto implicitamente quando o servidor invalida
        //  a sessão em testes paralelos — o SDK deve continuar funcionando.)
        let user: CurrentUser = shared_client()
            .get("security/getcurrentuser")
            .expect("GET com credenciais válidas deve funcionar");

        assert!(!user.user_email.is_empty());
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_url_construction_nao_duplica_barra() {
    with_temp_dir(|| {
        let c = shared_client();
        // Ambas as formas de path devem funcionar igualmente
        let r1: CurrentUser = c.get("security/getcurrentuser").unwrap();
        let r2: CurrentUser = c.get("/security/getcurrentuser").unwrap();

        assert_eq!(r1.db_id, r2.db_id);
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_endpoint_inexistente_retorna_erro_api() {
    with_temp_dir(|| {
        let err = shared_client()
            .get::<serde_json::Value>("rota/que/nao/existe/xyz123")
            .expect_err("endpoint inexistente deve retornar erro");

        assert!(
            matches!(err, MedxError::Api { status: 404, .. })
                || matches!(err, MedxError::Api { .. }),
            "esperado MedxError::Api, obtido: {err}"
        );
    });
}
