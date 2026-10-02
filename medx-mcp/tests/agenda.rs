//! Ferramentas de agenda.

mod common;

use common::fake_medx::FakeMedx;
use common::{call_error, call_json, connect_fake, fixtures};
use serde_json::{Value, json};

const DAY: &str = "/api/hoje/GetAgendaDiaUsuario";
const PARAMS: &str = "/api/agenda/GetAllParametersAgenda";
const REPORT: &str = "/api/report/ReportAgenda";

fn ids(list: &Value) -> Vec<i64> {
    list.as_array()
        .expect("lista")
        .iter()
        .map(|item| item["id"].as_i64().expect("id"))
        .collect()
}

#[tokio::test]
async fn agenda_do_dia_consulta_o_profissional_e_a_data() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        DAY,
        json!([
            fixtures::appointment(1, 900, "CONSULTA"),
            fixtures::appointment(2, 0, "ALMOCO")
        ]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(
        &client,
        "agenda_do_dia",
        json!({ "profissional_id": 42, "data": "2026-10-02" }),
    )
    .await;

    assert_eq!(ids(&list), [1, 2]);
    assert_eq!(list[0]["description"], "CONSULTA");
    assert_eq!(list[0]["contact_id"], 900);
    assert_eq!(list[0]["start"], "2026-10-02T08:00:00");
    let request = &fake.requests_to(DAY)[0];
    assert_eq!(request.path, format!("{DAY}?Id=42&Dt=2026-10-02"));
}

#[tokio::test]
async fn bloqueios_do_dia_traz_so_os_horarios_sem_paciente() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        DAY,
        json!([
            fixtures::appointment(1, 900, "CONSULTA"),
            fixtures::appointment(2, 0, "ALMOCO")
        ]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(
        &client,
        "bloqueios_do_dia",
        json!({ "profissional_id": 42, "data": "2026-10-02" }),
    )
    .await;

    assert_eq!(ids(&list), [2]);
}

#[tokio::test]
async fn agenda_do_dia_com_data_invalida_nao_chama_a_medx() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    let message = call_error(
        &client,
        "agenda_do_dia",
        json!({ "profissional_id": 42, "data": "02/10/2026" }),
    )
    .await;

    assert!(message.contains("AAAA-MM-DD"), "{message}");
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn parametros_agenda_traz_rotulos_profissionais_setores_e_horarios() {
    let fake = FakeMedx::start();
    fake.json("GET", PARAMS, fixtures::agenda_parameters());
    let client = connect_fake(&fake).await;

    let params = call_json(&client, "parametros_agenda", json!({})).await;

    assert_eq!(params["color_labels"]["labels"][1], "AGENDADO");
    assert_eq!(params["users"][0]["id"], 42);
    assert_eq!(params["sectors"][0]["name"], "CONSULTORIO 1");
    assert_eq!(params["general"]["slot_duration"], "0030");
}

#[tokio::test]
async fn listar_profissionais_agenda_traz_so_quem_tem_agenda() {
    let fake = FakeMedx::start();
    fake.json("GET", PARAMS, fixtures::agenda_parameters());
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "listar_profissionais_agenda", json!({})).await;

    assert_eq!(ids(&list), [42]);
    assert_eq!(list[0]["username"], "DRA TESTE");
}

#[tokio::test]
async fn listar_setores_agenda() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/agenda/getagendasetores",
        json!([{ "Id_do_Setor": 1, "Setor": "CONSULTORIO 1" }, { "Id_do_Setor": 2, "Setor": "SALA 2" }]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "listar_setores_agenda", json!({})).await;

    assert_eq!(ids(&list), [1, 2]);
    assert_eq!(list[1]["name"], "SALA 2");
}

#[tokio::test]
async fn relatorio_agenda_usa_os_padroes_da_medx() {
    let fake = FakeMedx::start();
    fake.json(
        "POST",
        REPORT,
        fixtures::report("https://arquivos.example.invalid/agenda.pdf"),
    );
    let client = connect_fake(&fake).await;

    let report = call_json(
        &client,
        "relatorio_agenda",
        json!({ "inicio": "2026-10-01", "fim": "2026-10-31" }),
    )
    .await;

    assert_eq!(
        report["file_url"],
        "https://arquivos.example.invalid/agenda.pdf"
    );
    let body = fake.requests_to(REPORT)[0].json();
    assert_eq!(body["inicio"], "2026-10-01");
    assert_eq!(body["final"], "2026-10-31");
    assert_eq!(body["idUsuario"], 0);
    assert_eq!(body["Usuarioid"], json!([0]));
    assert_eq!(body["exibedesmarcados"], false);
    assert_eq!(body["textobusca"], "");
    assert_eq!(body["idStatus"], -1);
    assert_eq!(body["ExportaXLS"], false);
}

#[tokio::test]
async fn relatorio_agenda_leva_os_filtros() {
    let fake = FakeMedx::start();
    fake.json(
        "POST",
        REPORT,
        fixtures::report("https://arquivos.example.invalid/a.pdf"),
    );
    let client = connect_fake(&fake).await;

    call_json(
        &client,
        "relatorio_agenda",
        json!({
            "inicio": "2026-10-01",
            "fim": "2026-10-31",
            "profissional_id": 42,
            "incluir_desmarcados": true,
            "busca": "retorno",
            "status": 2
        }),
    )
    .await;

    let body = fake.requests_to(REPORT)[0].json();
    assert_eq!(body["idUsuario"], 42);
    assert_eq!(body["Usuarioid"], json!([42]));
    assert_eq!(body["exibedesmarcados"], true);
    assert_eq!(body["textobusca"], "retorno");
    assert_eq!(body["idStatus"], 2);
}

#[tokio::test]
async fn relatorio_agenda_com_periodo_invertido_nao_chama_a_medx() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    let message = call_error(
        &client,
        "relatorio_agenda",
        json!({ "inicio": "2026-10-31", "fim": "2026-10-01" }),
    )
    .await;

    assert!(message.contains("fim"), "{message}");
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn relatorio_faltas_de_todos_usa_o_relatorio_geral() {
    let fake = FakeMedx::start();
    fake.json(
        "POST",
        "/api/report/ReportAgendaNoShowGeral",
        fixtures::report("https://arquivos.example.invalid/faltas.pdf"),
    );
    let client = connect_fake(&fake).await;

    let report = call_json(
        &client,
        "relatorio_faltas",
        json!({ "inicio": "2026-10-01", "fim": "2026-10-31" }),
    )
    .await;

    assert_eq!(
        report["file_url"],
        "https://arquivos.example.invalid/faltas.pdf"
    );
    let body = fake.requests_to("/api/report/ReportAgendaNoShowGeral")[0].json();
    assert_eq!(body["idUsuario"], 0);
}

#[tokio::test]
async fn relatorio_faltas_de_um_profissional() {
    let fake = FakeMedx::start();
    fake.json(
        "POST",
        "/api/report/ReportAgendaNoShow",
        fixtures::report("https://arquivos.example.invalid/f.pdf"),
    );
    let client = connect_fake(&fake).await;

    call_json(
        &client,
        "relatorio_faltas",
        json!({ "inicio": "2026-10-01", "fim": "2026-10-31", "profissional_id": 42 }),
    )
    .await;

    let body = fake.requests_to("/api/report/ReportAgendaNoShow")[0].json();
    assert_eq!(body["idUsuario"], 42);
}
