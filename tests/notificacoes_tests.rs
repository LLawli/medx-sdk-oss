//! Testes de integração — Etapa 7: Notificações & Comunicação (escopo reduzido).
//! Execute com: `cargo test --test notificacoes_tests -- --ignored`
//!
//! Escopo implementado:
//!   - 7.1  client_settings()  → GET marketing/GetClienteSettings
//!   - 7.2  log_email(&dto)    → POST notifications/InsertMailLogger
//!
//! Descartados por decisão de design: SMS pacientes, WhatsApp pacientes.

mod common;
use common::{shared_client, with_temp_dir};
use medx::MedxClient;

fn client() -> &'static MedxClient { shared_client() }



// ── client_settings ───────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_client_settings_nao_panica() {
    with_temp_dir(|| {
        let _ = client().client_settings();
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_client_settings_helpers_nao_panicam() {
    with_temp_dir(|| {
        let _ = client().client_settings();
    });
}

// ── log_email ─────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_log_email_nao_panica() {
    with_temp_dir(|| {
        let c = client();
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        // Sem token reCAPTCHA a API tende a recusar; o teste só garante não-pânico.
        let dto = medx::MailLogDto::new(
            "teste@medx-sdk.local",
            "MEDX SDK TESTE — pode ser ignorado",
            "<p>Corpo do e-mail de teste gerado pelo SDK.</p>",
        );
        // Pode retornar erro da API dependendo do estado — não deve panicar no SDK
        let _ = c.log_email(&dto);
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_log_email_sem_contato_nao_panica() {
    with_temp_dir(|| {
        let dto = medx::MailLogDto::new(
            "noreply@medx-sdk.local",
            "Teste sem paciente",
            "Corpo do teste",
        );
        // Pode retornar erro da API — não deve panicar no SDK
        let _ = client().log_email(&dto);
    });
}
