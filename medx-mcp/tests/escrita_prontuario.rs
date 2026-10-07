//! Escrita no prontuário: registrar, editar registro, atualizar o sumário e
//! anexar arquivo.

mod common;

use common::fake_medx::FakeMedx;
use common::{call_error, call_json, connect_fake_with_write, fixtures};
use serde_json::json;
use std::path::{Path, PathBuf};

const INSERT: &str = "/api/prontuario/InsertMedicalHistory";
const UPDATE: &str = "/api/prontuario/UpdateMedicalHistory";
const RECORDS: &str = "/api/prontuario/GetProntuario";
const SUMMARY: &str = "/api/prontuario/GetMedicalHistorySummary";
const UPSERT_SUMMARY: &str = "/api/prontuario/InsertOrUpdateMedicalHistorySummary";
const ATTACH: &str = "/api/prontuario/AttachFiles";

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

/// Grava `content` em `name` dentro de `dir` e devolve o caminho absoluto.
fn write_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).expect("grava o arquivo de teste");
    path
}

fn path_arg(path: &Path) -> String {
    path.to_str().expect("caminho UTF-8").to_owned()
}

#[tokio::test]
async fn anexar_envia_o_arquivo_em_base64_com_nome_tipo_e_descricao() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_file(dir.path(), "exame.pdf", b"PDF sintetico");
    let fake = FakeMedx::start();
    fake.on("POST", ATTACH, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "anexar_ao_prontuario",
        json!({
            "paciente_id": 900,
            "arquivo": path_arg(&path),
            "descricao": "Hemograma de setembro"
        }),
    )
    .await;

    assert_eq!(result["paciente_id"], 900);
    assert_eq!(result["arquivo"], "exame.pdf");
    assert_eq!(result["tamanho"], 13);
    let requests = fake.requests_to(ATTACH);
    assert_eq!(requests.len(), 1);
    let body = requests[0].json();
    assert_eq!(body["Id_do_Cliente"], 900);
    assert_eq!(body["Descricao"], "Hemograma de setembro");
    let arquivos = body["arquivos"].as_array().expect("arquivos");
    assert_eq!(arquivos.len(), 1);
    assert_eq!(arquivos[0]["filename"], "exame.pdf");
    assert_eq!(arquivos[0]["filetype"], "application/pdf");
    assert_eq!(arquivos[0]["filesize"], 13);
    assert_eq!(arquivos[0]["file_base64"], "UERGIHNpbnRldGljbw==");
}

#[tokio::test]
async fn anexar_extensao_desconhecida_vai_como_octet_stream() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_file(dir.path(), "laudo.dcm", b"abc");
    let fake = FakeMedx::start();
    fake.on("POST", ATTACH, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    call_json(
        &client,
        "anexar_ao_prontuario",
        json!({ "paciente_id": 900, "arquivo": path_arg(&path), "descricao": "Laudo" }),
    )
    .await;

    let body = fake.requests_to(ATTACH)[0].json();
    assert_eq!(body["arquivos"][0]["filename"], "laudo.dcm");
    assert_eq!(body["arquivos"][0]["filetype"], "application/octet-stream");
    assert_eq!(body["arquivos"][0]["file_base64"], "YWJj");
}

/// Acima do limite de corpo, a MedX responde 200 com o texto de uma
/// exceção .NET em vez de "Success" (medido em 2026-10-07).
#[tokio::test]
async fn anexar_resposta_diferente_de_success_e_erro() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_file(dir.path(), "exame.pdf", b"PDF sintetico");
    let fake = FakeMedx::start();
    fake.on(
        "POST",
        ATTACH,
        200,
        "\"Object reference not set to an instance of an object.\"",
    );
    let client = connect_fake_with_write(&fake).await;

    let message = call_error(
        &client,
        "anexar_ao_prontuario",
        json!({ "paciente_id": 900, "arquivo": path_arg(&path), "descricao": "Exame" }),
    )
    .await;

    assert!(message.contains("não anexou"), "{message}");
    assert!(message.contains("Object reference"), "{message}");
}

#[tokio::test]
async fn anexar_valida_antes_de_chamar_a_medx() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ok = write_file(dir.path(), "exame.pdf", b"PDF sintetico");
    let empty = write_file(dir.path(), "vazio.pdf", b"");
    let big = dir.path().join("grande.pdf");
    std::fs::File::create(&big)
        .and_then(|file| file.set_len(22_000_001))
        .expect("arquivo esparso");
    let fake = FakeMedx::start();
    let client = connect_fake_with_write(&fake).await;

    let cases = [
        (
            json!({ "paciente_id": 900, "arquivo": path_arg(&ok), "descricao": "  " }),
            "descricao",
        ),
        (
            json!({ "paciente_id": 900, "arquivo": "exame.pdf", "descricao": "Exame" }),
            "absoluto",
        ),
        (
            json!({
                "paciente_id": 900,
                "arquivo": path_arg(&dir.path().join("nao-existe.pdf")),
                "descricao": "Exame"
            }),
            "nao-existe.pdf",
        ),
        (
            json!({ "paciente_id": 900, "arquivo": path_arg(dir.path()), "descricao": "Exame" }),
            "não é um arquivo",
        ),
        (
            json!({ "paciente_id": 900, "arquivo": path_arg(&empty), "descricao": "Exame" }),
            "vazio",
        ),
        (
            json!({ "paciente_id": 900, "arquivo": path_arg(&big), "descricao": "Exame" }),
            "22.000.000",
        ),
    ];
    for (arguments, expected) in cases {
        let message = call_error(&client, "anexar_ao_prontuario", arguments.clone()).await;
        assert!(message.contains(expected), "{arguments}: {message}");
    }
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn anexar_no_limite_exato_passa() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("limite.pdf");
    std::fs::File::create(&path)
        .and_then(|file| file.set_len(22_000_000))
        .expect("arquivo esparso");
    let fake = FakeMedx::start();
    fake.on("POST", ATTACH, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let result = call_json(
        &client,
        "anexar_ao_prontuario",
        json!({ "paciente_id": 900, "arquivo": path_arg(&path), "descricao": "Exame" }),
    )
    .await;

    assert_eq!(result["tamanho"], 22_000_000);
    let body = fake.requests_to(ATTACH)[0].json();
    assert_eq!(body["arquivos"][0]["filesize"], 22_000_000);
}
