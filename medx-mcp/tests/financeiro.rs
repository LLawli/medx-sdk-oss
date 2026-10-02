//! Ferramentas do financeiro (faturas de atendimento).

mod common;

use common::fake_medx::FakeMedx;
use common::{call_error, call_json, connect_fake, fixtures};
use serde_json::{Value, json};

const ALL: &str = "/api/atendimentos/GetAllAtendimentos";

fn ids(list: &Value) -> Vec<String> {
    list.as_array()
        .expect("lista")
        .iter()
        .map(|item| item["id"].as_str().expect("id").to_owned())
        .collect()
}

fn attendances() -> Value {
    json!([
        fixtures::attendance("1", "2026-01-10T09:00:00"),
        fixtures::attendance("3", "2026-09-30T09:00:00"),
        fixtures::attendance("2", "2026-05-02T09:00:00"),
    ])
}

#[tokio::test]
async fn atendimentos_do_paciente_mais_recentes_primeiro() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/atendimentos/GetAtendimentosByIdPac",
        attendances(),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(
        &client,
        "atendimentos_do_paciente",
        json!({ "paciente_id": 900 }),
    )
    .await;

    assert_eq!(ids(&list), ["3", "2", "1"]);
    assert_eq!(list[0]["invoice_value"], 150.0);
    assert_eq!(list[0]["total_paid"], 100.0);
    assert!(fake.requests()[0].path.ends_with("?Id=900"));
}

#[tokio::test]
async fn listar_atendimentos_sem_filtro() {
    let fake = FakeMedx::start();
    fake.json("GET", ALL, attendances());
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "listar_atendimentos", json!({})).await;

    assert_eq!(ids(&list), ["3", "2", "1"]);
    let path = &fake.requests_to(ALL)[0].path;
    assert!(path.contains("filterstring=&filter="), "{path}");
}

#[tokio::test]
async fn listar_atendimentos_com_filtro_e_busca() {
    let fake = FakeMedx::start();
    fake.json("GET", ALL, attendances());
    let client = connect_fake(&fake).await;

    for (filtro, esperado) in [
        ("pendencias", "Pend"),
        ("ultimos_7_dias", "7"),
        ("faturas_canceladas", "Canceladas"),
        ("orcamentos_em_aberto", "aberto"),
    ] {
        call_json(
            &client,
            "listar_atendimentos",
            json!({ "filtro": filtro, "busca": "teste" }),
        )
        .await;
        let path = fake.requests_to(ALL).last().unwrap().path.clone();
        let filter = path.split("filter=").nth(1).unwrap_or("");
        assert!(filter.contains(esperado), "{filtro}: {path}");
        assert!(path.contains("filterstring=teste"), "{path}");
    }
}

#[tokio::test]
async fn listar_atendimentos_com_filtro_desconhecido_e_erro() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    let message = call_error(&client, "listar_atendimentos", json!({ "filtro": "todos" })).await;

    // O erro de desserialização do rmcp lista as opções válidas.
    for opcao in [
        "ultimos_7_dias",
        "pendencias",
        "faturas_canceladas",
        "orcamentos_em_aberto",
    ] {
        assert!(message.contains(opcao), "{message}");
    }
    assert!(fake.requests().is_empty());
}
