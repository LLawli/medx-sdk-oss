//! Escrita no prontuário. Só com `MEDX_MCP_ALLOW_WRITE`.

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
}
