//! Ferramentas de pacientes (contatos da MedX).

mod common;

use common::fake_medx::FakeMedx;
use common::{call, call_error, call_json, connect_fake, fixtures, texts};
use serde_json::json;

const SEARCH: &str = "/api/contatos/GetContatosGridBySearch";
const CONTACT: &str = "/api/contatos/GetContatosFichaById";
const PHOTO: &str = "/api/contatos/GetFotoBase64";
const HOMONYMS: &str = "/api/contatos/GetContatosHomonimos";

#[tokio::test]
async fn buscar_pacientes_pelo_nome() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        SEARCH,
        json!([
            fixtures::contact_summary(1, "MARIA SILVA"),
            fixtures::contact_summary(2, "MARIA SOUZA")
        ]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "buscar_pacientes", json!({ "nome": "maria s" })).await;

    assert_eq!(list[0]["id"], 1);
    assert_eq!(list[1]["name"], "MARIA SOUZA");
    assert_eq!(list[0]["insurance_name"], "PARTICULAR");
    let path = &fake.requests_to(SEARCH)[0].path;
    assert!(path.contains("Group=1&GroupValue=1"), "{path}");
    assert!(path.contains("Name=maria+s"), "{path}");
}

#[tokio::test]
async fn buscar_aniversariantes_do_mes() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        SEARCH,
        json!([fixtures::contact_summary(1, "MARIA SILVA")]),
    );
    let client = connect_fake(&fake).await;

    call_json(
        &client,
        "buscar_pacientes",
        json!({ "aniversariantes_do_mes": true }),
    )
    .await;

    let path = &fake.requests_to(SEARCH)[0].path;
    assert!(path.contains("Group=2"), "{path}");
}

#[tokio::test]
async fn buscar_pacientes_sem_nome_e_erro() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    for args in [json!({}), json!({ "nome": "   " })] {
        let message = call_error(&client, "buscar_pacientes", args).await;
        assert!(message.contains("nome"), "{message}");
    }
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn busca_sem_resultado_e_lista_vazia() {
    let fake = FakeMedx::start();
    fake.on("GET", SEARCH, 200, "null");
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "buscar_pacientes", json!({ "nome": "ninguem" })).await;

    assert_eq!(list, json!([]));
}

#[tokio::test]
async fn ver_paciente_traz_a_ficha() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        CONTACT,
        json!([fixtures::contact(900, "PACIENTE TESTE")]),
    );
    let client = connect_fake(&fake).await;

    let patient = call_json(&client, "ver_paciente", json!({ "paciente_id": 900 })).await;

    assert_eq!(patient["id"], 900);
    assert_eq!(patient["name"], "PACIENTE TESTE");
    assert_eq!(patient["mobile"], "00000000000");
    assert_eq!(
        fake.requests_to(CONTACT)[0].path,
        format!("{CONTACT}?Id=900")
    );
}

#[tokio::test]
async fn paciente_inexistente_e_erro() {
    let fake = FakeMedx::start();
    fake.on("GET", CONTACT, 200, "[]");
    let client = connect_fake(&fake).await;

    let message = call_error(&client, "ver_paciente", json!({ "paciente_id": 1 })).await;

    assert!(message.contains("não encontrado"), "{message}");
}

#[tokio::test]
async fn pacientes_homonimos() {
    let fake = FakeMedx::start();
    fake.json("GET", HOMONYMS, json!([fixtures::homonym(1, "ANA TESTE")]));
    let client = connect_fake(&fake).await;

    let list = call_json(
        &client,
        "pacientes_homonimos",
        json!({ "nome": "ANA TESTE", "sexo": "f", "nascimento": "1990-05-15" }),
    )
    .await;

    assert_eq!(list[0]["id"], 1);
    let path = &fake.requests_to(HOMONYMS)[0].path;
    assert!(path.contains("Gender=F"), "{path}");
    assert!(path.contains("Birth=1990-05-15"), "{path}");
}

#[tokio::test]
async fn pacientes_homonimos_valida_sexo_e_nascimento() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    let sexo = call_error(
        &client,
        "pacientes_homonimos",
        json!({ "nome": "ANA", "sexo": "X", "nascimento": "1990-05-15" }),
    )
    .await;
    assert!(sexo.contains("sexo"), "{sexo}");

    let nascimento = call_error(
        &client,
        "pacientes_homonimos",
        json!({ "nome": "ANA", "sexo": "F", "nascimento": "15/05/1990" }),
    )
    .await;
    assert!(nascimento.contains("nascimento"), "{nascimento}");
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn listar_planos_de_saude() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        "/api/contatos/GetContatosConvenios",
        json!([
            fixtures::insurance_plan(1, "PARTICULAR"),
            fixtures::insurance_plan(2, "PLANO TESTE")
        ]),
    );
    let client = connect_fake(&fake).await;

    let list = call_json(&client, "listar_planos_de_saude", json!({})).await;

    assert_eq!(list[1]["id"], 2);
    assert_eq!(list[1]["name"], "PLANO TESTE");
}

// ── Foto ──────────────────────────────────────────────────────────────────────

async fn photo(body: &str) -> rmcp::model::CallToolResult {
    let fake = FakeMedx::start();
    fake.on("GET", PHOTO, 200, body);
    let client = connect_fake(&fake).await;
    call(&client, "foto_paciente", json!({ "paciente_id": 900 })).await
}

fn image(result: &rmcp::model::CallToolResult) -> (String, String) {
    assert_ne!(result.is_error, Some(true), "{result:?}");
    let image = result
        .content
        .iter()
        .find_map(|block| block.as_image())
        .unwrap_or_else(|| panic!("sem bloco de imagem: {result:?}"));
    (image.mime_type.clone(), image.data.clone())
}

#[tokio::test]
async fn foto_jpeg_em_base64_cru() {
    let (mime, data) = image(&photo("/9j/4AAQSkZJRgABAQ==").await);
    assert_eq!(mime, "image/jpeg");
    assert_eq!(data, "/9j/4AAQSkZJRgABAQ==");
}

#[tokio::test]
async fn foto_png_entre_aspas_json() {
    let (mime, data) = image(&photo("\"iVBORw0KGgoAAAANSUhEUg==\"").await);
    assert_eq!(mime, "image/png");
    assert_eq!(data, "iVBORw0KGgoAAAANSUhEUg==");
}

#[tokio::test]
async fn foto_em_data_url() {
    let (mime, data) = image(&photo("data:image/webp;base64,UklGRiQAAABXRUJQ").await);
    assert_eq!(mime, "image/webp");
    assert_eq!(data, "UklGRiQAAABXRUJQ");
}

#[tokio::test]
async fn paciente_sem_foto_e_texto_e_nao_erro() {
    for body in ["", "\"\""] {
        let result = photo(body).await;
        assert_ne!(result.is_error, Some(true), "{result:?}");
        assert!(result.content.iter().all(|b| b.as_image().is_none()));
        let text = texts(&result).join("\n");
        assert!(text.contains("não tem foto"), "{text}");
    }
}
