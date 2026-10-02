//! Ferramentas de prontuário, convênios, procedimentos e formulários.

mod common;

use common::fake_medx::FakeMedx;
use common::{call, call_error, call_json, connect_fake, fixtures, texts};
use serde_json::{Value, json};

const RECORDS: &str = "/api/prontuario/GetProntuario";

fn ids(list: &Value) -> Vec<i64> {
    list.as_array()
        .expect("lista")
        .iter()
        .map(|item| item["id"].as_i64().expect("id"))
        .collect()
}

fn records() -> Value {
    json!([
        fixtures::medical_record(1, "2026-01-10T09:00:00", "primeira consulta"),
        fixtures::medical_record(3, "2026-09-30T09:00:00", "retorno recente"),
        fixtures::medical_record(2, "2026-05-02T09:00:00", "retorno"),
    ])
}

#[tokio::test]
async fn sumario_prontuario() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/prontuario/GetMedicalHistorySummary",
        fixtures::medical_summary(),
    );
    let client = connect_fake(&fake).await;

    let summary = call_json(&client, "sumario_prontuario", json!({ "paciente_id": 900 })).await;

    assert_eq!(summary["alergias"], "Nenhuma conhecida");
    let path = &fake.requests()[0].path;
    assert!(path.ends_with("?Pacid=900"), "{path}");
}

#[tokio::test]
async fn ver_prontuario_traz_os_mais_recentes_primeiro() {
    let fake = FakeMedx::start();
    fake.json("GET", RECORDS, records());
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "ver_prontuario", json!({ "paciente_id": 900 })).await;

    assert_eq!(ids(&list), [3, 2, 1]);
    assert_eq!(list[0]["content"], "<p>retorno recente</p>");
    assert_eq!(
        fake.requests_to(RECORDS)[0].path,
        format!("{RECORDS}?PacId=900")
    );
}

#[tokio::test]
async fn ver_prontuario_corta_depois_de_ordenar() {
    let fake = FakeMedx::start();
    fake.json("GET", RECORDS, records());
    let client = connect_fake(&fake).await;

    let result = call(
        &client,
        "ver_prontuario",
        json!({ "paciente_id": 900, "limite": 1 }),
    )
    .await;
    let blocks = texts(&result);
    let list: Value = serde_json::from_str(&blocks[0]).unwrap();

    assert_eq!(ids(&list), [3]);
    assert!(blocks[1].contains("1 de 3"), "{}", blocks[1]);
}

#[tokio::test]
async fn galeria_de_fotos_mais_recentes_primeiro() {
    let fake = FakeMedx::start();
    fake.json("GET", RECORDS, records());
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "galeria_de_fotos", json!({ "paciente_id": 900 })).await;

    assert_eq!(ids(&list), [3, 2, 1]);
    assert_eq!(
        fake.requests_to(RECORDS)[0].path,
        format!("{RECORDS}?PacId=900&galeria=true")
    );
}

#[tokio::test]
async fn buscar_no_prontuario() {
    let fake = FakeMedx::start();
    fake.json("GET", "/api/prontuario/GetProntuarioBusca", records());
    let client = connect_fake(&fake).await;

    let list = call_json(
        &client,
        "buscar_no_prontuario",
        json!({ "paciente_id": 900, "texto": "retorno" }),
    )
    .await;

    assert_eq!(ids(&list), [3, 2, 1]);
    let path = &fake.requests()[0].path;
    assert!(path.contains("PacId=900&busca=retorno"), "{path}");
}

#[tokio::test]
async fn buscar_no_prontuario_sem_texto_e_erro() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    let message = call_error(
        &client,
        "buscar_no_prontuario",
        json!({ "paciente_id": 900, "texto": " " }),
    )
    .await;

    assert!(message.contains("texto"), "{message}");
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn link_do_arquivo_do_prontuario() {
    let fake = FakeMedx::start();
    fake.on(
        "GET",
        "/api/azure/getfileurl",
        200,
        "\"https://medxdata.blob.core.windows.net/c/1-abc.pdf?sig=x\"",
    );
    let client = connect_fake(&fake).await;

    let link = call_json(
        &client,
        "link_arquivo_prontuario",
        json!({ "classe": "1-abc.pdf" }),
    )
    .await;

    assert_eq!(
        link["url"],
        "https://medxdata.blob.core.windows.net/c/1-abc.pdf?sig=x"
    );
    assert!(fake.requests()[0].path.ends_with("?blobname=1-abc.pdf"));
}

#[tokio::test]
async fn link_do_arquivo_sem_classe_e_erro() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    let message = call_error(&client, "link_arquivo_prontuario", json!({ "classe": "" })).await;

    assert!(message.contains("classe"), "{message}");
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn palavras_chave_do_prontuario_viram_lista() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/prontuario/GetMedicalKeywords",
        json!([{ "Keywords": "rotina, retorno,urgência" }]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "palavras_chave_prontuario", json!({})).await;

    assert_eq!(list, json!(["rotina", "retorno", "urgência"]));
}

#[tokio::test]
async fn listar_convenios_e_procedimentos() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/Convenios/GetAllConvenios",
        json!([fixtures::convenio(1, "PLANO TESTE")]),
    )
    .json(
        "GET",
        "/api/Convenios/GetProcedimentosByIdConvenio",
        json!([fixtures::convenio_procedure(10, "CONSULTA")]),
    )
    .json(
        "GET",
        "/api/Procedimentos/GetAllProcedimentos",
        json!([
            fixtures::procedure(10, "CONSULTA"),
            fixtures::procedure(11, "RETORNO")
        ]),
    );
    let client = connect_fake(&fake).await;

    let convenios = call_json(&client, "listar_convenios", json!({})).await;
    assert_eq!(convenios[0]["name"], "PLANO TESTE");

    let by_plan = call_json(
        &client,
        "procedimentos_do_convenio",
        json!({ "convenio_id": 1 }),
    )
    .await;
    assert_eq!(by_plan[0]["price"], 200.0);
    assert!(
        fake.requests_to("/api/Convenios/GetProcedimentosByIdConvenio")[0]
            .path
            .ends_with("?IddoConvenio=1")
    );

    let procedures = call_json(&client, "listar_procedimentos", json!({})).await;
    assert_eq!(ids(&procedures), [10, 11]);
    assert_eq!(procedures[1]["base_price"], 150.0);
}

#[tokio::test]
async fn formularios() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/formularios/getformularios",
        json!([fixtures::form(5, "anamnese")]),
    )
    .on(
        "GET",
        "/api/formularios/GetFormulariosHTML",
        200,
        "<form><label>Queixa</label></form>",
    );
    let client = connect_fake(&fake).await;

    let forms = call_json(&client, "listar_formularios", json!({})).await;
    assert_eq!(forms[0]["name"], "anamnese");

    let result = call(&client, "ver_formulario", json!({ "formulario_id": 5 })).await;
    assert_ne!(result.is_error, Some(true), "{result:?}");
    assert_eq!(texts(&result), ["<form><label>Queixa</label></form>"]);
    assert!(
        fake.requests_to("/api/formularios/GetFormulariosHTML")[0]
            .path
            .ends_with("?iddoformulario=5")
    );
}

#[tokio::test]
async fn registros_do_modulo() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/modulos/GetRecords",
        json!([fixtures::module_record(1, "anamnese")]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(
        &client,
        "registros_do_modulo",
        json!({ "paciente_id": 900, "modulo": "anamnese" }),
    )
    .await;

    assert_eq!(list[0]["module"], "anamnese");
    assert!(
        fake.requests()[0]
            .path
            .ends_with("?pacid=900&modulo=anamnese")
    );
}

#[tokio::test]
async fn relatorio_prontuario_busca_o_nome_do_paciente() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/contatos/GetContatosFichaById",
        json!([fixtures::contact(900, "PACIENTE TESTE")]),
    )
    .json(
        "POST",
        "/api/report/reportprontuario",
        fixtures::report("https://arquivos.example.invalid/prontuario.pdf"),
    );
    let client = connect_fake(&fake).await;

    let report = call_json(
        &client,
        "relatorio_prontuario",
        json!({ "paciente_id": 900, "inicio": "2026-01-01", "fim": "2026-10-02" }),
    )
    .await;

    assert_eq!(
        report["file_url"],
        "https://arquivos.example.invalid/prontuario.pdf"
    );
    let body = fake.requests_to("/api/report/reportprontuario")[0].json();
    assert_eq!(body["Iddopaciente"], 900);
    assert_eq!(body["Paciente"], "PACIENTE TESTE");
    assert_eq!(body["DataInicial"], "2026-01-01");
    assert_eq!(body["DataFinal"], "2026-10-02");
}

#[tokio::test]
async fn relatorio_prontuario_valida_o_periodo() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    let message = call_error(
        &client,
        "relatorio_prontuario",
        json!({ "paciente_id": 900, "inicio": "2026-10-02", "fim": "2026-01-01" }),
    )
    .await;

    assert!(message.contains("fim"), "{message}");
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn listar_unidades() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/UN/GetAllUN",
        json!([fixtures::business_unit(1001, "CLINICA TESTE")]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "listar_unidades", json!({})).await;

    assert_eq!(list[0]["id"], 1001);
    assert_eq!(list[0]["name"], "CLINICA TESTE");
}
