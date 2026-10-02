//! Ferramentas do painel "hoje": notificações, últimos atendidos e notas.

mod common;

use common::fake_medx::FakeMedx;
use common::{call_json, connect_fake, fixtures};
use serde_json::json;

#[tokio::test]
async fn notificacoes_de_hoje() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/hoje/GetHojeNotificacoes",
        json!([fixtures::hoje_notificacao(
            1,
            "Aniversário de Paciente Teste"
        )]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "notificacoes_de_hoje", json!({})).await;

    assert_eq!(list[0]["id"], 1);
    assert_eq!(list[0]["message"], "Aniversário de Paciente Teste");
    assert_eq!(list[0]["tipo"], "Aniversario");
}

#[tokio::test]
async fn ultimos_atendidos() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/hoje/GetUltimosAtendidos",
        json!([fixtures::ultimo_atendido(900, "PACIENTE TESTE")]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "ultimos_atendidos", json!({})).await;

    assert_eq!(list[0]["patient_id"], 900);
    assert_eq!(list[0]["patient_name"], "PACIENTE TESTE");
}

#[tokio::test]
async fn listar_notas() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/hoje/GetNotas",
        json!([
            fixtures::nota(5, "Ligar para o laboratório"),
            fixtures::nota(6, "Repor estoque")
        ]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "listar_notas", json!({ "limite": 1 })).await;

    assert_eq!(list.as_array().map(Vec::len), Some(1));
    assert_eq!(list[0]["text"], "Ligar para o laboratório");
}
