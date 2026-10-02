//! Ferramentas de marketing, configurações e ajustes da clínica.

mod common;

use common::fake_medx::FakeMedx;
use common::{call, call_json, connect_fake, fixtures, texts};
use serde_json::json;

#[tokio::test]
async fn listar_eventos_questionarios_e_diagnosticos() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/eventos/getAllEventos",
        json!([fixtures::evento(1, "Retorno")]),
    )
    .json(
        "GET",
        "/api/settings/XML_GetQuests",
        json!([fixtures::quest(10, "Satisfação")]),
    )
    .json(
        "GET",
        "/api/diagnosticoqp/GetAllDiagnosticoQP",
        json!([fixtures::diagnostico_qp(5, "Dor lombar")]),
    );
    let client = connect_fake(&fake).await;

    let eventos = call_json(&client, "listar_eventos", json!({})).await;
    assert_eq!(eventos[0]["name"], "Retorno");

    let quests = call_json(&client, "listar_questionarios", json!({})).await;
    assert_eq!(quests[0]["name"], "Satisfação");

    let diags = call_json(&client, "listar_diagnosticos_qp", json!({})).await;
    assert_eq!(diags[0]["id"], 5);
    assert_eq!(diags[0]["name"], "Dor lombar");
}

#[tokio::test]
async fn modelos_de_mensagem() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/marketing/GetClienteSettings",
        fixtures::cliente_settings(),
    );
    let client = connect_fake(&fake).await;

    let settings = call_json(&client, "modelos_de_mensagem", json!({})).await;

    assert_eq!(
        settings["whatsapp_template"],
        "Lembrete: consulta em {{DATA}}."
    );
    assert_eq!(settings["instagram"], "@clinicateste");
}

#[tokio::test]
async fn parametros_gerais_e_de_cores() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/settings/GetGeneralParameters",
        fixtures::general_parameters(),
    )
    .json(
        "GET",
        "/api/parametrosCores/getallparametroscores",
        fixtures::color_parameters(),
    );
    let client = connect_fake(&fake).await;

    let general = call_json(&client, "parametros_gerais", json!({})).await;
    assert_eq!(general["business_hours_start"], "0800");
    assert_eq!(general["timer_enabled"], true);

    let colors = call_json(&client, "parametros_de_cores", json!({})).await;
    assert_eq!(colors["labels"], json!(["DESMARCADO", "AGENDADO"]));
}

#[tokio::test]
async fn listar_relatorios_e_pastas_de_documentos() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/report/listarelatorios",
        json!([fixtures::relatorio(1, "Agenda")]),
    )
    .json(
        "GET",
        "/api/autodocs/getfoldersdocs",
        json!([fixtures::doc_folder(3, "Documentos do Paciente")]),
    );
    let client = connect_fake(&fake).await;

    let reports = call_json(&client, "listar_relatorios", json!({})).await;
    assert_eq!(reports[0]["name"], "Agenda");

    let folders = call_json(
        &client,
        "pastas_de_documentos",
        json!({ "filtro": "paciente" }),
    )
    .await;
    assert_eq!(folders[0]["filter_key"], "paciente");
    assert!(
        fake.requests_to("/api/autodocs/getfoldersdocs")[0]
            .path
            .ends_with("?filter=paciente")
    );

    call_json(&client, "pastas_de_documentos", json!({})).await;
    assert!(
        fake.requests_to("/api/autodocs/getfoldersdocs")[1]
            .path
            .ends_with("?filter=")
    );
}

#[tokio::test]
async fn calendario_ics_sem_o_token() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/localizadorICS/GetLocalizadorICS",
        fixtures::ics_config(),
    );
    let client = connect_fake(&fake).await;

    let result = call(&client, "calendario_ics", json!({})).await;
    let text = texts(&result).join("\n");
    let ics: serde_json::Value = serde_json::from_str(&texts(&result)[0]).unwrap();

    assert_eq!(ics["url"], "https://ics.example.invalid/agenda.ics");
    assert_eq!(ics["active"], true);
    assert!(ics.get("token").is_none(), "{text}");
    assert!(!text.contains("token-secreto-ics"), "{text}");
}
