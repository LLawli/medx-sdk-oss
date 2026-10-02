//! Chat interno da equipe.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{
    LimitParams, ToolError, check_limit, json_result, list_result, newest_first_by,
};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChatHistoryParams {
    /// Id do outro usuário da conversa (o `id` de `listar_usuarios_chat`).
    pub usuario_id: i64,
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

#[tool_router(router = chat_router, vis = "pub(crate)")]
impl MedxServer {
    /// Usuários do chat interno, se estão online e quantas mensagens não
    /// lidas cada um mandou (`unread`).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_usuarios_chat(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.chat_users()).await?;
        list_result(list, params.limite)
    }

    /// Conversa do chat interno com um usuário, da mensagem mais recente
    /// para a mais antiga.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn historico_chat(
        &self,
        Parameters(params): Parameters<ChatHistoryParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self
            .medx
            .call(move |client| client.chat_history(params.usuario_id))
            .await?;
        list_result(newest_first_by(list, |m| &m.date), params.limite)
    }

    /// Total de mensagens não lidas no chat interno.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn mensagens_nao_lidas_chat(&self) -> Result<CallToolResult, ToolError> {
        let total = self.medx.call(|client| client.chat_unread_count()).await?;
        json_result(&serde_json::json!({ "total": total }), &[])
    }

    /// Mensagens recebidas no chat interno que ainda não foram lidas.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn mensagens_recebidas_chat(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.chat_incoming()).await?;
        list_result(list, params.limite)
    }
}
