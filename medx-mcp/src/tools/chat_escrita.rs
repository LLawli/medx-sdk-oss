//! Escrita no chat interno. Só com `MEDX_MCP_ALLOW_WRITE`.

use medx::SendMessageDto;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{ToolError, json_result};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SendMessageParams {
    /// Id do destinatário (o `id` de `listar_usuarios_chat`).
    pub usuario_id: i64,
    /// Texto da mensagem.
    pub texto: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MarkReadParams {
    /// Ids das mensagens (o `id` de `mensagens_recebidas_chat` ou de
    /// `historico_chat`).
    pub ids: Vec<i64>,
}

#[tool_router(router = chat_escrita_router, vis = "pub(crate)")]
impl MedxServer {
    /// Manda uma mensagem no chat interno da equipe, em nome do usuário
    /// logado.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    pub async fn enviar_mensagem_chat(
        &self,
        Parameters(params): Parameters<SendMessageParams>,
    ) -> Result<CallToolResult, ToolError> {
        let text = params.texto.trim().to_owned();
        if text.is_empty() {
            return Err(ToolError::InvalidParams(
                "`texto` não pode ficar vazio".to_owned(),
            ));
        }
        let to_id = params.usuario_id;
        self.medx
            .call(move |client| {
                let me = client.current_user()?;
                let message = SendMessageDto::new(
                    me.user_id,
                    me.full_name,
                    to_id,
                    "",
                    text,
                    medx::util::current_datetime_str(),
                );
                client.send_chat_message(&message)
            })
            .await?;
        json_result(&serde_json::json!({ "enviada": true }), &[])
    }

    /// Marca mensagens do chat interno como lidas.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn marcar_mensagens_lidas(
        &self,
        Parameters(params): Parameters<MarkReadParams>,
    ) -> Result<CallToolResult, ToolError> {
        if params.ids.is_empty() {
            return Err(ToolError::InvalidParams(
                "`ids` não pode ficar vazio".to_owned(),
            ));
        }
        let ids = params.ids;
        let sent = ids.clone();
        self.medx
            .call(move |client| client.mark_messages_read(&ids))
            .await?;
        json_result(&serde_json::json!({ "ids": sent }), &[])
    }
}
