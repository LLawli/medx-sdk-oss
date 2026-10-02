//! Escrita de pacientes: cadastro e atualização. Só com
//! `MEDX_MCP_ALLOW_WRITE`.

use medx::ContactDto;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::MedxServer;
use crate::tools::{ToolError, check_date, json_result};

/// Campos do cadastro que o modelo pode preencher. Ausente: não muda (na
/// atualização) ou fica vazio (no cadastro).
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct PatientFields {
    /// Nome social.
    #[serde(default)]
    pub nome_social: Option<String>,
    /// Sexo: M ou F.
    #[serde(default)]
    pub sexo: Option<String>,
    /// Data de nascimento, AAAA-MM-DD.
    #[serde(default)]
    pub nascimento: Option<String>,
    /// CPF, só números.
    #[serde(default)]
    pub cpf: Option<String>,
    /// RG.
    #[serde(default)]
    pub rg: Option<String>,
    /// E-mail.
    #[serde(default)]
    pub email: Option<String>,
    /// Celular com DDD, só números.
    #[serde(default)]
    pub celular: Option<String>,
    /// Telefone residencial com DDD.
    #[serde(default)]
    pub telefone: Option<String>,
    /// Endereço residencial (rua e número).
    #[serde(default)]
    pub endereco: Option<String>,
    /// Bairro.
    #[serde(default)]
    pub bairro: Option<String>,
    /// Cidade.
    #[serde(default)]
    pub cidade: Option<String>,
    /// UF, duas letras.
    #[serde(default)]
    pub estado: Option<String>,
    /// CEP, só números.
    #[serde(default)]
    pub cep: Option<String>,
    /// Profissão.
    #[serde(default)]
    pub profissao: Option<String>,
    /// Estado civil.
    #[serde(default)]
    pub estado_civil: Option<String>,
    /// Id do plano de saúde (o `id` de `listar_planos_de_saude`).
    #[serde(default)]
    pub convenio_id: Option<i64>,
    /// Número da carteirinha do plano.
    #[serde(default)]
    pub numero_carteirinha: Option<String>,
    /// Nome da mãe.
    #[serde(default)]
    pub mae: Option<String>,
    /// Nome do pai.
    #[serde(default)]
    pub pai: Option<String>,
    /// Quem indicou o paciente.
    #[serde(default)]
    pub indicado_por: Option<String>,
    /// Observações do cadastro.
    #[serde(default)]
    pub observacoes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreatePatientParams {
    /// Nome completo.
    pub nome: String,
    #[serde(flatten)]
    pub campos: PatientFields,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdatePatientParams {
    /// Id do paciente (o `id` de `buscar_pacientes`).
    pub paciente_id: i64,
    /// Nome completo.
    #[serde(default)]
    pub nome: Option<String>,
    #[serde(flatten)]
    pub campos: PatientFields,
}

/// Valida os campos e deixa `sexo` em maiúscula e `nascimento` na forma que a
/// MedX espera. Roda antes de qualquer chamada à MedX.
fn normalize(mut fields: PatientFields) -> Result<PatientFields, ToolError> {
    if let Some(sexo) = &fields.sexo {
        fields.sexo = Some(match sexo.trim().to_ascii_lowercase().as_str() {
            "m" => "M".to_owned(),
            "f" => "F".to_owned(),
            _ => {
                return Err(ToolError::InvalidParams(format!(
                    "`sexo` deve ser M ou F (recebido: {sexo:?})"
                )));
            }
        });
    }
    if let Some(nascimento) = &fields.nascimento {
        check_date("nascimento", nascimento)?;
        fields.nascimento = Some(format!("{nascimento}T00:00:00"));
    }
    Ok(fields)
}

/// `nome` aparado, ou `InvalidParams` citando o campo quando fica vazio.
fn clean_name(nome: &str) -> Result<String, ToolError> {
    let nome = nome.trim();
    if nome.is_empty() {
        return Err(ToolError::InvalidParams(
            "`nome` não pode ficar vazio".to_owned(),
        ));
    }
    Ok(nome.to_owned())
}

/// Copia para `dto` só os campos presentes e devolve quantos eram. Os campos
/// já passaram por [`normalize`].
fn apply(dto: &mut ContactDto, fields: PatientFields) -> usize {
    let mut applied = 0;
    macro_rules! set {
        ($field:ident => $target:expr) => {
            if let Some(value) = fields.$field {
                $target = value;
                applied += 1;
            }
        };
    }
    set!(nome_social => dto.social_name);
    set!(sexo => dto.gender);
    if let Some(nascimento) = fields.nascimento {
        dto.birth_date = Some(nascimento);
        applied += 1;
    }
    set!(cpf => dto.cpf);
    set!(rg => dto.rg);
    set!(email => dto.email);
    set!(celular => dto.mobile);
    set!(telefone => dto.phone_home);
    set!(endereco => dto.address_home);
    set!(bairro => dto.neighborhood_home);
    set!(cidade => dto.city_home);
    set!(estado => dto.state_home);
    set!(cep => dto.zip_home);
    set!(profissao => dto.profession);
    set!(estado_civil => dto.marital_status);
    set!(convenio_id => dto.insurance_id);
    set!(numero_carteirinha => dto.insurance_number);
    set!(mae => dto.mother);
    set!(pai => dto.father);
    set!(indicado_por => dto.referred_by);
    set!(observacoes => dto.notes);
    applied
}

#[tool_router(router = pacientes_escrita_router, vis = "pub(crate)")]
impl MedxServer {
    /// Cadastra um paciente novo e devolve o `id`. Antes, confira com
    /// `pacientes_homonimos` (ou `buscar_pacientes`) se ele já existe.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    pub async fn cadastrar_paciente(
        &self,
        Parameters(params): Parameters<CreatePatientParams>,
    ) -> Result<CallToolResult, ToolError> {
        let name = clean_name(&params.nome)?;
        let fields = normalize(params.campos)?;
        let mut dto = ContactDto::new(name);
        dto.contact_type = "Paciente".to_owned();
        apply(&mut dto, fields);
        let id = self
            .medx
            .call(move |client| client.create_contact(&dto))
            .await?;
        json_result(&serde_json::json!({ "id": id }), &[])
    }

    /// Atualiza o cadastro de um paciente. Só os campos informados mudam; o
    /// resto fica como está na MedX.
    #[tool(annotations(
        read_only_hint = false,
        destructive_hint = true,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    pub async fn atualizar_paciente(
        &self,
        Parameters(params): Parameters<UpdatePatientParams>,
    ) -> Result<CallToolResult, ToolError> {
        let name = params.nome.as_deref().map(clean_name).transpose()?;
        let fields = normalize(params.campos)?;
        // Ensaio só para contar os campos; o DTO real vem da ficha da MedX.
        let present = apply(&mut ContactDto::new(""), fields.clone());
        if name.is_none() && present == 0 {
            return Err(ToolError::InvalidParams(
                "nenhum campo para atualizar: informe `nome` ou algum dos campos do cadastro"
                    .to_owned(),
            ));
        }
        let patient_id = params.paciente_id;
        // Uma chamada só: ler a ficha e gravar sem outra chamada no meio.
        self.medx
            .call(move |client| {
                let mut dto = ContactDto::from(client.contact(patient_id)?);
                if let Some(name) = name {
                    dto.name = name;
                }
                apply(&mut dto, fields);
                client.update_contact(&dto)
            })
            .await?;
        json_result(&serde_json::json!({ "id": patient_id }), &[])
    }
}
