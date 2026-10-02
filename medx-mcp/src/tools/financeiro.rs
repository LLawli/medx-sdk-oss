//! Financeiro: faturas de atendimento.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{PatientListParams, ToolError, check_limit, list_result, newest_first_by};

/// Filtros da lista de atendimentos, os mesmos do webapp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttendanceFilter {
    /// Atendimentos dos últimos 7 dias.
    #[serde(rename = "ultimos_7_dias")]
    Ultimos7Dias,
    /// Faturas com pagamento pendente.
    Pendencias,
    /// Faturas canceladas.
    FaturasCanceladas,
    /// Orçamentos ainda não convertidos em fatura.
    OrcamentosEmAberto,
}

impl AttendanceFilter {
    /// O valor que a MedX espera em `filter`.
    pub fn api_value(self) -> &'static str {
        match self {
            Self::Ultimos7Dias => "Últimos 7 Dias",
            Self::Pendencias => "Pendências",
            Self::FaturasCanceladas => "Faturas Canceladas",
            Self::OrcamentosEmAberto => "Orçamentos em aberto",
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AttendancesParams {
    /// Filtro do webapp. Ausente: todos.
    #[serde(default)]
    pub filtro: Option<AttendanceFilter>,
    /// Texto de busca livre (nome do paciente, recibo).
    #[serde(default)]
    pub busca: Option<String>,
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

#[tool_router(router = financeiro_router, vis = "pub(crate)")]
impl MedxServer {
    /// Faturas de atendimento de um paciente, da mais recente para a mais
    /// antiga: valor, total pago, desconto, se está fechada (`closed`) e se
    /// é orçamento (`budget`).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn atendimentos_do_paciente(
        &self,
        Parameters(params): Parameters<PatientListParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self
            .medx
            .call(move |client| client.attendances_by_patient(params.paciente_id))
            .await?;
        list_result(newest_first_by(list, |a| &a.date), params.limite)
    }

    /// Faturas de atendimento da clínica, da mais recente para a mais
    /// antiga, com os filtros do webapp e busca livre.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_atendimentos(
        &self,
        Parameters(params): Parameters<AttendancesParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let filter = params.filtro.map(AttendanceFilter::api_value).unwrap_or("");
        let search = params.busca.unwrap_or_default();
        let list = self
            .medx
            .call(move |client| client.all_attendances(filter, &search))
            .await?;
        list_result(newest_first_by(list, |a| &a.date), params.limite)
    }
}
