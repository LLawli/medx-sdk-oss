//! Agenda: parâmetros, profissionais, agenda do dia e relatórios.

use medx::agenda::{AgendaReportDto, NoShowReportDto};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{
    LimitParams, ToolError, check_date, check_limit, check_period, json_result, list_result,
};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DayParams {
    /// Id do profissional (o `id` de `listar_profissionais_agenda`).
    pub profissional_id: i64,
    /// Data no formato AAAA-MM-DD.
    pub data: String,
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgendaReportParams {
    /// Data inicial, AAAA-MM-DD.
    pub inicio: String,
    /// Data final, AAAA-MM-DD, inclusive.
    pub fim: String,
    /// Id do profissional. Ausente ou 0: todos.
    #[serde(default)]
    pub profissional_id: Option<i64>,
    /// Incluir os agendamentos desmarcados. Padrão: não.
    #[serde(default)]
    pub incluir_desmarcados: Option<bool>,
    /// Texto para filtrar o relatório.
    #[serde(default)]
    pub busca: Option<String>,
    /// Só agendamentos com este status (a posição do rótulo em
    /// `parametros_agenda`). Ausente: todos.
    #[serde(default)]
    pub status: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NoShowReportParams {
    /// Data inicial, AAAA-MM-DD.
    pub inicio: String,
    /// Data final, AAAA-MM-DD, inclusive.
    pub fim: String,
    /// Id do profissional. Ausente ou 0: todos.
    #[serde(default)]
    pub profissional_id: Option<i64>,
}

#[tool_router(router = agenda_router, vis = "pub(crate)")]
impl MedxServer {
    /// Parâmetros da agenda: rótulos e cores de cada status (o número do
    /// status é a posição do rótulo), profissionais, setores, duração padrão
    /// e horário de funcionamento.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn parametros_agenda(&self) -> Result<CallToolResult, ToolError> {
        let params = self.medx.call(|client| client.agenda_parameters()).await?;
        json_result(&params, &[])
    }

    /// Profissionais com agenda na clínica, com setor e horário de trabalho
    /// de cada dia da semana.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_profissionais_agenda(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let users = self.medx.call(|client| client.agenda_users()).await?;
        list_result(users, params.limite)
    }

    /// Setores (salas, consultórios) da agenda.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_setores_agenda(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let sectors = self.medx.call(|client| client.agenda_sectors()).await?;
        list_result(sectors, params.limite)
    }

    /// Agendamentos de um profissional num dia, incluindo bloqueios de
    /// horário. `contact_id` é o paciente (0 ou ausente: bloqueio);
    /// `arrived_at` e `attended_at` em 0001-01-01 significam que ainda não
    /// aconteceu.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn agenda_do_dia(
        &self,
        Parameters(params): Parameters<DayParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_date("data", &params.data)?;
        check_limit(params.limite)?;
        let DayParams {
            profissional_id,
            data,
            limite,
        } = params;
        let list = self
            .medx
            .call(move |client| client.daily_agenda(profissional_id, &data))
            .await?;
        list_result(list, limite)
    }

    /// Só os bloqueios de horário (sem paciente) de um profissional num dia.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn bloqueios_do_dia(
        &self,
        Parameters(params): Parameters<DayParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_date("data", &params.data)?;
        check_limit(params.limite)?;
        let DayParams {
            profissional_id,
            data,
            limite,
        } = params;
        let list = self
            .medx
            .call(move |client| client.daily_blocks(profissional_id, &data))
            .await?;
        list_result(list, limite)
    }

    /// Gera na MedX o relatório da agenda de um período e devolve o link do
    /// PDF (`file_url`; vazio quando não há agendamentos).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn relatorio_agenda(
        &self,
        Parameters(params): Parameters<AgendaReportParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_period(&params.inicio, &params.fim)?;
        let mut dto = AgendaReportDto::new(
            &params.inicio,
            &params.fim,
            params.profissional_id.unwrap_or(0),
        );
        if let Some(show) = params.incluir_desmarcados {
            dto.show_cancelled = show;
        }
        if let Some(text) = params.busca {
            dto.search_text = text;
        }
        if let Some(status) = params.status {
            dto.status_id = status;
        }
        let report = self
            .medx
            .call(move |client| client.agenda_report(&dto))
            .await?;
        json_result(&report, &[])
    }

    /// Gera na MedX o relatório de faltas (no-show) de um período e devolve o
    /// link do PDF (`file_url`; vazio quando não há faltas).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn relatorio_faltas(
        &self,
        Parameters(params): Parameters<NoShowReportParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_period(&params.inicio, &params.fim)?;
        let dto = NoShowReportDto::new(
            &params.inicio,
            &params.fim,
            params.profissional_id.unwrap_or(0),
        );
        let report = self
            .medx
            .call(move |client| client.no_show_report(&dto))
            .await?;
        json_result(&report, &[])
    }
}
