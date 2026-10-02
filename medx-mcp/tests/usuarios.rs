//! Ferramentas de usuários, e o comportamento comum das ferramentas: JSON com
//! os nomes do SDK, `limite`, segredos fora e erros como resultado.

mod common;

use common::fake_medx::{FakeMedx, VALID_TOKEN};
use common::{call, call_error, call_json, connect, connect_fake, fixtures, texts};
use medx_mcp::config::Config;
use serde_json::json;

const CURRENT_USER: &str = "/api/security/getcurrentuser";
const USERS: &str = "/api/Users/GetUsers";

fn fake_with_users(n: i64) -> FakeMedx {
    let fake = FakeMedx::start();
    let users: Vec<_> = (1..=n)
        .map(|id| fixtures::user(id, &format!("USUARIO {id}")))
        .collect();
    fake.json("GET", USERS, json!(users));
    fake
}

#[tokio::test]
async fn usuario_atual_devolve_os_nomes_do_sdk() {
    let fake = FakeMedx::start();
    fake.json("GET", CURRENT_USER, fixtures::current_user());
    let client = connect_fake(&fake).await;

    let user = call_json(&client, "usuario_atual", json!({})).await;

    assert_eq!(user["db_id"], 4242);
    assert_eq!(user["full_name"], "DRA ANA TESTE");
    assert_eq!(user["email"], "medico@example.invalid");
    assert!(
        user.get("UserFullName").is_none(),
        "nome cru da API na saída: {user}"
    );

    let requests = fake.requests_to(CURRENT_USER);
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some(format!("Bearer {VALID_TOKEN}").as_str())
    );
}

#[tokio::test]
async fn usuario_atual_nao_expoe_a_chave_do_rd_station() {
    let fake = FakeMedx::start();
    fake.json("GET", CURRENT_USER, fixtures::current_user());
    let client = connect_fake(&fake).await;

    let result = call(&client, "usuario_atual", json!({})).await;
    let text = texts(&result).join("\n");

    assert!(!text.contains("rd_station_key"), "{text}");
    assert!(!text.contains("chave-secreta-rd"), "{text}");
}

#[tokio::test]
async fn resposta_e_json_compacto() {
    let fake = FakeMedx::start();
    fake.json("GET", CURRENT_USER, fixtures::current_user());
    let client = connect_fake(&fake).await;

    let result = call(&client, "usuario_atual", json!({})).await;
    let text = &texts(&result)[0];

    assert!(!text.contains('\n'), "JSON indentado: {text}");
    assert!(!text.contains(": "), "JSON com espaços: {text}");
}

#[tokio::test]
async fn listar_usuarios_sem_corte_tem_um_bloco_so() {
    let fake = fake_with_users(3);
    let client = connect_fake(&fake).await;

    let result = call(&client, "listar_usuarios", json!({})).await;
    assert_ne!(result.is_error, Some(true), "{result:?}");
    let blocks = texts(&result);

    assert_eq!(blocks.len(), 1, "{blocks:?}");
    let users: serde_json::Value = serde_json::from_str(&blocks[0]).unwrap();
    let users = users.as_array().expect("lista como array JSON");
    assert_eq!(users.len(), 3);
    assert_eq!(users[0]["id"], 1);
    assert_eq!(users[0]["full_name"], "USUARIO 1 COMPLETO");
}

#[tokio::test]
async fn listar_usuarios_respeita_o_limite_e_avisa_o_corte() {
    let fake = fake_with_users(5);
    let client = connect_fake(&fake).await;

    let result = call(&client, "listar_usuarios", json!({ "limite": 2 })).await;
    assert_ne!(result.is_error, Some(true), "{result:?}");
    let blocks = texts(&result);

    assert_eq!(blocks.len(), 2, "{blocks:?}");
    let users: serde_json::Value = serde_json::from_str(&blocks[0]).unwrap();
    assert_eq!(users.as_array().map(Vec::len), Some(2));
    assert!(
        blocks[1].contains("2 de 5"),
        "aviso de corte: {}",
        blocks[1]
    );
    assert!(
        blocks[1].contains("limite"),
        "aviso de corte: {}",
        blocks[1]
    );
}

#[tokio::test]
async fn limite_padrao_e_50() {
    let fake = fake_with_users(60);
    let client = connect_fake(&fake).await;

    let result = call(&client, "listar_usuarios", json!({})).await;
    let blocks = texts(&result);
    let users: serde_json::Value = serde_json::from_str(&blocks[0]).unwrap();

    assert_eq!(users.as_array().map(Vec::len), Some(50));
    assert!(blocks[1].contains("50 de 60"), "{}", blocks[1]);
}

#[tokio::test]
async fn limite_fora_da_faixa_e_erro_sem_chamar_a_medx() {
    let fake = fake_with_users(3);
    let client = connect_fake(&fake).await;

    for limite in [0, 501] {
        let message = call_error(&client, "listar_usuarios", json!({ "limite": limite })).await;
        assert!(message.contains("limite"), "{message}");
    }
    assert!(fake.requests().is_empty(), "{:?}", fake.requests());
}

#[tokio::test]
async fn erro_da_medx_vira_erro_da_ferramenta() {
    let fake = FakeMedx::start();
    fake.on("GET", CURRENT_USER, 500, "{\"Message\":\"falha interna\"}");
    let client = connect_fake(&fake).await;

    let message = call_error(&client, "usuario_atual", json!({})).await;

    assert!(message.contains("500"), "{message}");
}

#[tokio::test]
async fn sem_sessao_e_sem_credenciais_explica_o_que_configurar() {
    let client = connect(Config::default(), None).await;

    let message = call_error(&client, "usuario_atual", json!({})).await;

    assert!(message.contains("MEDX_LOGIN_CREDENTIAL"), "{message}");
    assert!(message.contains("medx-cli auth login"), "{message}");
}

#[tokio::test]
async fn leituras_sao_anotadas_como_somente_leitura() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    let tools = client.list_all_tools().await.expect("tools/list");
    for name in ["usuario_atual", "listar_usuarios"] {
        let tool = tools
            .iter()
            .find(|tool| tool.name == name)
            .unwrap_or_else(|| panic!("{name} não anunciada"));
        let annotations = tool.annotations.as_ref().expect("anotações");
        assert_eq!(annotations.read_only_hint, Some(true), "{name}");
        assert!(
            tool.description.as_deref().is_some_and(|d| !d.is_empty()),
            "{name} sem descrição"
        );
    }
}
