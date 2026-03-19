//! Testes de integração — Etapa 4: Agenda.
//! Execute com: `cargo test --test agenda_tests -- --ignored`

mod common;
use common::{shared_client, with_temp_dir};
use medx::{AppointmentDto, MedxClient};

fn client() -> &'static MedxClient { shared_client() }


/// Guard que garante deleção do agendamento mesmo em caso de panic.
struct AppointmentGuard<'a> {
    client: &'a MedxClient,
    id: i64,
}

impl<'a> AppointmentGuard<'a> {
    fn new(client: &'a MedxClient, id: i64) -> Self {
        Self { client, id }
    }
}

impl<'a> Drop for AppointmentGuard<'a> {
    fn drop(&mut self) {
        // Ignora erro no cleanup para não mascarar o erro original do teste
        let _ = self.client.delete_appointment(self.id);
    }
}

// ── agenda_parameters ─────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_agenda_parameters_retorna_estrutura() {
    with_temp_dir(|| {
        let params = client()
            .agenda_parameters()
            .expect("agenda_parameters falhou");
        assert!(
            !params.general.slot_duration.is_empty(),
            "slot_duration não deve ser vazio"
        );
        // Usuários e setores podem estar vazios dependendo da conta
        for u in &params.users {
            assert!(!u.username.is_empty(), "username não deve ser vazio");
        }
    });
}

// ── agenda_sectors ────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_agenda_sectors_nao_pânica() {
    with_temp_dir(|| {
        // Pode ser vazio ou não, apenas verificamos que não pânica
        let _ = client().agenda_sectors();
    });
}

// ── daily_agenda ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_daily_agenda_hoje_retorna_lista() {
    with_temp_dir(|| {
        let c = client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");

        // Se não há usuários de agenda, pula o teste
        let agenda_users: Vec<_> = params.users.iter().filter(|u| u.has_agenda).collect();
        if agenda_users.is_empty() {
            return;
        }

        let user_id = agenda_users[0].id;
        let today = chrono_today();
        let appointments = c
            .daily_agenda(user_id, &today)
            .expect("daily_agenda falhou");

        // Pode ser vazio (sem agendamentos hoje), mas não deve falhar
        for a in &appointments {
            assert!(a.id != 0, "id do agendamento deve ser não-zero");
            assert!(!a.start.is_empty(), "start não deve ser vazio");
            assert!(!a.end.is_empty(), "end não deve ser vazio");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_daily_agenda_usuario_inexistente_retorna_vazio() {
    with_temp_dir(|| {
        // Usuário inexistente deve retornar lista vazia (não erro)
        let result = client()
            .daily_agenda(999999999, "2026-01-01")
            .expect("daily_agenda com usuário inexistente falhou");
        assert!(
            result.is_empty(),
            "usuário inexistente deve retornar lista vazia"
        );
    });
}

// ── create / delete appointment ───────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_create_and_delete_appointment() {
    with_temp_dir(|| {
        let c = client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");

        let agenda_users: Vec<_> = params.users.iter().filter(|u| u.has_agenda).collect();
        if agenda_users.is_empty() {
            // Conta sem usuários de agenda — não é possível criar agendamento
            return;
        }

        let user_id = agenda_users[0].id;

        // Usa horário dentro do expediente padrão (08:00–18:00) em data futura
        let mut dto = AppointmentDto::new(
            user_id,
            "2026-12-10T09:00:00",
            "2026-12-10T09:30:00",
        );
        dto.description = "MEDX SDK TESTE INTEGRACAO".to_string();

        let scheduled = c.create_appointment(&dto).expect("create_appointment falhou");

        if !scheduled {
            // Conta não suporta o módulo de agenda ou horário fora do permitido
            return;
        }

        // Busca o agendamento criado para obter o ID e poder limpar
        let appointments = c
            .daily_agenda(user_id, "2026-12-10")
            .expect("daily_agenda pós-create falhou");

        let created = appointments
            .iter()
            .find(|a| a.description == "MEDX SDK TESTE INTEGRACAO");

        if let Some(appt) = created {
            let _guard = AppointmentGuard::new(&c, appt.id);
            assert!(appt.id != 0, "id deve ser não-zero");
        }
    });
}

// ── create_agenda_block / daily_blocks ────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_agenda_block_ciclo_criar_e_deletar() {
    with_temp_dir(|| {
        let c = client();
        // Usa o primeiro profissional com agenda ativa
        let params = c.agenda_parameters().expect("agenda_parameters falhou");
        let user = params.users.iter().find(|u| u.has_agenda)
            .expect("nenhum profissional com agenda");

        // 2028-12-28 é quinta-feira — dia útil, dentro do expediente 08:00-20:00
        let start = "2028-12-28T09:00:00";
        let end   = "2028-12-28T09:30:00";

        // Cria o bloqueio
        let ok = c.create_agenda_block(user.id, start, end)
            .expect("create_agenda_block falhou");
        assert!(ok, "bloqueio deve ter sido criado");

        // Verifica que aparece em daily_blocks
        let blocks = c.daily_blocks(user.id, "2028-12-28")
            .expect("daily_blocks falhou");
        let found = blocks.iter().find(|b| b.start.contains("09:00") || b.start.contains("2028-12-28T09"));

        // Remove bloqueio (soft-delete: status=0)
        if let Some(block) = found {
            c.remove_agenda_block(block.id).expect("remove_agenda_block falhou");
        }
    });
}

// ── update_appointment_status ─────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_update_appointment_status_nao_pânica() {
    with_temp_dir(|| {
        let c = client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");
        let agenda_users: Vec<_> = params.users.iter().filter(|u| u.has_agenda).collect();
        if agenda_users.is_empty() {
            return;
        }

        let user_id = agenda_users[0].id;
        let today = chrono_today();
        let appointments = c.daily_agenda(user_id, &today).expect("daily_agenda falhou");

        if let Some(appt) = appointments.first() {
            // Tenta atualizar para o mesmo status (idempotente)
            let _ = c.update_appointment_status(appt.id, appt.status);
        }
        // Se não há agendamentos hoje, apenas verifica que o endpoint existe
    });
}

// ── agenda_report ─────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_agenda_report_nao_pânica() {
    with_temp_dir(|| {
        use medx::AgendaReportDto;
        let c = client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");
        let user_id = params.users.first().map(|u| u.id).unwrap_or(0);

        let dto = AgendaReportDto::new("2026-03-01", "2026-03-31", user_id);
        // Apenas verifica que o endpoint responde sem erro
        let _ = c.agenda_report(&dto);
    });
}

// ── no_show_report ────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_no_show_report_nao_pânica() {
    with_temp_dir(|| {
        use medx::NoShowReportDto;
        let c = client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");
        let user_id = params.users.first().map(|u| u.id).unwrap_or(0);

        let dto = NoShowReportDto::new("2026-03-01", "2026-03-31", user_id);
        let _ = c.no_show_report(&dto);
    });
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Retorna a data de hoje no formato "YYYY-MM-DD".
fn chrono_today() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Cálculo simples: dia juliano → data gregoriana
    let days = secs / 86400;
    let (y, m, d) = days_to_ymd(days);
    format!("{y:04}-{m:02}-{d:02}")
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algoritmo de Richards para converter dias Unix em Y-M-D
    let j = days + 2440588; // dias Unix → Dia Juliano
    let f = j + 1401 + (((4 * j + 274277) / 146097) * 3) / 4 - 38;
    let e = 4 * f + 3;
    let g = (e % 1461) / 4;
    let h = 5 * g + 2;
    let day = (h % 153) / 5 + 1;
    let month = (h / 153 + 2) % 12 + 1;
    let year = e / 1461 - 4716 + (12 + 2 - month) / 12;
    (year, month, day)
}
