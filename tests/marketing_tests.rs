//! Testes de integração — Etapa 8: Marketing & Eventos.
//! Execute com: `cargo test --test marketing_tests -- --ignored`
//!
//! Endpoints cobertos:
//!   8.1  events()                            → GET  eventos/getAllEventos
//!   8.2  insert_quest(&dto)                  → POST marketing/InsertQuests
//!   8.3  quests()                            → GET  settings/XML_GetQuests
//!   8.4  diagnostico_qp()                    → GET  diagnosticoqp/GetAllDiagnosticoQP
//!   8.5  update_local_atendimento(&dto)      → POST marketing/UpdateLocalAtendimentoNomeClinica

mod common;
use common::{shared_client, with_temp_dir};
use medx::MedxClient;

fn client() -> &'static MedxClient {
    shared_client()
}

// ── events ────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_events_nao_panica() {
    with_temp_dir(|| {
        // IddoBoleto pode ser 0 para eventos sem boleto — não verificamos o id
        let _ = client().events();
    });
}

// ── quests ────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_quests_nao_panica() {
    with_temp_dir(|| {
        let _ = client().quests();
    });
}

// ── insert_quest ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_insert_quest_nao_panica() {
    with_temp_dir(|| {
        let c = client();
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let quests = c.quests().expect("quests falhou");
        if quests.is_empty() {
            return;
        }
        let contact = &contacts[0];
        let mut dto = medx::InsertQuestDto::new(contact.id, "", "", &contact.email, &contact.name);
        dto.add_quest(quests[0].id, &quests[0].name);
        // Pode retornar erro da API dependendo do estado — não deve panicar no SDK
        let _ = c.insert_quest(&dto);
    });
}

// ── diagnostico_qp ────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_diagnostico_qp_nao_panica() {
    with_temp_dir(|| {
        // IddoDiagnosticoQP pode ser 0 — não verificamos o id
        let _ = client().diagnostico_qp();
    });
}

// ── update_local_atendimento ──────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_update_local_atendimento_nao_panica() {
    with_temp_dir(|| {
        let c = client();
        let attendances = c.all_attendances("", "").expect("all_attendances falhou");
        if attendances.is_empty() {
            return;
        }
        let settings = c.client_settings().expect("client_settings falhou");
        let dto = medx::UpdateLocalAtendimentoDto::new(&attendances[0].id, &settings.logo);
        // Pode retornar erro se atendimento não suportar essa operação — não deve panicar
        let _ = c.update_local_atendimento(&dto);
    });
}

#[test]
#[ignore = "debug raw marketing"]
fn debug_marketing_raw() {
    with_temp_dir(|| {
        let c = client();
        let events_raw = c
            .get_text("agenda/GetEventos")
            .unwrap_or_else(|e| format!("ERR: {:?}", e));
        println!("EVENTS RAW: {}", &events_raw[..events_raw.len().min(400)]);
        let quests_raw = c
            .get_text("agenda/GetQuest")
            .unwrap_or_else(|e| format!("ERR: {:?}", e));
        println!("QUESTS RAW: {}", &quests_raw[..quests_raw.len().min(400)]);
        let diag_raw = c
            .get_text("agenda/GetDiagnosticoQP")
            .unwrap_or_else(|e| format!("ERR: {:?}", e));
        println!("DIAG RAW: {}", &diag_raw[..diag_raw.len().min(400)]);
    });
}

#[test]
#[ignore = "debug raw marketing correct endpoints"]
fn debug_marketing_raw_correct() {
    with_temp_dir(|| {
        let c = client();
        let events_raw = c
            .get_text("eventos/getAllEventos")
            .unwrap_or_else(|e| format!("ERR: {:?}", e));
        println!("EVENTS: {}", &events_raw[..events_raw.len().min(400)]);
        let quests_raw = c
            .get_text("settings/XML_GetQuests")
            .unwrap_or_else(|e| format!("ERR: {:?}", e));
        println!("QUESTS: {}", &quests_raw[..quests_raw.len().min(400)]);
        let diag_raw = c
            .get_text("diagnosticoqp/GetAllDiagnosticoQP")
            .unwrap_or_else(|e| format!("ERR: {:?}", e));
        println!("DIAG: {}", &diag_raw[..diag_raw.len().min(400)]);
    });
}
