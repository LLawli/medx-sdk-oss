//! Escrita de notas do painel e de mensagens do chat interno.

mod common;

use common::fake_medx::FakeMedx;
use common::{call_error, call_json, connect_fake_with_write, fixtures};
use serde_json::json;

#[tokio::test]
async fn criar_nota_com_o_usuario_atual() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/security/getcurrentuser",
        fixtures::current_user(),
    )
    .json("POST", "/api/hoje/InsertNota", json!(15));
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "criar_nota",
        json!({ "texto": "Ligar para o laboratório" }),
    )
    .await;

    assert_eq!(result, json!({ "id": 15 }));
    let body = fake.requests_to("/api/hoje/InsertNota")[0].json();
    assert_eq!(body["Id"], 0);
    assert_eq!(body["Memo"], "Ligar para o laboratório");
    assert_eq!(body["IddoUsuario"], 7);
    assert_eq!(body["Concluida"], false);
}

#[tokio::test]
async fn editar_nota_concluir_mantem_o_texto() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/hoje/GetNotas",
        json!([fixtures::nota(5, "Ligar para o laboratório")]),
    )
    .on("PUT", "/api/hoje/UpdateNota", 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "editar_nota",
        json!({ "nota_id": 5, "concluida": true }),
    )
    .await;

    assert_eq!(result["id"], 5);
    let body = fake.requests_to("/api/hoje/UpdateNota")[0].json();
    assert_eq!(body["Id"], 5);
    assert_eq!(body["Memo"], "Ligar para o laboratório");
    assert_eq!(body["Concluida"], true);
    assert_eq!(body["IddoUsuario"], 7);
}

#[tokio::test]
async fn editar_nota_troca_o_texto() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/hoje/GetNotas",
        json!([fixtures::nota(5, "texto antigo")]),
    )
    .on("PUT", "/api/hoje/UpdateNota", 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    call_json(
        &client,
        "editar_nota",
        json!({ "nota_id": 5, "texto": "texto novo" }),
    )
    .await;

    let body = fake.requests_to("/api/hoje/UpdateNota")[0].json();
    assert_eq!(body["Memo"], "texto novo");
    assert_eq!(body["Concluida"], false);
}

#[tokio::test]
async fn editar_nota_que_nao_existe_e_erro() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/hoje/GetNotas",
        json!([fixtures::nota(6, "outra")]),
    );
    let client = connect_fake_with_write(&fake).await;

    let message = call_error(
        &client,
        "editar_nota",
        json!({ "nota_id": 5, "texto": "x" }),
    )
    .await;

    assert!(message.contains("não está"), "{message}");
}

#[tokio::test]
async fn criar_nota_sem_texto_e_erro() {
    let fake = FakeMedx::start();
    let client = connect_fake_with_write(&fake).await;

    let message = call_error(&client, "criar_nota", json!({ "texto": " " })).await;

    assert!(message.contains("texto"), "{message}");
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn enviar_mensagem_chat_como_o_usuario_atual() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/security/getcurrentuser",
        fixtures::current_user(),
    )
    .on("POST", "/api/chat/SendChatMessage", 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "enviar_mensagem_chat",
        json!({ "usuario_id": 2, "texto": "Paciente chegou" }),
    )
    .await;

    assert_eq!(result["enviada"], true);
    let body = fake.requests_to("/api/chat/SendChatMessage")[0].json();
    assert_eq!(body["De"], 7);
    assert_eq!(body["Para"], 2);
    assert_eq!(body["MessageText"], "Paciente chegou");
    assert_eq!(body["strDe"], "DRA ANA TESTE");
    assert_eq!(body["MessageId"], 0);
}

#[tokio::test]
async fn marcar_mensagens_lidas() {
    let fake = FakeMedx::start();
    fake.on("PUT", "/api/chat/UpdateLida", 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(&client, "marcar_mensagens_lidas", json!({ "ids": [1, 2] })).await;

    assert_eq!(result["ids"], json!([1, 2]));
    let body = fake.requests_to("/api/chat/UpdateLida")[0].json();
    assert_eq!(body, json!({ "Ids": [1, 2] }));
}

#[tokio::test]
async fn marcar_lidas_sem_ids_e_erro() {
    let fake = FakeMedx::start();
    let client = connect_fake_with_write(&fake).await;

    let message = call_error(&client, "marcar_mensagens_lidas", json!({ "ids": [] })).await;

    assert!(message.contains("ids"), "{message}");
    assert!(fake.requests().is_empty());
}
