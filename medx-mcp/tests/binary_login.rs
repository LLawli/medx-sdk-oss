//! Login pelo binário: o login grava o session.json, então só roda num
//! processo filho com `MEDX_CONFIG_DIR` temporário. Confere também que nada do
//! login vaza para o stdout.

mod common;

use common::fake_medx::{EMAIL, FakeMedx, NEW_TOKEN, PASSWORD};
use common::fixtures;
use common::stdio::{StdioServer, result_text};
use serde_json::{Value, json};

const CURRENT_USER: &str = "/api/security/getcurrentuser";

fn saved_token(dir: &std::path::Path) -> String {
    let text = std::fs::read_to_string(dir.join("session.json")).expect("session.json salvo");
    let session: Value = serde_json::from_str(&text).expect("session.json válido");
    session["token"].as_str().unwrap_or_default().to_owned()
}

#[test]
fn sem_sessao_faz_login_com_as_credenciais() {
    let fake = FakeMedx::start();
    fake.with_login()
        .require_token(NEW_TOKEN)
        .json("GET", CURRENT_USER, fixtures::current_user());
    let dir = tempfile::tempdir().unwrap();

    let mut server = StdioServer::start(
        dir.path(),
        &[
            ("MEDX_BASE_URL", fake.url()),
            ("MEDX_LOGIN_CREDENTIAL", EMAIL),
            ("MEDX_PASSWORD_CREDENTIAL", PASSWORD),
        ],
    );
    server.initialize();
    let result = server.call_tool(2, "usuario_atual", json!({}));
    let (status, stderr) = server.finish();

    assert_ne!(result["isError"], true, "{result}\n{stderr}");
    let user: Value = serde_json::from_str(&result_text(&result)).expect("JSON");
    assert_eq!(user["full_name"], "DRA ANA TESTE");
    assert_eq!(fake.requests_to("/api/LoginUnificado/loginV3").len(), 1);
    assert_eq!(saved_token(dir.path()), NEW_TOKEN);
    assert!(status.success(), "{stderr}");
}

#[test]
fn token_vencido_refaz_o_login_e_repete_a_chamada() {
    let fake = FakeMedx::start();
    fake.with_login()
        .require_token(NEW_TOKEN)
        .json("GET", CURRENT_USER, fixtures::current_user());
    let dir = tempfile::tempdir().unwrap();
    let expired = json!({
        "token": "tok_vencido",
        "email": EMAIL,
        "db_id": "4242",
        "host": fake.url(),
    });
    std::fs::write(dir.path().join("session.json"), expired.to_string()).unwrap();

    let mut server = StdioServer::start(
        dir.path(),
        &[
            ("MEDX_LOGIN_CREDENTIAL", EMAIL),
            ("MEDX_PASSWORD_CREDENTIAL", PASSWORD),
        ],
    );
    server.initialize();
    let result = server.call_tool(2, "usuario_atual", json!({}));
    let (status, stderr) = server.finish();

    assert_ne!(result["isError"], true, "{result}\n{stderr}");
    let calls = fake.requests_to(CURRENT_USER);
    assert_eq!(
        calls.len(),
        2,
        "uma com o token vencido, outra depois do login"
    );
    assert_eq!(saved_token(dir.path()), NEW_TOKEN);
    assert!(status.success(), "{stderr}");
}

#[test]
fn login_recusado_vira_erro_da_ferramenta() {
    let fake = FakeMedx::start();
    fake.with_login()
        .on("POST", "/api/LoginUnificado/loginV3", 401, "");
    let dir = tempfile::tempdir().unwrap();

    let mut server = StdioServer::start(
        dir.path(),
        &[
            ("MEDX_BASE_URL", fake.url()),
            ("MEDX_LOGIN_CREDENTIAL", EMAIL),
            ("MEDX_PASSWORD_CREDENTIAL", PASSWORD),
        ],
    );
    server.initialize();
    let first = server.call_tool(2, "usuario_atual", json!({}));
    let second = server.call_tool(3, "usuario_atual", json!({}));
    let (_, stderr) = server.finish();

    assert_eq!(first["isError"], true, "{first}");
    assert!(
        result_text(&first).contains("Credenciais inválidas"),
        "{first}"
    );
    assert_eq!(second["isError"], true, "{second}");
    assert_eq!(
        fake.requests_to("/api/LoginUnificado/loginV3").len(),
        2,
        "a segunda chamada deveria tentar o login de novo\n{stderr}"
    );
    assert!(!dir.path().join("session.json").exists());
}
