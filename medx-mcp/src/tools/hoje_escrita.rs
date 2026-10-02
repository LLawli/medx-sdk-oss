//! Escrita das notas do painel. Só com `MEDX_MCP_ALLOW_WRITE`.

use medx::{InsertNotaDto, UpdateNotaDto};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{ToolError, json_result, required_text};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateNoteParams {
    /// Texto da nota.
    pub texto: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct EditNoteParams {
    /// Id da nota (o `id` de `listar_notas`).
    pub nota_id: i64,
    /// Novo texto. Ausente: mantém o atual.
    #[serde(default)]
    pub texto: Option<String>,
    /// Marca a nota como concluída. Padrão: não.
    #[serde(default)]
    pub concluida: Option<bool>,
}

#[tool_router(router = hoje_escrita_router, vis = "pub(crate)")]
impl MedxServer {
    /// Cria uma nota no painel do usuário logado e devolve o `id`.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    pub async fn criar_nota(
        &self,
        Parameters(params): Parameters<CreateNoteParams>,
    ) -> Result<CallToolResult, ToolError> {
        let text = required_text("texto", &params.texto)?;
        let id = self
            .medx
            .call(move |client| {
                let me = client.current_user()?;
                client.insert_nota(&InsertNotaDto::new(me.user_id, &text))
            })
            .await?;
        json_result(&serde_json::json!({ "id": id }), &[])
    }

    /// Edita uma nota do painel: troca o texto e/ou marca como concluída.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn editar_nota(
        &self,
        Parameters(params): Parameters<EditNoteParams>,
    ) -> Result<CallToolResult, ToolError> {
        let text = params
            .texto
            .as_deref()
            .map(|text| required_text("texto", text))
            .transpose()?;
        let note_id = params.nota_id;
        let done = params.concluida.unwrap_or(false);
        let found = self
            .medx
            .call(move |client| {
                let Some(note) = client.notas()?.into_iter().find(|n| n.id == note_id) else {
                    return Ok(false);
                };
                let text = text.unwrap_or_else(|| note.text.clone());
                let mut dto = UpdateNotaDto::new(note_id, note.user_id, &text);
                dto.done = done;
                client.update_nota(&dto)?;
                Ok(true)
            })
            .await?;
        if !found {
            return Err(ToolError::Rejected(format!(
                "a nota {note_id} não está entre as notas do painel. Confira com `listar_notas`"
            )));
        }
        json_result(&serde_json::json!({ "id": note_id }), &[])
    }
}
