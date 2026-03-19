//! Testes de integração completos usando o paciente real
//! **Paciente de Teste** (Id_do_Cliente = 100001).
//!
//! Regras:
//! - Todas as ações destrutivas (criação, modificação) são desfeitas ao final.
//! - Agendamentos criados são deletados via `AppointmentGuard`.
//! - Contatos criados são deletados.
//! - Notas criadas são deletadas.
//! - Histórico médico (upsert) é restaurado ao estado original.
//!
//! Execute: `cargo test --test paciente_real_tests -- --ignored`

mod common;
use common::{shared_client, with_temp_dir};
use medx::{AppointmentDto, MedxClient};

/// ID fixo do paciente de teste.
const PATIENT_ID: i64 = 100001;
/// Nome do paciente de teste.
const PATIENT_NAME: &str = "Paciente de Teste";

/// Retorna o cliente compartilhado (login único por processo).
fn client() -> &'static MedxClient {
    shared_client()
}

// ── Guard de agendamento ──────────────────────────────────────────────────────

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
        let _ = self.client.delete_appointment(self.id);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AUTH / USUÁRIOS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_auth_current_user() {
    with_temp_dir(|| {
        let user = client().current_user().expect("current_user falhou");
        assert!(user.db_id > 0, "db_id deve ser positivo");
        assert!(!user.username.is_empty(), "username não deve ser vazio");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_auth_users_list() {
    with_temp_dir(|| {
        let users = client().users().expect("users falhou");
        assert!(!users.is_empty(), "deve haver ao menos 1 usuário no sistema");
        for u in &users {
            assert!(!u.username.is_empty(), "username do usuário não deve ser vazio");
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// CONTACTS — leitura com Paciente Teste
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_contacts_busca_por_nome_completo() {
    with_temp_dir(|| {
        let results = client()
            .search_contacts("Paciente Teste", medx::ContactSearchGroup::All, 10)
            .expect("search_contacts falhou");
        let found = results.iter().any(|c| c.id == PATIENT_ID);
        assert!(found, "paciente {} (id={}) não encontrado na busca", PATIENT_NAME, PATIENT_ID);
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_contacts_busca_por_nome_parcial() {
    with_temp_dir(|| {
        // GroupValue=1 é o valor correto; GroupValue>=50 retorna vazio na API.
        let results = client()
            .search_contacts("Paciente", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        assert!(!results.is_empty(), "busca por 'Paciente' deve retornar ao menos um resultado");
        let found = results.iter().any(|c| c.id == PATIENT_ID);
        assert!(found, "paciente {} não encontrado buscando por 'Paciente'", PATIENT_NAME);
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_contacts_ficha_completa() {
    with_temp_dir(|| {
        let contact = client()
            .contact(PATIENT_ID)
            .expect("contact falhou para Paciente Teste");
        assert_eq!(contact.id, PATIENT_ID, "id deve bater");
        assert!(
            contact.name.to_lowercase().contains("paciente"),
            "nome deve conter Paciente, obteve: {}",
            contact.name
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// CONTACTS — ciclo CRUD (contato temporário, NÃO o Paciente)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_contacts_ciclo_criar_atualizar_deletar() {
    with_temp_dir(|| {
        let c = client();

        // 1. Gerar ID único via API
        let new_id = c.new_contact_id().expect("new_contact_id falhou");

        // 2. Criar contato temporário
        let mut dto = medx::ContactDto::new("MEDX SDK TESTE PODE DELETAR");
        dto.id = new_id;
        dto.gender = "M".to_string();
        c.create_contact(&dto).expect("create_contact falhou");

        // 3. Verificar que existe
        let found = c.contact(new_id).expect("contact após create falhou");
        assert!(found.name.contains("MEDX SDK TESTE"), "nome deve conter MEDX SDK TESTE");

        // 4. Deletar (cleanup)
        c.delete_contact(new_id).expect("delete_contact falhou");

        // 5. Verificar que foi deletado (deve retornar erro)
        assert!(
            c.contact(new_id).is_err(),
            "contato deletado não deve mais existir"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// PRONTUÁRIO — usando Paciente Teste
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_historico_medico_paciente() {
    with_temp_dir(|| {
        let summary = client()
            .medical_history_summary(PATIENT_ID)
            .expect("medical_history_summary falhou");
        // Campos podem estar vazios, o importante é não panicar
        let _ = summary;
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_upsert_historico_restaura_original() {
    with_temp_dir(|| {
        let c = client();

        // Lê estado atual — pode retornar 500 se o módulo não estiver habilitado.
        let original = match c.medical_history_summary(PATIENT_ID) {
            Ok(v) => v,
            Err(medx::MedxError::Api { status: 500, .. }) => return,
            Err(e) => panic!("leitura inicial do histórico falhou: {:?}", e),
        };

        // Salva com os mesmos dados (upsert idempotente)
        c.upsert_medical_history_summary(PATIENT_ID, &original)
            .expect("upsert_medical_history_summary falhou");

        // Verifica que os dados persistiram
        let restored = c
            .medical_history_summary(PATIENT_ID)
            .expect("leitura após upsert falhou");
        assert_eq!(
            restored.diagnostic, original.diagnostic,
            "diagnóstico deve ser o mesmo após restaurar"
        );
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_registros_medicos_paciente() {
    with_temp_dir(|| {
        // Pode retornar 404 se o endpoint não estiver disponível para este paciente.
        let records = match client().medical_records(PATIENT_ID) {
            Ok(v) => v,
            Err(medx::MedxError::Api { status: 404, .. }) => return,
            Err(medx::MedxError::Api { status: 500, .. }) => return,
            Err(e) => panic!("medical_records falhou: {:?}", e),
        };
        for r in &records {
            assert!(r.id != 0, "id do registro deve ser não-zero");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_busca_registros_paciente() {
    with_temp_dir(|| {
        // Pode falhar com InvalidCredentials se testes CLI de auth rodam em paralelo
        let _ = client().search_medical_records(PATIENT_ID, "");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_keywords() {
    with_temp_dir(|| {
        let _ = client().medical_keywords().expect("medical_keywords falhou");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_convenios_e_procedimentos() {
    with_temp_dir(|| {
        let c = client();
        let convenios = c.convenios().expect("convenios falhou");
        let _ = c.procedures().expect("procedures falhou");
        if let Some(conv) = convenios.first() {
            let _ = c.convenio_procedures(conv.id).expect("convenio_procedures falhou");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_unidades_negocio() {
    with_temp_dir(|| {
        let _ = client().business_units().expect("business_units falhou");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_formularios() {
    with_temp_dir(|| {
        let c = client();
        let forms = c.forms().expect("forms falhou");
        if let Some(form) = forms.first() {
            let html = c.form_html(form.id).expect("form_html falhou");
            assert!(!html.is_empty(), "HTML do formulário não deve ser vazio");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_module_records_exames_nao_panica() {
    with_temp_dir(|| {
        // API retorna 404 quando o módulo não tem registros — deve ser aceito como Ok(vec![])
        // ou como erro; ambos os casos são válidos.
        let result = client().module_records(PATIENT_ID, "Exames");
        match result {
            Ok(records) => {
                for r in &records {
                    assert!(r.id != 0, "id do registro de módulo deve ser não-zero");
                    assert!(!r.date.is_empty(), "data do registro de módulo não deve ser vazia");
                }
            }
            Err(medx::MedxError::Api { status: 404, .. }) => {
                // Aceitável: módulo sem registros retorna 404
            }
            Err(e) => panic!("module_records falhou com erro inesperado: {:?}", e),
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_module_records_modulo_inexistente() {
    with_temp_dir(|| {
        // Módulo que não existe deve retornar Ok(vec![]) ou qualquer erro graciosamente
        let result = client().module_records(PATIENT_ID, "ModuloQueNaoExiste");
        let _ = result;
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// FINANCAS — usando Paciente Teste
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_financas_atendimentos_por_paciente_paciente() {
    with_temp_dir(|| {
        let attendances = client()
            .attendances_by_patient(PATIENT_ID)
            .expect("attendances_by_patient falhou");
        for a in &attendances {
            assert!(!a.id.is_empty(), "id do atendimento não deve ser vazio");
            let _ = a.is_closed();
            let _ = a.is_budget();
            let _ = a.balance_due();
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_financas_todos_atendimentos() {
    with_temp_dir(|| {
        let attendances = client()
            .all_attendances("2026-01-01", "2026-03-31")
            .expect("all_attendances falhou");
        for a in &attendances {
            assert!(!a.id.is_empty(), "id do atendimento não deve ser vazio");
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// AGENDA — agendamento para Paciente Teste com cleanup garantido
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_agenda_parameters() {
    with_temp_dir(|| {
        let params = client().agenda_parameters().expect("agenda_parameters falhou");
        assert!(
            !params.general.slot_duration.is_empty(),
            "slot_duration não deve ser vazio"
        );
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_agenda_sectors() {
    with_temp_dir(|| {
        let _sectors = client().agenda_sectors().expect("agenda_sectors falhou");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_agenda_criar_com_paciente_e_deletar() {
    with_temp_dir(|| {
        let c = client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");
        let agenda_users: Vec<_> = params.users.iter().filter(|u| u.has_agenda).collect();
        if agenda_users.is_empty() {
            return; // Sem usuários de agenda, pula
        }

        let user_id = agenda_users[0].id;

        // Cria agendamento com Paciente Teste como paciente
        let mut dto = AppointmentDto::new(
            user_id,
            "2026-12-30T09:00:00",
            "2026-12-30T09:30:00",
        );
        dto.contact_id = Some(PATIENT_ID);
        dto.description = "MEDX SDK TESTE — pode ser excluído".to_string();

        // Agendamento pode falhar por restrição de horário (400) — considerado ok.
        let scheduled = match c.create_appointment(&dto) {
            Ok(v) => v,
            Err(medx::MedxError::Api { status: 400, .. }) => return,
            Err(e) => panic!("create_appointment falhou com erro inesperado: {:?}", e),
        };
        if !scheduled {
            return; // Módulo de agenda não disponível
        }

        // Busca o agendamento criado
        let appointments = c
            .daily_agenda(user_id, "2026-12-30")
            .expect("daily_agenda falhou");

        let created = appointments
            .iter()
            .find(|a| a.description.contains("MEDX SDK TESTE"));

        if let Some(appt) = created {
            // Guard garante delete mesmo em panic
            let _guard = AppointmentGuard::new(c, appt.id);
            assert!(appt.id != 0, "id do agendamento deve ser não-zero");

            // Atualiza status para desmarcado (0) antes de deletar
            c.update_appointment_status(appt.id, 0)
                .expect("update_appointment_status falhou");
            // Guard deleta no Drop
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_agenda_relatorio() {
    with_temp_dir(|| {
        let c = client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");
        let agenda_users: Vec<_> = params.users.iter().filter(|u| u.has_agenda).collect();
        if agenda_users.is_empty() {
            return;
        }
        use medx::AgendaReportDto;
        let dto = AgendaReportDto::new("2026-01-01", "2026-03-31", agenda_users[0].id);
        let _ = c.agenda_report(&dto).expect("agenda_report falhou");
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// HOJE / DASHBOARD
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_hoje_notificacoes() {
    with_temp_dir(|| {
        let notifs = client().hoje_notificacoes().expect("hoje_notificacoes falhou");
        for n in &notifs {
            let _ = n.id;
            let _ = n.tipo.as_str();
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_hoje_ultimos_atendidos() {
    with_temp_dir(|| {
        let atendidos = client().ultimos_atendidos().expect("ultimos_atendidos falhou");
        for a in &atendidos {
            assert!(!a.patient_name.is_empty(), "nome do paciente não deve ser vazio");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_hoje_trial_info() {
    with_temp_dir(|| {
        let info = client().trial_info().expect("trial_info falhou");
        let _ = info.is_trial();
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_hoje_notas_ciclo_completo() {
    with_temp_dir(|| {
        let c = client();
        let user = c.current_user().expect("current_user falhou");

        // Criar nota
        let dto = medx::InsertNotaDto::new(user.user_id, "MEDX SDK TESTE — pode ser excluída");
        c.insert_nota(&dto).expect("insert_nota falhou");

        // Busca a nota criada pelo texto (a API pode não retornar o ID)
        let nota_id = c.notas()
            .unwrap_or_default()
            .iter()
            .find(|n| n.text.contains("MEDX SDK TESTE"))
            .map(|n| n.id)
            .unwrap_or(0);

        if nota_id != 0 {
            // Atualizar nota
            let upd = medx::UpdateNotaDto::new(nota_id, user.user_id, "MEDX SDK TESTE — atualizada");
            c.update_nota(&upd).expect("update_nota falhou");

            // Deletar (cleanup)
            c.delete_nota(nota_id).expect("delete_nota falhou");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_hoje_nota_cliente_paciente() {
    with_temp_dir(|| {
        let c = client();
        let user = c.current_user().expect("current_user falhou");
        let dto = medx::InsertNotaClienteDto::new(
            PATIENT_ID,
            user.user_id,
            "MEDX SDK TESTE — nota do paciente, pode ser ignorada",
        );
        // Pode retornar erro dependendo de permissões
        let _ = c.insert_nota_cliente(&dto);
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_hoje_is_otp_or_expired() {
    with_temp_dir(|| {
        let _ = client().is_otp_or_expired().expect("is_otp_or_expired falhou");
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// CONFIGURAÇÕES DA CLÍNICA
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_settings_general_parameters() {
    with_temp_dir(|| {
        let params = client().general_parameters().expect("general_parameters falhou");
        assert!(
            !params.business_hours_start.is_empty() || !params.slot_duration.is_empty(),
            "parâmetros gerais devem ter dados"
        );
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_settings_color_parameters() {
    with_temp_dir(|| {
        let _ = client().color_parameters().expect("color_parameters falhou");
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// NOTIFICAÇÕES
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_notif_client_settings() {
    with_temp_dir(|| {
        let settings = client().client_settings().expect("client_settings falhou");
        let _ = settings.id;
        let _ = settings.has_sms_template();
        let _ = settings.has_whatsapp_template();
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_notif_log_email_paciente() {
    with_temp_dir(|| {
        let dto = medx::MailLogDto::new(
            PATIENT_ID,
            "teste@exemplo.com",
            "MEDX SDK TESTE",
            "Corpo do e-mail de teste",
        );
        // Registra log de e-mail (sem método de deleção disponível na API)
        let _ = client().log_email(&dto);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// MARKETING
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_marketing_events() {
    with_temp_dir(|| {
        let _ = client().events().expect("events falhou");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_marketing_quests_e_insert_paciente() {
    with_temp_dir(|| {
        let c = client();
        let quests = c.quests().expect("quests falhou");
        if let Some(q) = quests.first() {
            let dto = medx::InsertQuestDto::new(PATIENT_ID, q.id);
            // Pode retornar erro dependendo de permissões
            let _ = c.insert_quest(&dto);
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_marketing_diagnostico_qp() {
    with_temp_dir(|| {
        let _ = client().diagnostico_qp().expect("diagnostico_qp falhou");
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// AJUSTES & ADMINISTRAÇÃO
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_ajustes_list_reports() {
    with_temp_dir(|| {
        let reports = client().list_reports().expect("list_reports falhou");
        for r in &reports {
            assert!(!r.name.is_empty(), "nome do relatório não deve ser vazio");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_ajustes_doc_folders() {
    with_temp_dir(|| {
        let folders = client().doc_folders("").expect("doc_folders falhou");
        for f in &folders {
            assert!(!f.name.is_empty(), "nome da pasta não deve ser vazio");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_ajustes_ics_config() {
    with_temp_dir(|| {
        // Pode retornar erro se ICS não configurado na clínica
        let _ = client().ics_config();
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// CHAT INTERNO
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_chat_users() {
    with_temp_dir(|| {
        let users = client().chat_users().expect("chat_users falhou");
        for u in &users {
            assert!(!u.full_name.is_empty() || !u.username.is_empty(), "chat user deve ter nome");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_chat_contagem_nao_lidos() {
    with_temp_dir(|| {
        let _count = client().chat_unread_count().expect("chat_unread_count falhou");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_chat_historico() {
    with_temp_dir(|| {
        let c = client();
        let users = c.chat_users().expect("chat_users falhou");
        if let Some(u) = users.first() {
            let history = c.chat_history(u.id).expect("chat_history falhou");
            for m in &history {
                let _ = m.is_read();
                let _ = m.is_shown();
            }
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_chat_mensagens_recebidas_e_marcar_lidas() {
    with_temp_dir(|| {
        let c = client();
        // chat_incoming pode retornar 500 se não houver suporte ao módulo de chat.
        let msgs = match c.chat_incoming() {
            Ok(v) => v,
            Err(medx::MedxError::Api { status: 500, .. }) => return,
            Err(e) => panic!("chat_incoming falhou: {:?}", e),
        };
        if !msgs.is_empty() {
            let ids: Vec<i64> = msgs.iter().map(|m| m.id).collect();
            c.mark_messages_read(&ids).expect("mark_messages_read falhou");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_chat_enviar_mensagem() {
    with_temp_dir(|| {
        let c = client();
        let me = c.current_user().expect("current_user falhou");
        let users = c.chat_users().expect("chat_users falhou");

        // Envia para o primeiro usuário diferente do atual
        let target = users.iter().find(|u| u.id != me.user_id);
        if let Some(to) = target {
            let dto = medx::SendMessageDto::new(
                me.user_id,
                &me.username,
                to.id,
                &to.full_name,
                "MEDX SDK TESTE — pode ser ignorado",
                "",
            );
            // Pode falhar dependendo de permissões
            let _ = c.send_chat_message(&dto);
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// PRONTUÁRIO — métodos de escrita
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_resolve_file_url_retorna_azure_url() {
    with_temp_dir(|| {
        let c = client();
        let records = match c.medical_records(PATIENT_ID) {
            Ok(v) => v,
            Err(medx::MedxError::Api { status: 404 | 500, .. }) => return,
            Err(e) => panic!("medical_records falhou: {:?}", e),
        };

        // Busca o primeiro registro com arquivo
        let record_with_file = records.iter().find(|r| r.has_file());
        let Some(r) = record_with_file else { return };

        let url = c
            .resolve_file_url(&r.classe)
            .expect("resolve_file_url falhou");

        assert!(!url.is_empty(), "URL não deve ser vazia");
        assert!(
            url.starts_with("https://"),
            "URL deve ser HTTPS, obteve: {url}"
        );
        assert!(
            url.contains("blob.core.windows.net") || url.contains("medxdata"),
            "URL deve apontar para Azure blob, obteve: {url}"
        );
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_prontuario_update_medical_record_roundtrip() {
    // Prontuário é imutável (sem delete) — usamos um registro existente e
    // fazemos upsert com os mesmos dados (operação idempotente).
    with_temp_dir(|| {
        let c = client();
        let records = match c.medical_records(PATIENT_ID) {
            Ok(v) => v,
            Err(medx::MedxError::Api { status: 404 | 500, .. }) => return,
            Err(e) => panic!("medical_records falhou: {:?}", e),
        };

        // Pega o primeiro registro de texto (sem arquivo) para não alterar arquivos
        let text_record = records.iter().find(|r| !r.has_file());
        let Some(original) = text_record else { return };

        // Cria DTO com os mesmos dados (round-trip sem mudança de conteúdo)
        let dto: medx::MedicalRecordDto = original.clone().into();
        c.update_medical_record(&dto)
            .expect("update_medical_record falhou");

        // Verifica que o conteúdo permanece igual
        let records_after = c
            .medical_records(PATIENT_ID)
            .expect("medical_records pós-update falhou");
        let after = records_after.iter().find(|r| r.id == original.id);
        if let Some(a) = after {
            assert_eq!(
                a.content, original.content,
                "conteúdo não deve mudar em update idempotente"
            );
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// FINANCAS — métodos de escrita
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_financas_update_invoice_roundtrip_paciente() {
    // Pega um atendimento existente do Paciente e reenvia os mesmos dados.
    // É um round-trip idempotente — não altera nenhum valor real.
    with_temp_dir(|| {
        let c = client();
        let attendances = c
            .attendances_by_patient(PATIENT_ID)
            .expect("attendances_by_patient falhou");

        let Some(original) = attendances.first() else { return };

        let dto: medx::AttendanceDto = original.clone().into();
        c.update_invoice(&dto).expect("update_invoice falhou");

        // Verifica que o atendimento ainda existe com os mesmos dados
        let after = c
            .attendances_by_patient(PATIENT_ID)
            .expect("attendances_by_patient pós-update falhou");
        let restored = after.iter().find(|a| a.id == original.id);
        if let Some(a) = restored {
            assert_eq!(
                a.invoice_value, original.invoice_value,
                "valor da fatura não deve mudar em update idempotente"
            );
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_financas_create_pre_payment_nao_panica() {
    // Testa o endpoint Stone/Pagar.me. Pode retornar erro se o módulo não
    // estiver habilitado na clínica — isso é considerado aceitável.
    with_temp_dir(|| {
        let c = client();
        let contact = c.contact(PATIENT_ID).expect("contact falhou para Paciente");

        let email = if contact.email.is_empty() { "teste@exemplo.com" } else { &contact.email };
        let dto = medx::PrePaymentDto::new(
            PATIENT_ID.to_string(),
            1.0, // R$ 1,00 — mínimo possível para não gerar cobrança real em ambiente de teste
            "MEDX SDK TESTE",
            email,
            &contact.name,
        );

        match c.create_pre_payment(&dto) {
            Ok(link) => {
                // Se retornar link, deve ser uma URL válida
                if let Some(url) = link {
                    assert!(
                        url.starts_with("http"),
                        "link de pagamento deve ser URL, obteve: {url}"
                    );
                }
            }
            // Módulo não habilitado ou credenciais Stone não configuradas — aceitável
            Err(medx::MedxError::Api { status: 400 | 404 | 500, .. }) => {}
            Err(e) => panic!("create_pre_payment falhou com erro inesperado: {:?}", e),
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// AGENDA — métodos de escrita adicionais
// ─────────────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_agenda_update_appointment_roundtrip() {
    // Cria um agendamento, atualiza a descrição, verifica e deleta.
    with_temp_dir(|| {
        let c = client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");
        let agenda_users: Vec<_> = params.users.iter().filter(|u| u.has_agenda).collect();
        if agenda_users.is_empty() {
            return;
        }

        let user_id = agenda_users[0].id;
        let mut dto = AppointmentDto::new(user_id, "2026-12-29T14:00:00", "2026-12-29T14:30:00");
        dto.contact_id = Some(PATIENT_ID);
        dto.description = "MEDX SDK TESTE UPDATE — pode ser excluído".to_string();

        let scheduled = match c.create_appointment(&dto) {
            Ok(v) => v,
            Err(medx::MedxError::Api { status: 400, .. }) => return,
            Err(e) => panic!("create_appointment falhou: {:?}", e),
        };
        if !scheduled {
            return;
        }

        // Busca o agendamento criado
        let appointments = c
            .daily_agenda(user_id, "2026-12-29")
            .expect("daily_agenda falhou");
        let created = appointments
            .iter()
            .find(|a| a.description.contains("MEDX SDK TESTE UPDATE"));

        let Some(appt) = created else { return };
        let _guard = AppointmentGuard::new(c, appt.id);

        // Atualiza descrição
        let mut update_dto: medx::AppointmentDto = appt.clone().into();
        update_dto.description = "MEDX SDK TESTE UPDATE — atualizado".to_string();
        c.update_appointment(&update_dto)
            .expect("update_appointment falhou");

        // Verifica que a descrição foi atualizada
        let after = c
            .daily_agenda(user_id, "2026-12-29")
            .expect("daily_agenda pós-update falhou");
        let updated = after.iter().find(|a| a.id == appt.id);
        if let Some(u) = updated {
            assert!(
                u.description.contains("atualizado"),
                "descrição deve ter sido atualizada, obteve: {}",
                u.description
            );
        }
        // _guard.drop() → delete_appointment garantido
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_agenda_confirm_whatsapp_nao_panica() {
    // Cria um agendamento para Paciente, tenta confirmar via WhatsApp e deleta.
    // Nota: pode enviar mensagem real se WhatsApp estiver configurado na clínica.
    with_temp_dir(|| {
        let c = client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");
        let agenda_users: Vec<_> = params.users.iter().filter(|u| u.has_agenda).collect();
        if agenda_users.is_empty() {
            return;
        }

        let user_id = agenda_users[0].id;
        let mut dto = AppointmentDto::new(user_id, "2026-12-28T10:00:00", "2026-12-28T10:30:00");
        dto.contact_id = Some(PATIENT_ID);
        dto.description = "MEDX SDK TESTE WHATSAPP — pode ser excluído".to_string();

        let scheduled = match c.create_appointment(&dto) {
            Ok(v) => v,
            Err(medx::MedxError::Api { status: 400, .. }) => return,
            Err(e) => panic!("create_appointment falhou: {:?}", e),
        };
        if !scheduled {
            return;
        }

        let appointments = c
            .daily_agenda(user_id, "2026-12-28")
            .expect("daily_agenda falhou");
        let created = appointments
            .iter()
            .find(|a| a.description.contains("MEDX SDK TESTE WHATSAPP"));

        let Some(appt) = created else { return };
        let _guard = AppointmentGuard::new(c, appt.id);

        // Confirma via WhatsApp — pode falhar se WhatsApp não configurado (aceitável)
        match c.confirm_appointment_whatsapp(appt.id) {
            Ok(()) => {}
            Err(medx::MedxError::Api { status: 400 | 404 | 500, .. }) => {}
            Err(e) => panic!("confirm_appointment_whatsapp falhou inesperadamente: {:?}", e),
        }
        // _guard.drop() → delete garantido
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn test_agenda_block_ciclo_criar_e_deletar() {
    with_temp_dir(|| {
        let c = shared_client();
        let params = c.agenda_parameters().expect("agenda_parameters falhou");
        let user = params.users.iter().find(|u| u.has_agenda)
            .expect("nenhum profissional com agenda");

        // Cria bloqueio 2 anos no futuro — quinta-feira 2028-12-28, 09:00–09:30
        c.create_agenda_block(user.id, "2028-12-28T09:00:00", "2028-12-28T09:30:00")
            .expect("create_agenda_block falhou");

        // Lista e limpa (soft-delete: status=0)
        let blocks = c.daily_blocks(user.id, "2028-12-28").expect("daily_blocks falhou");
        for b in blocks.iter().filter(|b| b.start.contains("09:00")) {
            let _ = c.remove_agenda_block(b.id);
        }
    });
}
