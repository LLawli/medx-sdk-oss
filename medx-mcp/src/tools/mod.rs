//! Ferramentas MCP, um módulo por área do SDK.

pub mod agenda;
pub mod chat;
pub mod configuracoes;
pub mod financeiro;
pub mod hoje;
pub mod pacientes;
pub mod prontuario;
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

/// Parâmetros de uma ferramenta sobre um paciente.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PatientParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
}

/// Parâmetros de uma lista sobre um paciente.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PatientListParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
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

/// Ordena do mais recente para o mais antigo pela data em texto ISO que
/// `date` extrai de cada item. Vai antes do corte do `limite`, para o corte
/// ficar com os recentes. A ordenação é estável.
pub fn newest_first_by<T>(mut items: Vec<T>, date: impl Fn(&T) -> &str) -> Vec<T> {
    items.sort_by(|a, b| date(b).cmp(date(a)));
    items
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

/// Confere que `value` é uma data `AAAA-MM-DD` que existe no calendário
/// (mês de 1 a 12, dia dentro do mês, 29 de fevereiro só em ano bissexto).
/// O erro cita `field` e o formato.
pub fn check_date(field: &str, value: &str) -> Result<(), ToolError> {
    let invalid = || {
        ToolError::InvalidParams(format!(
            "`{field}` deve ser uma data válida no formato AAAA-MM-DD (recebido: {value:?})"
        ))
    };
    let bytes = value.as_bytes();
    let shape_ok = bytes.len() == 10
        && bytes.iter().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                *b == b'-'
            } else {
                b.is_ascii_digit()
            }
        });
    if !shape_ok {
        return Err(invalid());
    }
    // O formato já garantiu só dígitos ASCII nas posições abaixo.
    let year: u32 = value[0..4].parse().map_err(|_| invalid())?;
    let month: u32 = value[5..7].parse().map_err(|_| invalid())?;
    let day: u32 = value[8..10].parse().map_err(|_| invalid())?;
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return Err(invalid()),
    };
    if (1..=days_in_month).contains(&day) {
        Ok(())
    } else {
        Err(invalid())
    }
}

/// Confere as duas datas de um período (`inicio` e `fim`, nessa ordem) e que
/// `fim` não vem antes de `inicio`.
pub fn check_period(inicio: &str, fim: &str) -> Result<(), ToolError> {
    check_date("inicio", inicio)?;
    check_date("fim", fim)?;
    // Datas válidas têm largura fixa, então a ordem de texto é a cronológica.
    if fim < inicio {
        return Err(ToolError::InvalidParams(format!(
            "`fim` ({fim}) não pode ser anterior a `inicio` ({inicio})"
        )));
    }
    Ok(())
}
