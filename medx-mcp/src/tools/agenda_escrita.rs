//! Escrita na agenda. Só com `MEDX_MCP_ALLOW_WRITE`.

use medx::AppointmentDto;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{ToolError, check_date, check_time_range, json_result};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateAppointmentParams {
    /// Id do profissional (o `id` de `listar_profissionais_agenda`).
    pub profissional_id: i64,
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
    /// Início, AAAA-MM-DDTHH:MM, hora local.
    pub inicio: String,
    /// Fim, AAAA-MM-DDTHH:MM, hora local.
    pub fim: String,
    /// Texto do agendamento. Padrão: o nome do paciente.
    #[serde(default)]
    pub descricao: Option<String>,
    /// Id do procedimento (o `id` de `listar_procedimentos`).
    #[serde(default)]
    pub procedimento_id: Option<i64>,
    /// Id do diagnóstico QP (o `id` de `listar_diagnosticos_qp`).
    #[serde(default)]
    pub diagnostico_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BlockParams {
    /// Id do profissional (o `id` de `listar_profissionais_agenda`).
    pub profissional_id: i64,
    /// Início, AAAA-MM-DDTHH:MM, hora local.
    pub inicio: String,
    /// Fim, AAAA-MM-DDTHH:MM, hora local.
    pub fim: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct StatusParams {
    /// Id do agendamento (o `id` de `agenda_do_dia`).
    pub agendamento_id: i64,
    /// Novo status: a posição do rótulo em `parametros_agenda` (por exemplo
    /// 0 desmarcado, 1 agendado, 2 compareceu).
    pub status: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RescheduleParams {
    /// Id do agendamento (o `id` de `agenda_do_dia`).
    pub agendamento_id: i64,
    /// Profissional atual do agendamento.
    pub profissional_id: i64,
    /// Dia atual do agendamento, AAAA-MM-DD.
    pub data: String,
    /// Novo início, AAAA-MM-DDTHH:MM, hora local.
    pub novo_inicio: String,
    /// Novo fim, AAAA-MM-DDTHH:MM, hora local.
    pub novo_fim: String,
    /// Outro profissional, se o agendamento muda de agenda.
    #[serde(default)]
    pub novo_profissional_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AppointmentParams {
    /// Id do agendamento (o `id` de `agenda_do_dia`).
    pub agendamento_id: i64,
}

/// Mensagem de quando a MedX respondeu que não gravou o agendamento.
fn not_created(what: &str) -> ToolError {
    ToolError::Rejected(format!(
        "a MedX não criou {what} (horário ocupado ou dados recusados). Confira a agenda do dia com `agenda_do_dia` antes de tentar de novo"
    ))
}

#[tool_router(router = agenda_escrita_router, vis = "pub(crate)")]
impl MedxServer {
    /// Cria um agendamento de um paciente com um profissional, com status
    /// agendado. Devolve `{"criado": true}`; a MedX pode recusar (horário
    /// ocupado, por exemplo), e aí a ferramenta devolve erro.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    pub async fn criar_agendamento(
        &self,
        Parameters(params): Parameters<CreateAppointmentParams>,
    ) -> Result<CallToolResult, ToolError> {
        let (start, end) = check_time_range(&params.inicio, &params.fim)?;
        let created = self
            .medx
            .call(move |client| {
                let contact = client.contact(params.paciente_id)?;
                let mut dto = AppointmentDto::new(params.profissional_id, start, end);
                dto.contact_id = Some(params.paciente_id);
                dto.status = 1;
                dto.description = params
                    .descricao
                    .filter(|text| !text.trim().is_empty())
                    .unwrap_or(contact.name);
                dto.sms = contact.mobile;
                dto.procedure_id = params.procedimento_id;
                dto.diagnostic_id = params.diagnostico_id;
                client.create_appointment(&dto)
            })
            .await?;
        if !created {
            return Err(not_created("o agendamento"));
        }
        json_result(&serde_json::json!({ "criado": true }), &[])
    }

    /// Bloqueia um horário na agenda de um profissional (almoço, reunião,
    /// folga), sem paciente.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    pub async fn bloquear_horario(
        &self,
        Parameters(params): Parameters<BlockParams>,
    ) -> Result<CallToolResult, ToolError> {
        let (start, end) = check_time_range(&params.inicio, &params.fim)?;
        let created = self
            .medx
            .call(move |client| client.create_agenda_block(params.profissional_id, &start, &end))
            .await?;
        if !created {
            return Err(not_created("o bloqueio"));
        }
        json_result(&serde_json::json!({ "criado": true }), &[])
    }

    /// Muda o status de um agendamento (desmarcado, agendado, compareceu e
    /// os outros rótulos de `parametros_agenda`).
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn mudar_status_agendamento(
        &self,
        Parameters(params): Parameters<StatusParams>,
    ) -> Result<CallToolResult, ToolError> {
        self.medx
            .call(move |client| {
                client.update_appointment_status(params.agendamento_id, params.status)
            })
            .await?;
        json_result(
            &serde_json::json!({ "id": params.agendamento_id, "status": params.status }),
            &[],
        )
    }

    /// Muda o horário de um agendamento (e, se pedido, o profissional),
    /// mantendo paciente, descrição e o resto.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn remarcar_agendamento(
        &self,
        Parameters(params): Parameters<RescheduleParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_date("data", &params.data)?;
        let (start, end) = check_time_range(&params.novo_inicio, &params.novo_fim)?;
        let RescheduleParams {
            agendamento_id,
            profissional_id,
            data,
            novo_profissional_id,
            ..
        } = params;
        let lookup_date = data.clone();
        // Uma chamada só: achar o agendamento no dia e gravar a mudança.
        let found = self
            .medx
            .call(move |client| {
                let day = client.daily_agenda(profissional_id, &data)?;
                let Some(appointment) = day.into_iter().find(|a| a.id == agendamento_id) else {
                    return Ok(false);
                };
                let mut dto = AppointmentDto::from(appointment);
                dto.start = start;
                dto.end = end;
                if let Some(user_id) = novo_profissional_id {
                    dto.user_id = user_id;
                }
                client.update_appointment(&dto)?;
                Ok(true)
            })
            .await?;
        if !found {
            return Err(ToolError::Rejected(format!(
                "o agendamento {agendamento_id} não está na agenda do profissional {profissional_id} em {lookup_date}. Confira com `agenda_do_dia`"
            )));
        }
        json_result(&serde_json::json!({ "id": agendamento_id }), &[])
    }

    /// Manda ao paciente, pelo WhatsApp da clínica, o pedido de confirmação
    /// de um agendamento. A mensagem sai para o paciente na hora.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = true
    ))]
    pub async fn confirmar_agendamento_whatsapp(
        &self,
        Parameters(params): Parameters<AppointmentParams>,
    ) -> Result<CallToolResult, ToolError> {
        self.medx
            .call(move |client| client.confirm_appointment_whatsapp(params.agendamento_id))
            .await?;
        json_result(&serde_json::json!({ "id": params.agendamento_id }), &[])
    }
}
