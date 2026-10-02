//! Escrita no prontuário: registrar, editar registro e atualizar o sumário.

mod common;

use common::fake_medx::FakeMedx;
use common::{call_error, call_json, connect_fake_with_write, fixtures};
use serde_json::json;

const INSERT: &str = "/api/prontuario/InsertMedicalHistory";
const UPDATE: &str = "/api/prontuario/UpdateMedicalHistory";
const RECORDS: &str = "/api/prontuario/GetProntuario";
const SUMMARY: &str = "/api/prontuario/GetMedicalHistorySummary";
const UPSERT_SUMMARY: &str = "/api/prontuario/InsertOrUpdateMedicalHistorySummary";

fn fake_with_user() -> FakeMedx {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/security/getcurrentuser",
        fixtures::current_user(),
    );
    fake
}

#[tokio::test]
async fn registrar_no_prontuario_em_html_e_com_o_usuario_atual() {
    let fake = fake_with_user();
    fake.on("POST", INSERT, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "registrar_no_prontuario",
        json!({
            "paciente_id": 900,
            "texto": "Paciente estável.\nRetorno em 30 dias.",
            "data": "2026-10-02T10:00",
            "palavras_chave": "retorno"
        }),
    )
    .await;

    assert_eq!(result["paciente_id"], 900);
    let body = fake.requests_to(INSERT)[0].json();
    assert_eq!(body["Id_do_Cliente"], 900);
    assert_eq!(body["Id_do_Usuario"], 7);
    assert_eq!(
        body["Historico"],
        "<p>Paciente estável.<br>Retorno em 30 dias.</p>"
    );
    assert_eq!(body["Data"], "2026-10-02T10:00:00");
    assert_eq!(body["Palavraschave"], "retorno");
}

#[tokio::test]
async fn registrar_sem_data_usa_agora() {
    let fake = fake_with_user();
    fake.on("POST", INSERT, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    call_json(
        &client,
        "registrar_no_prontuario",
        json!({ "paciente_id": 900, "texto": "Consulta de rotina." }),
    )
    .await;

    let body = fake.requests_to(INSERT)[0].json();
    let data = body["Data"].as_str().expect("Data");
    assert_eq!(data.len(), 19, "{data}");
    assert_eq!(&data[10..11], "T", "{data}");
}

#[tokio::test]
async fn registrar_valida_texto_e_data() {
    let fake = fake_with_user();
    let client = connect_fake_with_write(&fake).await;

    let texto = call_error(
        &client,
        "registrar_no_prontuario",
        json!({ "paciente_id": 900, "texto": "  " }),
    )
    .await;
    assert!(texto.contains("texto"), "{texto}");

    let data = call_error(
        &client,
        "registrar_no_prontuario",
        json!({ "paciente_id": 900, "texto": "ok", "data": "02/10/2026" }),
    )
    .await;
    assert!(data.contains("data"), "{data}");
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn editar_registro_troca_so_o_texto() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        RECORDS,
        json!([fixtures::medical_record(
            3,
            "2026-09-30T09:00:00",
            "texto antigo"
        )]),
    )
    .on("PUT", UPDATE, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "editar_registro_prontuario",
        json!({ "paciente_id": 900, "registro_id": 3, "texto": "texto corrigido" }),
    )
    .await;

    assert_eq!(result["id"], 3);
    let body = fake.requests_to(UPDATE)[0].json();
    assert_eq!(body["Id_do_Historico"], 3);
    assert_eq!(body["Historico"], "<p>texto corrigido</p>");
    assert_eq!(body["Data"], "2026-09-30T09:00:00");
    assert_eq!(body["Id_do_Usuario"], 7);
    assert_eq!(body["Palavraschave"], "rotina");
}

#[tokio::test]
async fn editar_registro_que_nao_existe_e_erro() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        RECORDS,
        json!([fixtures::medical_record(3, "2026-09-30T09:00:00", "texto")]),
    );
    let client = connect_fake_with_write(&fake).await;

    let message = call_error(
        &client,
        "editar_registro_prontuario",
        json!({ "paciente_id": 900, "registro_id": 4, "texto": "x" }),
    )
    .await;

    assert!(message.contains("não está"), "{message}");
    assert!(fake.requests_to(UPDATE).is_empty());
}

#[tokio::test]
async fn atualizar_sumario_muda_so_o_informado() {
    let fake = FakeMedx::start();
    fake.json("GET", SUMMARY, fixtures::medical_summary()).on(
        "POST",
        UPSERT_SUMMARY,
        200,
        "\"Success\"",
    );
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "atualizar_sumario_prontuario",
        json!({ "paciente_id": 900, "alergias": "Dipirona" }),
    )
    .await;

    assert_eq!(result["paciente_id"], 900);
    let request = &fake.requests_to(UPSERT_SUMMARY)[0];
    assert!(request.path.ends_with("?intPacid=900"), "{}", request.path);
    let body = request.json();
    assert_eq!(body["alergias"], "Dipirona");
    assert_eq!(body["diagnostico"], "Diagnóstico de teste");
    assert_eq!(body["medicamentos"], "Nenhum");
}

#[tokio::test]
async fn atualizar_sumario_sem_campo_e_erro() {
    let fake = FakeMedx::start();
    let client = connect_fake_with_write(&fake).await;

    let message = call_error(
        &client,
        "atualizar_sumario_prontuario",
        json!({ "paciente_id": 900 }),
    )
    .await;

    assert!(message.contains("nenhum campo"), "{message}");
    assert!(fake.requests().is_empty());
}
