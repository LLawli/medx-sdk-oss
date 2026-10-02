//! Ferramentas MCP, um módulo por área do SDK.

pub mod usuarios;

use rmcp::handler::server::tool::IntoCallToolResult;
use rmcp::model::{CallToolResponse, CallToolResult, ContentBlock};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Itens por resposta quando `limite` não vem.
pub const DEFAULT_LIMIT: u32 = 50;
/// Maior `limite` aceito.
pub const MAX_LIMIT: u32 = 500;

/// Por que uma chamada de ferramenta não produziu resultado. Toda variante
/// chega ao cliente como resultado com `isError: true`, com a mensagem.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Parâmetro bem formado mas inválido para a ferramenta. Barrado antes de
    /// qualquer chamada à MedX.
    #[error("{0}")]
    InvalidParams(String),
    /// Sem sessão salva e sem credenciais.
    #[error(
        "sem sessão na MedX: defina MEDX_LOGIN_CREDENTIAL e MEDX_PASSWORD_CREDENTIAL na configuração do servidor MCP, ou faça login com `medx-cli auth login <email> <senha>`"
    )]
    NoSession,
    /// A MedX ou o SDK falharam.
    #[error("falha na MedX: {0}")]
    Medx(#[from] medx::MedxError),
    /// A thread da conexão acabou (não deveria acontecer).
    #[error("a conexão com a MedX foi encerrada")]
    WorkerGone,
    #[error("não foi possível serializar a resposta: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoCallToolResult for ToolError {
    fn into_call_tool_result(self) -> Result<CallToolResponse, rmcp::ErrorData> {
        Ok(CallToolResult::error(vec![ContentBlock::text(self.to_string())]).into())
    }
}

/// Parâmetros de uma ferramenta de lista sem outro argumento.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct LimitParams {
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

/// Resposta com um objeto: um bloco de texto com o JSON compacto de `value`,
/// sem os campos de `omit` (segredos; ver docs/decisoes.md).
pub fn json_result<T: Serialize>(value: &T, omit: &[&str]) -> Result<CallToolResult, ToolError> {
    let mut value = serde_json::to_value(value)?;
    if let Some(object) = value.as_object_mut() {
        for key in omit {
            object.remove(*key);
        }
    }
    Ok(CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string(&value)?,
    )]))
}

/// Resposta com uma lista: um bloco com o array JSON compacto dos primeiros
/// `limite` itens. Quando a lista passa do limite, um segundo bloco avisa
/// "Mostrando N de M itens." e como pedir mais (`limite`, até 500, ou uma
/// busca mais específica). `limite` fora de 1 a 500 é `InvalidParams`.
pub fn list_result<T: Serialize>(
    items: Vec<T>,
    limite: Option<u32>,
) -> Result<CallToolResult, ToolError> {
    let limit = check_limit(limite)?;
    let total = items.len();
    let shown: Vec<&T> = items.iter().take(limit).collect();
    let mut blocks = vec![ContentBlock::text(serde_json::to_string(&shown)?)];
    if total > limit {
        blocks.push(ContentBlock::text(format!(
            "Mostrando {limit} de {total} itens. Para ver mais, use `limite` (até {MAX_LIMIT}) ou refine a busca."
        )));
    }
    Ok(CallToolResult::success(blocks))
}

/// Valida `limite` (1 a 500; ausente vale 50). Usado antes de chamar a MedX,
/// para um limite inválido não custar uma request.
pub fn check_limit(limite: Option<u32>) -> Result<usize, ToolError> {
    match limite {
        None => Ok(DEFAULT_LIMIT as usize),
        Some(n) if (1..=MAX_LIMIT).contains(&n) => Ok(n as usize),
        Some(n) => Err(ToolError::InvalidParams(format!(
            "`limite` deve ficar entre 1 e {MAX_LIMIT} (recebido: {n})"
        ))),
    }
}
