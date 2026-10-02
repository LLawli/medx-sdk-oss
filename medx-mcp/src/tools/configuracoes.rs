//! Marketing, configurações e ajustes da clínica.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{LimitParams, ToolError, check_limit, json_result, list_result};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DocFoldersParams {
    /// Filtro das pastas (o `filter_key` de uma pasta). Ausente: todas.
    #[serde(default)]
    pub filtro: Option<String>,
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

#[tool_router(router = configuracoes_router, vis = "pub(crate)")]
impl MedxServer {
    /// Eventos de marketing cadastrados (lembretes, retornos, campanhas).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_eventos(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.events()).await?;
        list_result(list, params.limite)
    }

    /// Questionários que a clínica envia aos pacientes.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_questionarios(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.quests()).await?;
        list_result(list, params.limite)
    }

    /// Diagnósticos de queixa principal (QP) cadastrados, com o tempo de
    /// consulta de cada um. O `id` é o `diagnostic_id` dos agendamentos.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_diagnosticos_qp(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.diagnostico_qp()).await?;
        list_result(list, params.limite)
    }

    /// Modelos das mensagens que a clínica manda aos pacientes (SMS,
    /// WhatsApp, pré-cadastro, questionário) e as redes sociais.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn modelos_de_mensagem(&self) -> Result<CallToolResult, ToolError> {
        let value = self.medx.call(|client| client.client_settings()).await?;
        json_result(&value, &[])
    }

    /// Parâmetros gerais da clínica: horário de funcionamento, duração
    /// padrão, remetente de SMS, metas nutricionais e atalhos.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn parametros_gerais(&self) -> Result<CallToolResult, ToolError> {
        let value = self.medx.call(|client| client.general_parameters()).await?;
        json_result(&value, &[])
    }

    /// Rótulos e cores dos status de agendamento.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn parametros_de_cores(&self) -> Result<CallToolResult, ToolError> {
        let value = self.medx.call(|client| client.color_parameters()).await?;
        json_result(&value, &[])
    }

    /// Relatórios disponíveis na MedX.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_relatorios(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.list_reports()).await?;
        list_result(list, params.limite)
    }

    /// Pastas de documentos automáticos (modelos de documento da clínica).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn pastas_de_documentos(
        &self,
        Parameters(params): Parameters<DocFoldersParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let filter = params.filtro.unwrap_or_default();
        let list = self
            .medx
            .call(move |client| client.doc_folders(&filter))
            .await?;
        list_result(list, params.limite)
    }

    /// Configuração do calendário ICS da agenda (endereço e se está ativo).
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn calendario_ics(&self) -> Result<CallToolResult, ToolError> {
        let value = self.medx.call(|client| client.ics_config()).await?;
        json_result(&value, &["token"])
    }
}
