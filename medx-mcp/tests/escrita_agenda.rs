//! Escrita na agenda: criar, bloquear, mudar status, remarcar, confirmar.

mod common;

use common::fake_medx::FakeMedx;
use common::{call_error, call_json, connect_fake_with_write, fixtures};
use serde_json::{Value, json};

const INSERT: &str = "/api/agenda/InsertAgendamento";
const CONTACT: &str = "/api/contatos/GetContatosFichaById";
const DAY: &str = "/api/hoje/GetAgendaDiaUsuario";

fn fake_with_patient() -> FakeMedx {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        CONTACT,
        json!([fixtures::contact(900, "PACIENTE TESTE")]),
    );
    fake
}

#[tokio::test]
async fn criar_agendamento_para_um_paciente() {
    let fake = fake_with_patient();
    fake.on("POST", INSERT, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "criar_agendamento",
        json!({
            "profissional_id": 42,
            "paciente_id": 900,
            "inicio": "2026-10-05T09:00",
            "fim": "2026-10-05T09:30"
        }),
    )
    .await;

    assert_eq!(result["criado"], true);
    let body = fake.requests_to(INSERT)[0].json();
    assert_eq!(body["Id_do_Usuario"], 42);
    assert_eq!(body["Vinculado_a"], 900);
    assert_eq!(body["Inicio"], "2026-10-05T09:00:00");
    assert_eq!(body["Final"], "2026-10-05T09:30:00");
    assert_eq!(body["Status"], 1);
    assert_eq!(body["Descricao"], "PACIENTE TESTE");
    assert_eq!(body["SMS"], "00000000000");
}

#[tokio::test]
async fn criar_agendamento_leva_descricao_procedimento_e_diagnostico() {
    let fake = fake_with_patient();
    fake.on("POST", INSERT, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    call_json(
        &client,
        "criar_agendamento",
        json!({
            "profissional_id": 42,
            "paciente_id": 900,
            "inicio": "2026-10-05T09:00:00",
            "fim": "2026-10-05T10:00:00",
            "descricao": "RETORNO",
            "procedimento_id": 10,
            "diagnostico_id": 5
        }),
    )
    .await;

    let body = fake.requests_to(INSERT)[0].json();
    assert_eq!(body["Descricao"], "RETORNO");
    assert_eq!(body["Id_do_Procedimento"], 10);
    assert_eq!(body["Id_do_Diagnostico_QP"], 5);
}

#[tokio::test]
async fn agendamento_recusado_pela_medx_vira_erro() {
    let fake = fake_with_patient();
    fake.json("POST", INSERT, json!([{ "StatusAgendado": false }]));
    let client = connect_fake_with_write(&fake).await;

    let message = call_error(
        &client,
        "criar_agendamento",
        json!({
            "profissional_id": 42,
            "paciente_id": 900,
            "inicio": "2026-10-05T09:00",
            "fim": "2026-10-05T09:30"
        }),
    )
    .await;

    assert!(message.contains("não criou"), "{message}");
}

#[tokio::test]
async fn criar_agendamento_valida_os_horarios() {
    let fake = fake_with_patient();
    let client = connect_fake_with_write(&fake).await;

    for (inicio, fim, campo) in [
        ("2026-10-05 09:00", "2026-10-05T09:30", "inicio"),
        ("2026-10-05T09:00", "2026-10-05T25:00", "fim"),
        ("2026-10-05T09:30", "2026-10-05T09:00", "fim"),
        ("2026-10-05T09:00", "2026-10-05T09:00", "fim"),
    ] {
        let message = call_error(
            &client,
            "criar_agendamento",
            json!({ "profissional_id": 42, "paciente_id": 900, "inicio": inicio, "fim": fim }),
        )
        .await;
        assert!(message.contains(campo), "{inicio} {fim}: {message}");
    }
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn bloquear_horario() {
    let fake = FakeMedx::start();
    fake.on("POST", INSERT, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "bloquear_horario",
        json!({ "profissional_id": 42, "inicio": "2026-10-05T12:00", "fim": "2026-10-05T13:00" }),
    )
    .await;

    assert_eq!(result["criado"], true);
    let body = fake.requests_to(INSERT)[0].json();
    assert_eq!(body["Id_do_Usuario"], 42);
    assert_eq!(body["Inicio"], "2026-10-05T12:00:00");
    assert!(body["Vinculado_a"].is_null(), "{body}");
}

#[tokio::test]
async fn mudar_status_agendamento() {
    let fake = FakeMedx::start();
    fake.on("PUT", "/api/agenda/updatestatus", 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "mudar_status_agendamento",
        json!({ "agendamento_id": 5, "status": 2 }),
    )
    .await;

    assert_eq!(result, json!({ "id": 5, "status": 2 }));
    let body = fake.requests_to("/api/agenda/updatestatus")[0].json();
    assert_eq!(body, json!({ "Id_do_Agendamento": 5, "Status": 2 }));
}

#[tokio::test]
async fn remarcar_agendamento_mantem_o_resto() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        DAY,
        json!([fixtures::appointment(5, 900, "CONSULTA")]),
    )
    .on("PUT", "/api/agenda/UpdateAgendamento", 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "remarcar_agendamento",
        json!({
            "agendamento_id": 5,
            "profissional_id": 42,
            "data": "2026-10-02",
            "novo_inicio": "2026-10-09T14:00",
            "novo_fim": "2026-10-09T14:30"
        }),
    )
    .await;

    assert_eq!(result["id"], 5);
    assert!(
        fake.requests_to(DAY)[0]
            .path
            .ends_with("?Id=42&Dt=2026-10-02")
    );
    let body: Value = fake.requests_to("/api/agenda/UpdateAgendamento")[0].json();
    assert_eq!(body["Id_do_Agendamento"], 5);
    assert_eq!(body["Inicio"], "2026-10-09T14:00:00");
    assert_eq!(body["Final"], "2026-10-09T14:30:00");
    assert_eq!(body["Id_do_Usuario"], 42);
    assert_eq!(body["Vinculado_a"], 900);
    assert_eq!(body["Descricao"], "CONSULTA");
}

#[tokio::test]
async fn remarcar_para_outro_profissional() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        DAY,
        json!([fixtures::appointment(5, 900, "CONSULTA")]),
    )
    .on("PUT", "/api/agenda/UpdateAgendamento", 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    call_json(
        &client,
        "remarcar_agendamento",
        json!({
            "agendamento_id": 5,
            "profissional_id": 42,
            "data": "2026-10-02",
            "novo_inicio": "2026-10-09T14:00",
            "novo_fim": "2026-10-09T14:30",
            "novo_profissional_id": 43
        }),
    )
    .await;

    let body: Value = fake.requests_to("/api/agenda/UpdateAgendamento")[0].json();
    assert_eq!(body["Id_do_Usuario"], 43);
}

#[tokio::test]
async fn remarcar_agendamento_que_nao_esta_no_dia_e_erro() {
    let fake = FakeMedx::start();
    fake.json("GET", DAY, json!([fixtures::appointment(6, 900, "OUTRO")]));
    let client = connect_fake_with_write(&fake).await;

    let message = call_error(
        &client,
        "remarcar_agendamento",
        json!({
            "agendamento_id": 5,
            "profissional_id": 42,
            "data": "2026-10-02",
            "novo_inicio": "2026-10-09T14:00",
            "novo_fim": "2026-10-09T14:30"
        }),
    )
    .await;

    assert!(message.contains("não está"), "{message}");
    assert!(fake.requests_to("/api/agenda/UpdateAgendamento").is_empty());
}

#[tokio::test]
async fn confirmar_agendamento_whatsapp() {
    let fake = FakeMedx::start();
    fake.on("POST", "/api/agenda/ConfirmaAgendamentoWhatsapp", 200, "");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "confirmar_agendamento_whatsapp",
        json!({ "agendamento_id": 5 }),
    )
    .await;

    assert_eq!(result["id"], 5);
    assert!(
        fake.requests_to("/api/agenda/ConfirmaAgendamentoWhatsapp")[0]
            .path
            .ends_with("?Iddoagendamento=5")
    );
}
