//! Pacientes (contatos da MedX): busca, ficha, homônimos, planos e foto.

use medx::contacts::ContactSearchGroup;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{
    LimitParams, PatientParams, ToolError, check_date, check_limit, json_result, list_result,
};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchPatientsParams {
    /// Nome ou parte do nome. Obrigatório, a não ser com
    /// `aniversariantes_do_mes`.
    #[serde(default)]
    pub nome: Option<String>,
    /// Só os aniversariantes do mês corrente.
    #[serde(default)]
    pub aniversariantes_do_mes: Option<bool>,
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct HomonymParams {
    /// Nome completo.
    pub nome: String,
    /// Sexo: M ou F.
    pub sexo: String,
    /// Data de nascimento, AAAA-MM-DD.
    pub nascimento: String,
    /// Máximo de itens na resposta (padrão 50, até 500).
    #[serde(default)]
    pub limite: Option<u32>,
}

#[tool_router(router = pacientes_router, vis = "pub(crate)")]
impl MedxServer {
    /// Busca pacientes pelo nome (ou lista os aniversariantes do mês), com
    /// contato e plano de saúde. Use o `id` nas outras ferramentas de
    /// paciente.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn buscar_pacientes(
        &self,
        Parameters(params): Parameters<SearchPatientsParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let birthdays = params.aniversariantes_do_mes.unwrap_or(false);
        let name = params.nome.as_deref().map(str::trim).unwrap_or_default();
        if name.is_empty() && !birthdays {
            return Err(ToolError::InvalidParams(
                "`nome` é obrigatório, a não ser com `aniversariantes_do_mes`".to_owned(),
            ));
        }
        let name = name.to_owned();
        let group = if birthdays {
            ContactSearchGroup::BirthdayThisMonth
        } else {
            ContactSearchGroup::All
        };
        let list = self
            .medx
            .call(move |client| client.search_contacts(&name, group, 1))
            .await?;
        list_result(list, params.limite)
    }

    /// Ficha completa de um paciente: documentos, contatos, endereço,
    /// convênio e observações.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn ver_paciente(
        &self,
        Parameters(params): Parameters<PatientParams>,
    ) -> Result<CallToolResult, ToolError> {
        let contact = self
            .medx
            .call(move |client| client.contact(params.paciente_id))
            .await?;
        json_result(&contact, &[])
    }

    /// Pacientes já cadastrados com o mesmo nome, sexo e nascimento. Use
    /// antes de cadastrar alguém, para não duplicar.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn pacientes_homonimos(
        &self,
        Parameters(params): Parameters<HomonymParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let gender = match params.sexo.trim().to_ascii_lowercase().as_str() {
            "m" => "M",
            "f" => "F",
            _ => {
                return Err(ToolError::InvalidParams(format!(
                    "`sexo` deve ser M ou F (recebido: {:?})",
                    params.sexo
                )));
            }
        };
        check_date("nascimento", &params.nascimento)?;
        let HomonymParams {
            nome,
            nascimento,
            limite,
            ..
        } = params;
        let list = self
            .medx
            .call(move |client| client.homonym_contacts(&nome, gender, &nascimento))
            .await?;
        list_result(list, limite)
    }

    /// Planos de saúde (convênios) aceitos no cadastro de pacientes.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn listar_planos_de_saude(
        &self,
        Parameters(params): Parameters<LimitParams>,
    ) -> Result<CallToolResult, ToolError> {
        check_limit(params.limite)?;
        let list = self.medx.call(|client| client.insurance_plans()).await?;
        list_result(list, params.limite)
    }

    /// Foto do paciente, como imagem.
    #[tool(annotations(read_only_hint = true, open_world_hint = false))]
    pub async fn foto_paciente(
        &self,
        Parameters(params): Parameters<PatientParams>,
    ) -> Result<CallToolResult, ToolError> {
        let raw = self
            .medx
            .call(move |client| client.contact_photo_base64(params.paciente_id))
            .await?;
        Ok(photo_result(&raw))
    }
}

/// Resposta da foto a partir do corpo cru da MedX: base64 puro, base64
/// entre aspas JSON ou data URL (`data:<mime>;base64,<dados>`). O tipo da
/// imagem vem da data URL ou da assinatura do base64 (JPEG `/9j/`, PNG
/// `iVBOR`, GIF `R0lGOD`, WebP `UklGR`; sem assinatura conhecida, JPEG).
/// Corpo vazio ou `null` (o que a MedX real responde para paciente sem foto)
/// vira um texto dizendo que o paciente não tem foto.
pub fn photo_result(raw: &str) -> CallToolResult {
    let mut body = raw.trim();
    // A MedX às vezes devolve o base64 como string JSON.
    let unquoted: String;
    if body.starts_with('"')
        && let Ok(inner) = serde_json::from_str::<String>(body)
    {
        unquoted = inner;
        body = unquoted.trim();
    }
    if body.is_empty() || body == "null" {
        return CallToolResult::success(vec![ContentBlock::text("O paciente não tem foto.")]);
    }

    let (declared_mime, data) = match body
        .strip_prefix("data:")
        .and_then(|rest| rest.split_once(','))
    {
        Some((header, data)) => {
            let mime = header.split(';').next().unwrap_or_default().trim();
            ((!mime.is_empty()).then(|| mime.to_owned()), data.trim())
        }
        None => (None, body),
    };
    let mime = declared_mime.unwrap_or_else(|| sniff_mime(data).to_owned());
    CallToolResult::success(vec![ContentBlock::image(data.to_owned(), mime)])
}

/// Tipo da imagem pela assinatura do base64; JPEG quando não reconhece.
fn sniff_mime(data: &str) -> &'static str {
    if data.starts_with("iVBOR") {
        "image/png"
    } else if data.starts_with("R0lGOD") {
        "image/gif"
    } else if data.starts_with("UklGR") {
        "image/webp"
    } else {
        "image/jpeg"
    }
}
