//! Usuários: o usuário logado e os usuários da clínica.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};

use crate::server::MedxServer;
use crate::tools::{LimitParams, ToolError, check_limit, json_result, list_result};

#[tool_router(router = usuarios_router, vis = "pub(crate)")]
impl MedxServer {
    /// Usuário logado na MedX: nome, e-mail, plano e bloqueios da conta.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn usuario_atual(&self) -> Result<CallToolResult, ToolError> {
        let user = self.medx.call(|client| client.current_user()).await?;
        json_result(&user, &["rd_station_key"])
    }

    /// Usuários da clínica (profissionais e equipe), com permissões e
    /// horários de trabalho. O `id` de um profissional é o que as
    /// ferramentas de agenda pedem.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_usuarios(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let users = self.medx.call(|client| client.users()).await?;
        list_result(users, params.limite)
    }
}
