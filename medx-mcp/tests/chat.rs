//! Ferramentas de leitura do chat interno.

mod common;

use common::fake_medx::FakeMedx;
use common::{call_json, connect_fake, fixtures};
use serde_json::{Value, json};

fn ids(list: &Value) -> Vec<i64> {
    list.as_array()
        .expect("lista")
        .iter()
        .map(|item| item["id"].as_i64().expect("id"))
        .collect()
}

#[tokio::test]
async fn listar_usuarios_chat() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/chat/GetUsers",
        json!([fixtures::chat_user(2, "RECEPCAO TESTE")]),
    );
    let client = connect_fake(&fake).await;

    let users = call_json(&client, "listar_usuarios_chat", json!({})).await;

    assert_eq!(users[0]["id"], 2);
    assert_eq!(users[0]["unread"], 2);
    assert_eq!(users[0]["online"], true);
}

#[tokio::test]
async fn historico_chat_mais_recentes_primeiro() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/chat/LoadMessageHistory",
        json!([
            fixtures::chat_message(1, "2026-10-01T09:00:00", "bom dia"),
            fixtures::chat_message(2, "2026-10-01T09:05:00", "paciente chegou"),
        ]),
    );
    let client = connect_fake(&fake).await;

    let messages = call_json(&client, "historico_chat", json!({ "usuario_id": 2 })).await;

    assert_eq!(ids(&messages), [2, 1]);
    assert_eq!(messages[0]["text"], "paciente chegou");
    assert!(fake.requests()[0].path.ends_with("?IddoRemetente=2"));
}

#[tokio::test]
async fn mensagens_nao_lidas_chat() {
    let fake = FakeMedx::start();
    fake.json("GET", "/api/chat/GetChatCount", json!({ "Total": 4 }));
    let client = connect_fake(&fake).await;

    let count = call_json(&client, "mensagens_nao_lidas_chat", json!({})).await;

    assert_eq!(count, json!({ "total": 4 }));
}

#[tokio::test]
async fn mensagens_recebidas_chat() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/chat/GetIncomingMessage",
        json!([fixtures::chat_message(
            9,
            "2026-10-01T09:00:00",
            "pode atender?"
        )]),
    );
    let client = connect_fake(&fake).await;

    let messages = call_json(&client, "mensagens_recebidas_chat", json!({})).await;

    assert_eq!(messages[0]["from_name"], "RECEPCAO TESTE");
}
