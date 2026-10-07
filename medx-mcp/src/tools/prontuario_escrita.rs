//! Escrita no prontuário. Só com `MEDX_MCP_ALLOW_WRITE`.

use std::path::PathBuf;

use medx::MedicalRecordDto;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{ToolError, check_datetime, json_result, required_text, text_to_html};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateRecordParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
    /// Texto do registro, em texto puro (linha em branco separa parágrafos).
    pub texto: String,
    /// Data e hora do registro, AAAA-MM-DDTHH:MM, hora local. Padrão: agora.
    #[serde(default)]
    pub data: Option<String>,
    /// Palavras-chave do registro, separadas por vírgula (as de
    /// `palavras_chave_prontuario`).
    #[serde(default)]
    pub palavras_chave: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct EditRecordParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
    /// Id do registro (o `id` de `ver_prontuario`).
    pub registro_id: i64,
    /// Novo texto do registro, em texto puro. Substitui o anterior.
    pub texto: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SummaryParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
    /// Diagnóstico.
    #[serde(default)]
    pub diagnostico: Option<String>,
    /// História patológica pregressa.
    #[serde(default)]
    pub hpp: Option<String>,
    /// Medicamentos em uso.
    #[serde(default)]
    pub medicamentos: Option<String>,
    /// Alergias.
    #[serde(default)]
    pub alergias: Option<String>,
    /// Campo livre.
    #[serde(default)]
    pub livre: Option<String>,
}

/// Maior arquivo que `anexar_ao_prontuario` envia. A MedX recusa corpo
/// acima de 30.000.000 bytes (medido em 2026-10-07; ver docs/decisoes.md), e
/// o arquivo vai em base64, que cresce 4/3, dentro do JSON.
pub const MAX_ATTACHMENT_BYTES: u64 = 22_000_000;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AttachParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
    /// Caminho absoluto do arquivo no computador onde o servidor roda. Até
    /// 22 MB.
    pub arquivo: String,
    /// Descrição do anexo, como aparece no prontuário.
    pub descricao: String,
}

#[tool_router(router = prontuario_escrita_router, vis = "pub(crate)")]
impl MedxServer {
    /// Registra uma evolução no prontuário de um paciente, em nome do
    /// usuário logado. O texto puro vira o HTML que o prontuário guarda.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    pub async fn registrar_no_prontuario(
        &self,
        Parameters(params): Parameters<CreateRecordParams>,
    ) -> Result<CallToolResult, ToolError> {
        let text = required_text("texto", &params.texto)?;
        let date = match params.data.as_deref() {
            Some(date) => check_datetime("data", date)?,
            None => medx::util::current_datetime_str(),
        };
        let keywords = params
            .palavras_chave
            .map(|k| k.trim().to_owned())
            .filter(|k| !k.is_empty());
        let patient_id = params.paciente_id;
        self.medx
            .call(move |client| {
                let me = client.current_user()?;
                let mut dto =
                    MedicalRecordDto::new(patient_id, me.user_id, text_to_html(&text), date);
                if let Some(keywords) = keywords {
                    dto.keywords = keywords;
                }
                client.create_medical_record(&dto)
            })
            .await?;
        json_result(&serde_json::json!({ "paciente_id": patient_id }), &[])
    }

    /// Substitui o texto de um registro do prontuário, mantendo data, autor
    /// e palavras-chave.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn editar_registro_prontuario(
        &self,
        Parameters(params): Parameters<EditRecordParams>,
    ) -> Result<CallToolResult, ToolError> {
        let text = required_text("texto", &params.texto)?;
        let EditRecordParams {
            paciente_id,
            registro_id,
            ..
        } = params;
        let found = self
            .medx
            .call(move |client| {
                let records = client.medical_records(paciente_id)?;
                let Some(record) = records.into_iter().find(|r| r.id == registro_id) else {
                    return Ok(false);
                };
                let mut dto = MedicalRecordDto::from(record);
                dto.content = text_to_html(&text);
                client.update_medical_record(&dto)?;
                Ok(true)
            })
            .await?;
        if !found {
            return Err(ToolError::Rejected(format!(
                "o registro {registro_id} não está no prontuário do paciente {paciente_id}. Confira com `ver_prontuario`"
            )));
        }
        json_result(&serde_json::json!({ "id": registro_id }), &[])
    }

    /// Atualiza o sumário fixo do paciente. Só os campos informados mudam.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn atualizar_sumario_prontuario(
        &self,
        Parameters(params): Parameters<SummaryParams>,
    ) -> Result<CallToolResult, ToolError> {
        let SummaryParams {
            paciente_id,
            diagnostico,
            hpp,
            medicamentos,
            alergias,
            livre,
        } = params;
        if [&diagnostico, &hpp, &medicamentos, &alergias, &livre]
            .iter()
            .all(|field| field.is_none())
        {
            return Err(ToolError::InvalidParams(
                "nenhum campo para atualizar: informe `diagnostico`, `hpp`, `medicamentos`, `alergias` ou `livre`"
                    .to_owned(),
            ));
        }
        self.medx
            .call(move |client| {
                let mut summary = client.medical_history_summary(paciente_id)?;
                if let Some(value) = diagnostico {
                    summary.diagnostic = value;
                }
                if let Some(value) = hpp {
                    summary.hpp = value;
                }
                if let Some(value) = medicamentos {
                    summary.medications = value;
                }
                if let Some(value) = alergias {
                    summary.allergies = value;
                }
                if let Some(value) = livre {
                    summary.free_text = value;
                }
                client.upsert_medical_history_summary(paciente_id, &summary)
            })
            .await?;
        json_result(&serde_json::json!({ "paciente_id": paciente_id }), &[])
    }

    /// Anexa um arquivo do computador ao prontuário de um paciente (PDF,
    /// imagem, documento; até 22 MB). Recebe o caminho absoluto do arquivo,
    /// não o conteúdo.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    pub async fn anexar_ao_prontuario(
        &self,
        Parameters(params): Parameters<AttachParams>,
    ) -> Result<CallToolResult, ToolError> {
        let description = required_text("descricao", &params.descricao)?;
        let path = PathBuf::from(&params.arquivo);
        if !path.is_absolute() {
            return Err(ToolError::InvalidParams(format!(
                "`arquivo` precisa ser um caminho absoluto, recebi `{}`",
                params.arquivo
            )));
        }
        let metadata = std::fs::metadata(&path).map_err(|e| {
            ToolError::InvalidParams(format!(
                "não foi possível ler o arquivo `{}`: {e}",
                params.arquivo
            ))
        })?;
        if !metadata.is_file() {
            return Err(ToolError::InvalidParams(format!(
                "`{}` não é um arquivo",
                params.arquivo
            )));
        }
        let size = metadata.len();
        if size == 0 {
            return Err(ToolError::InvalidParams(format!(
                "o arquivo `{}` está vazio",
                params.arquivo
            )));
        }
        if size > MAX_ATTACHMENT_BYTES {
            return Err(ToolError::InvalidParams(format!(
                "o arquivo tem {} bytes, acima do limite de {} bytes",
                group_thousands(size),
                group_thousands(MAX_ATTACHMENT_BYTES)
            )));
        }
        let Some(file_name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned)
        else {
            return Err(ToolError::InvalidParams(format!(
                "`{}` não tem um nome de arquivo válido",
                params.arquivo
            )));
        };

        let read_path = path.clone();
        let data = tokio::task::spawn_blocking(move || std::fs::read(read_path))
            .await
            .map_err(|e| {
                ToolError::Rejected(format!("a leitura do arquivo foi interrompida: {e}"))
            })?
            .map_err(|e| {
                ToolError::InvalidParams(format!(
                    "não foi possível ler o arquivo `{}`: {e}",
                    params.arquivo
                ))
            })?;
        let sent = data.len();

        let patient_id = params.paciente_id;
        let name = file_name.clone();
        let response = self
            .medx
            .call(move |client| {
                let file = medx::ArquivoDto::from_bytes(
                    &name,
                    medx::ArquivoDto::filetype_for(&name),
                    &data,
                );
                let dto = medx::AttachFilesDto::new(patient_id, description, vec![file]);
                client.attach_files(&dto)
            })
            .await?;
        if response.trim() != "Success" {
            return Err(ToolError::Rejected(format!(
                "a MedX não anexou o arquivo e respondeu: {response}"
            )));
        }
        json_result(
            &serde_json::json!({ "paciente_id": patient_id, "arquivo": file_name, "tamanho": sent }),
            &[],
        )
    }
}

/// Número com ponto de milhar (`22000000` vira `22.000.000`).
fn group_thousands(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(ch);
    }
    out
}
