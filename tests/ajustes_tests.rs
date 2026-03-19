//! Testes de integração — Etapa 11: Ajustes & Administração.
//! Execute com: `cargo test --test ajustes_tests -- --ignored`
//!
//! Endpoints cobertos:
//!   11.2  change_password(old, new)   → GET security/getkeys + POST usuarios/ChangeMePassword
//!   11.3  ics_config()               → GET localizadorICS/GetLocalizadorICS
//!   11.4  list_reports()             → GET report/listarelatorios
//!   11.5  doc_folders(filter)        → GET autodocs/getfoldersdocs?filter=

mod common;
use common::{shared_client, with_temp_dir};
use medx::MedxClient;

fn client() -> &'static MedxClient {
    shared_client()
}

// ── list_reports ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_list_reports_nao_panica() {
    with_temp_dir(|| {
        let _ = client().list_reports();
    });
}

// ── doc_folders ───────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_doc_folders_sem_filtro_nao_panica() {
    with_temp_dir(|| {
        // Endpoint pode retornar 500 dependendo da configuração da clínica — não deve panicar
        let _ = client().doc_folders("");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_doc_folders_com_filtro_nao_panica() {
    with_temp_dir(|| {
        // Busca com filtro qualquer — pode retornar lista vazia ou 500 sem erro
        let _ = client().doc_folders("paciente");
    });
}

// ── ics_config ────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_ics_config_nao_panica() {
    with_temp_dir(|| {
        // Pode retornar erro se a clínica não tem ICS configurado
        let _ = client().ics_config();
    });
}

// ── change_password ───────────────────────────────────────────────────────────

// NOTA: Não existe teste de integração automático para change_password pois
// alterar a senha real quebraria os demais testes. A função é testada apenas
// em nível unitário (fluxo de encriptação) e via inspeção manual.
#[test]
#[ignore = "requer rede e credenciais válidas — NÃO EXECUTAR: altera senha real"]
fn integration_change_password_placeholder() {
    // Placeholder intencional — testar manualmente se necessário.
}
