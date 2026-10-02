//! Painel "hoje": notificações do dia, últimos atendidos e notas.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};

use crate::server::MedxServer;
use crate::tools::{LimitParams, ToolError, check_limit, list_result};

#[tool_router(router = hoje_router, vis = "pub(crate)")]
impl MedxServer {
    /// Notificações do painel do dia (aniversários, retornos e avisos).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn notificacoes_de_hoje(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.hoje_notificacoes()).await?;
        list_result(list, params.limite)
    }

    /// Últimos pacientes atendidos, com a data do último atendimento.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn ultimos_atendidos(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.ultimos_atendidos()).await?;
        list_result(list, params.limite)
    }

    /// Notas do painel do usuário logado.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_notas(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.notas()).await?;
        list_result(list, params.limite)
    }
}
