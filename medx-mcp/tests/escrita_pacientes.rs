//! Escrita de pacientes: cadastro e atualização.

mod common;

use common::fake_medx::FakeMedx;
use common::{call_error, call_json, connect_fake_with_write, fixtures};
use serde_json::json;

const INSERT: &str = "/api/contatos/InsertContato";
const UPDATE: &str = "/api/contatos/UpdateContato";
const CONTACT: &str = "/api/contatos/GetContatosFichaById";

#[tokio::test]
async fn cadastrar_paciente_envia_o_cadastro() {
    let fake = FakeMedx::start();
    fake.on("POST", INSERT, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let created = call_json(
        &client,
        "cadastrar_paciente",
        json!({
            "nome": "PACIENTE NOVO",
            "sexo": "f",
            "nascimento": "1990-05-15",
            "celular": "11999990000",
            "email": "novo@example.invalid",
            "convenio_id": 2
        }),
    )
    .await;

    let id = created["id"].as_i64().expect("id do paciente criado");
    assert!(id > 0);
    let body = fake.requests_to(INSERT)[0].json();
    assert_eq!(body["Id_do_Cliente"], id);
    assert_eq!(body["Nome"], "PACIENTE NOVO");
    assert_eq!(body["Sexo"], "F");
    assert_eq!(body["Nascimento"], "1990-05-15T00:00:00");
    assert_eq!(body["Celular"], "11999990000");
    assert_eq!(body["Email"], "novo@example.invalid");
    assert_eq!(body["Id_do_Convenio"], 2);
    assert_eq!(body["Tipo"], "Paciente");
}

#[tokio::test]
async fn cadastrar_paciente_valida_antes_de_enviar() {
    let fake = FakeMedx::start();
    let client = connect_fake_with_write(&fake).await;

    for (args, campo) in [
        (json!({ "nome": "  " }), "nome"),
        (json!({ "nome": "A", "sexo": "X" }), "sexo"),
        (
            json!({ "nome": "A", "nascimento": "15/05/1990" }),
            "nascimento",
        ),
    ] {
        let message = call_error(&client, "cadastrar_paciente", args).await;
        assert!(message.contains(campo), "{campo}: {message}");
    }
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn atualizar_paciente_muda_so_o_informado() {
    let fake = FakeMedx::start();
    fake.json(
        "GET",
        CONTACT,
        json!([fixtures::contact(900, "PACIENTE TESTE")]),
    )
    .on("PUT", UPDATE, 200, "\"Success\"");
    let client = connect_fake_with_write(&fake).await;

    let updated = call_json(
        &client,
        "atualizar_paciente",
        json!({ "paciente_id": 900, "celular": "11999990000", "observacoes": "prefere manhã" }),
    )
    .await;

    assert_eq!(updated["id"], 900);
    let body = fake.requests_to(UPDATE)[0].json();
    assert_eq!(body["Id_do_Cliente"], 900);
    assert_eq!(body["Celular"], "11999990000");
    assert_eq!(body["Observacoes"], "prefere manhã");
    assert_eq!(body["Nome"], "PACIENTE TESTE");
    assert_eq!(body["Email"], "paciente@example.invalid");
    assert_eq!(body["Cidade_Residencial"], "CIDADE TESTE");
}

#[tokio::test]
async fn atualizar_paciente_sem_nenhum_campo_e_erro() {
    let fake = FakeMedx::start();
    let client = connect_fake_with_write(&fake).await;

    let message = call_error(&client, "atualizar_paciente", json!({ "paciente_id": 900 })).await;

    assert!(message.contains("nenhum campo"), "{message}");
    assert!(fake.requests().is_empty());
}
