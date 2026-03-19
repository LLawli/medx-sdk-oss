//! Testes de integração — Etapa 6: Finanças / Atendimentos.
//! Execute com: `cargo test --test financas_tests -- --ignored`

mod common;
use common::{shared_client, with_temp_dir};
use medx::MedxClient;

fn client() -> &'static MedxClient { shared_client() }


// ── attendances_by_patient ────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_attendances_by_patient_nao_pânica() {
    with_temp_dir(|| {
        let c = client();
        // Busca um paciente real
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let patient_id = contacts[0].id;
        let attendances = c
            .attendances_by_patient(patient_id)
            .expect("attendances_by_patient falhou");

        for a in &attendances {
            assert!(!a.id.is_empty(), "id do atendimento não deve ser vazio");
            assert!(!a.date.is_empty(), "data não deve ser vazia");
            assert!(a.invoice_value >= 0.0, "valor da fatura não deve ser negativo");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_attendances_by_patient_inexistente_retorna_vazio() {
    with_temp_dir(|| {
        let result = client()
            .attendances_by_patient(999999999)
            .expect("attendances_by_patient para paciente inexistente falhou");
        assert!(result.is_empty());
    });
}

// ── all_attendances ───────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_all_attendances_nao_pânica() {
    with_temp_dir(|| {
        let _ = client().all_attendances("", "");
    });
}

// ── helpers semânticos ────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_attendance_helpers_semanticos() {
    with_temp_dir(|| {
        let c = client();
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let attendances = c
            .attendances_by_patient(contacts[0].id)
            .expect("attendances_by_patient falhou");
        for a in &attendances {
            // Verifica que os helpers não pânicam
            let _ = a.is_closed();
            let _ = a.is_budget();
            let _ = a.balance_due();
        }
    });
}

// ── from/into ─────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_attendance_into_dto_roundtrip() {
    with_temp_dir(|| {
        let c = client();
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let attendances = c
            .attendances_by_patient(contacts[0].id)
            .expect("attendances_by_patient falhou");
        if attendances.is_empty() {
            return;
        }
        // Converte para DTO — só verifica que não pânica
        let dto: medx::AttendanceDto = attendances[0].clone().into();
        assert_eq!(dto.action, "UPDATE");
        assert!(!dto.id.is_empty());
    });
}
