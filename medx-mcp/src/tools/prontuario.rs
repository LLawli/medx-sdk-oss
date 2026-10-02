//! Prontuário, e o que ele referencia: convênios, procedimentos,
//! formulários, módulos e unidades.

use medx::prontuario::ProntuarioReportDto;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{
    LimitParams, PatientListParams, PatientParams, ToolError, check_limit, check_period,
    json_result, list_result,
};

/// Dos mais recentes para os mais antigos. As datas são texto ISO, que
/// ordena como data; o corte do `limite` vem depois, para ficar com os
/// recentes.
fn newest_first(mut records: Vec<medx::MedicalRecord>) -> Vec<medx::MedicalRecord> {
    records.sort_by(|a, b| b.date.cmp(&a.date));
    records
}

/// `value` aparado, ou `InvalidParams` citando `field` quando fica vazio.
fn required_text<'a>(field: &str, value: &'a str) -> Result<&'a str, ToolError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ToolError::InvalidParams(format!(
            "`{field}` não pode ficar vazio"
        )));
    }
    Ok(value)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchRecordsParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
    /// Texto a procurar nos registros.
    pub texto: String,
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FileParams {
    /// O campo `classe` do registro de prontuário que tem arquivo.
    pub classe: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConvenioParams {
    /// Id do convênio (o `id` de `listar_convenios`).
    pub convenio_id: i64,
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FormParams {
    /// Id do formulário (o `id` de `listar_formularios`).
    pub formulario_id: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ModuleParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
    /// Nome do módulo, por exemplo `anamnese` ou `evolucao`.
    pub modulo: String,
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProntuarioReportParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
    /// Data inicial, AAAA-MM-DD.
    pub inicio: String,
    /// Data final, AAAA-MM-DD, inclusive.
    pub fim: String,
}

#[tool_router(router = prontuario_router, vis = "pub(crate)")]
impl MedxServer {
    /// Sumário fixo do paciente: diagnóstico, história patológica pregressa
    /// (hpp), medicamentos, alergias e campo livre.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn sumario_prontuario(
        &self,
        Parameters(params): Parameters<PatientParams>,
    ) -> Result<CallToolResult, ToolError> {
        let summary = self
            .medx
            .call(move |client| client.medical_history_summary(params.paciente_id))
            .await?;
        json_result(&summary, &[])
    }

    /// Registros do prontuário de um paciente, do mais recente para o mais
    /// antigo. `content` é HTML. Registros com arquivo têm `tipo_doc` e
    /// `classe` (use `link_arquivo_prontuario`).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn ver_prontuario(
        &self,
        Parameters(params): Parameters<PatientListParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let records = self
            .medx
            .call(move |client| client.medical_records(params.paciente_id))
            .await?;
        list_result(newest_first(records), params.limite)
    }

    /// Fotos e imagens do prontuário de um paciente, da mais recente para a
    /// mais antiga. Use `link_arquivo_prontuario` com a `classe` para abrir.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn galeria_de_fotos(
        &self,
        Parameters(params): Parameters<PatientListParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let records = self
            .medx
            .call(move |client| client.photo_gallery(params.paciente_id))
            .await?;
        list_result(newest_first(records), params.limite)
    }

    /// Busca um texto nos registros do prontuário de um paciente; resultado
    /// do mais recente para o mais antigo.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn buscar_no_prontuario(
        &self,
        Parameters(params): Parameters<SearchRecordsParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let text = required_text("texto", &params.texto)?.to_owned();
        let patient_id = params.paciente_id;
        let records = self
            .medx
            .call(move |client| client.search_medical_records(patient_id, &text))
            .await?;
        list_result(newest_first(records), params.limite)
    }

    /// Link temporário para abrir o arquivo (PDF, imagem) de um registro do
    /// prontuário.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn link_arquivo_prontuario(
        &self,
        Parameters(params): Parameters<FileParams>,
    ) -> Result<CallToolResult, ToolError> {
        let class = required_text("classe", &params.classe)?.to_owned();
        let url = self
            .medx
            .call(move |client| client.resolve_file_url(&class))
            .await?;
        json_result(&serde_json::json!({ "url": url }), &[])
    }

    /// Palavras-chave cadastradas para classificar registros do prontuário.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn palavras_chave_prontuario(&self) -> Result<CallToolResult, ToolError> {
        let keywords = self.medx.call(|client| client.medical_keywords()).await?;
        let list: Vec<String> = keywords.as_list().into_iter().map(str::to_owned).collect();
        json_result(&list, &[])
    }

    /// Convênios cadastrados na clínica.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_convenios(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.convenios()).await?;
        list_result(list, params.limite)
    }

    /// Procedimentos de um convênio, com o valor pago por ele.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn procedimentos_do_convenio(
        &self,
        Parameters(params): Parameters<ConvenioParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let convenio_id = params.convenio_id;
        let list = self
            .medx
            .call(move |client| client.convenio_procedures(convenio_id))
            .await?;
        list_result(list, params.limite)
    }

    /// Procedimentos da clínica, com preço base, comissão e sessões.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_procedimentos(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.procedures()).await?;
        list_result(list, params.limite)
    }

    /// Formulários do prontuário (anamnese, evolução e outros).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_formularios(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.forms()).await?;
        list_result(list, params.limite)
    }

    /// HTML de um formulário do prontuário.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn ver_formulario(
        &self,
        Parameters(params): Parameters<FormParams>,
    ) -> Result<CallToolResult, ToolError> {
        let html = self
            .medx
            .call(move |client| client.form_html(params.formulario_id))
            .await?;
        Ok(CallToolResult::success(vec![ContentBlock::text(html)]))
    }

    /// Registros de um módulo personalizado do prontuário para um paciente.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn registros_do_modulo(
        &self,
        Parameters(params): Parameters<ModuleParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let module = required_text("modulo", &params.modulo)?.to_owned();
        let patient_id = params.paciente_id;
        let list = self
            .medx
            .call(move |client| client.module_records(patient_id, &module))
            .await?;
        list_result(list, params.limite)
    }

    /// Gera na MedX o PDF do prontuário de um paciente num período e devolve
    /// o link (`file_url`).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn relatorio_prontuario(
        &self,
        Parameters(params): Parameters<ProntuarioReportParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_period(&params.inicio, &params.fim)?;
        // Uma chamada só: o nome do paciente e o relatório não podem se
        // intercalar com outras chamadas.
        let report = self
            .medx
            .call(move |client| {
                let contact = client.contact(params.paciente_id)?;
                let dto = ProntuarioReportDto::new(
                    params.paciente_id,
                    contact.name,
                    &params.inicio,
                    &params.fim,
                );
                client.prontuario_report(&dto)
            })
            .await?;
        json_result(&report, &[])
    }

    /// Unidades de negócio (filiais) da clínica.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_unidades(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.business_units()).await?;
        list_result(list, params.limite)
    }
}
