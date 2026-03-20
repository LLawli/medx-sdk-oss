//! Etapa 5 — Prontuário, convênios, procedimentos, formulários, módulos e unidades de negócio.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{client::MedxClient, error::MedxError};

// ── Deserializadores auxiliares ───────────────────────────────────────────────

fn de_null_str<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    use serde::de::Visitor;
    struct AnyToStr;
    impl<'de> Visitor<'de> for AnyToStr {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "string, número ou null")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_string<E: serde::de::Error>(self, v: String) -> Result<String, E> { Ok(v) }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_unit<E: serde::de::Error>(self) -> Result<String, E> { Ok(String::new()) }
        fn visit_none<E: serde::de::Error>(self) -> Result<String, E> { Ok(String::new()) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<String, D2::Error> {
            d.deserialize_any(AnyToStr)
        }
    }
    d.deserialize_any(AnyToStr)
}

fn de_null_i64<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    use serde::de::Visitor;
    struct I64OrStr;
    impl<'de> Visitor<'de> for I64OrStr {
        type Value = i64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "i64, string de número ou null")
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<i64, E> { Ok(v) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<i64, E> { Ok(v as i64) }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<i64, E> { Ok(v as i64) }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<i64, E> {
            v.trim().parse::<i64>().map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<i64, E> { Ok(0) }
        fn visit_none<E: serde::de::Error>(self) -> Result<i64, E> { Ok(0) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<i64, D2::Error> {
            d.deserialize_any(I64OrStr)
        }
    }
    d.deserialize_any(I64OrStr)
}

fn de_null_f64<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    use serde::de::Visitor;
    struct F64OrStr;
    impl<'de> Visitor<'de> for F64OrStr {
        type Value = f64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "f64, string de número ou null")
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<f64, E> { Ok(v as f64) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<f64, E> { Ok(v as f64) }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<f64, E> { Ok(v) }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<f64, E> {
            v.trim().replace(',', ".").parse::<f64>()
                .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<f64, E> { Ok(0.0) }
        fn visit_none<E: serde::de::Error>(self) -> Result<f64, E> { Ok(0.0) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<f64, D2::Error> {
            d.deserialize_any(F64OrStr)
        }
    }
    d.deserialize_any(F64OrStr)
}

// ── Tipos — Prontuário ────────────────────────────────────────────────────────

/// Sumário fixo do paciente (diagnóstico, HPP, medicamentos, alergias, campo livre).
///
/// Obtido via `GET prontuario/GetMedicalHistorySummary?Pacid=`.
/// Atualizado via `POST prontuario/InsertOrUpdateMedicalHistorySummary?intPacid=`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MedicalHistorySummary {
    #[serde(rename = "diagnostico", deserialize_with = "de_null_str")]
    pub diagnostic: String,
    #[serde(rename = "hpp", deserialize_with = "de_null_str")]
    pub hpp: String,
    #[serde(rename = "medicamentos", deserialize_with = "de_null_str")]
    pub medications: String,
    #[serde(rename = "alergias", deserialize_with = "de_null_str")]
    pub allergies: String,
    #[serde(rename = "livre", deserialize_with = "de_null_str")]
    pub free_text: String,
}

/// Registro individual de prontuário.
///
/// Retornado por `GET prontuario/GetProntuario?PacId=` e `GET prontuario/GetProntuarioBusca`.
///
/// Registros com arquivo têm `tipo_doc` preenchido (ex: `"pdf"`, `"jpg"`) e `classe`
/// contendo o nome do blob no armazenamento MedX (ex: `"4242-uuid.pdf"`).
/// Use `file_url(base_url)` para obter a URL completa do arquivo.
#[derive(Debug, Clone, Deserialize)]
pub struct MedicalRecord {
    #[serde(rename = "Id_do_Historico", deserialize_with = "de_null_i64")]
    pub id: i64,
    #[serde(rename = "Id_da_Assinatura", deserialize_with = "de_null_i64")]
    pub subscription_id: i64,
    #[serde(rename = "Id_do_Cliente", deserialize_with = "de_null_i64")]
    pub patient_id: i64,
    /// Conteúdo HTML ou texto do registro.
    #[serde(rename = "Historico", deserialize_with = "de_null_str")]
    pub content: String,
    #[serde(rename = "Data", deserialize_with = "de_null_str")]
    pub date: String,
    #[serde(rename = "Id_do_Usuario", deserialize_with = "de_null_i64")]
    pub user_id: i64,
    /// Nome do blob de arquivo (ex: `"4242-uuid.pdf"`). Vazio se não houver arquivo.
    #[serde(default, rename = "Classe", deserialize_with = "de_null_str")]
    pub classe: String,
    #[serde(default, rename = "Palavraschave", deserialize_with = "de_null_str")]
    pub keywords: String,
    /// Nome do profissional que criou o registro.
    #[serde(default, rename = "Usuario", deserialize_with = "de_null_str")]
    pub usuario: String,
    /// Tipo do arquivo anexo: `"pdf"`, `"jpg"`, `"png"`, etc. Vazio para registros sem arquivo.
    #[serde(default, rename = "TipoDoc", deserialize_with = "de_null_str")]
    pub tipo_doc: String,
    /// Data da última edição do registro.
    #[serde(default, rename = "LastEditDate", deserialize_with = "de_null_str")]
    pub last_edit_date: String,
}

impl MedicalRecord {
    /// Retorna `true` se o registro tem um arquivo anexado (PDF, imagem, etc.).
    pub fn has_file(&self) -> bool {
        !self.classe.is_empty()
    }

    /// Constrói a URL completa do arquivo a partir da base_url do cliente MedX.
    ///
    /// O padrão é: `{base_url_sem_api}/medxdata/{classe}`
    ///
    /// Exemplo: `https://v65.medx.med.br/medxdata/4242-uuid.pdf`
    pub fn file_url(&self, base_url: &str) -> Option<String> {
        if self.classe.is_empty() {
            return None;
        }
        // base_url é "https://host/api" — remove o sufixo "/api"
        let host = base_url.trim_end_matches('/').trim_end_matches("api").trim_end_matches('/');
        Some(format!("{}/medxdata/{}", host, self.classe))
    }
}

/// DTO para criar ou atualizar um registro de prontuário.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MedicalRecordDto {
    #[serde(rename = "Id_do_Historico")]
    pub id: i64,
    #[serde(rename = "Id_da_Assinatura")]
    pub subscription_id: i64,
    #[serde(rename = "Id_do_Cliente")]
    pub patient_id: i64,
    #[serde(rename = "Historico")]
    pub content: String,
    #[serde(rename = "Data")]
    pub date: String,
    #[serde(rename = "Id_do_Usuario")]
    pub user_id: i64,
    #[serde(rename = "Classe")]
    pub class: String,
    #[serde(rename = "Palavraschave")]
    pub keywords: String,
}

impl MedicalRecordDto {
    /// Cria um DTO mínimo para novo registro.
    ///
    /// - `patient_id`: ID do paciente
    /// - `user_id`: ID do profissional
    /// - `content`: HTML do prontuário
    /// - `date`: ISO 8601, ex: `"2026-03-20T09:00:00"`
    pub fn new(patient_id: i64, user_id: i64, content: impl Into<String>, date: impl Into<String>) -> Self {
        MedicalRecordDto {
            patient_id,
            user_id,
            content: content.into(),
            date: date.into(),
            ..Default::default()
        }
    }
}

impl From<MedicalRecord> for MedicalRecordDto {
    fn from(r: MedicalRecord) -> Self {
        MedicalRecordDto {
            id: r.id,
            subscription_id: r.subscription_id,
            patient_id: r.patient_id,
            content: r.content,
            date: r.date,
            user_id: r.user_id,
            class: r.classe,
            keywords: r.keywords,
        }
    }
}

/// Palavras-chave médicas da conta.
///
/// A API retorna `[{"Keywords": "kw1,kw2,..."}]`.
/// Use `MedicalKeywords::as_list()` para obter um `Vec<String>` das palavras individuais.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MedicalKeywords {
    #[serde(rename = "Keywords", deserialize_with = "de_null_str")]
    pub raw: String,
}

impl MedicalKeywords {
    /// Divide `raw` por vírgulas e retorna as palavras-chave individuais sem espaços extras.
    pub fn as_list(&self) -> Vec<&str> {
        self.raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect()
    }
}

// ── Tipos — Convênios ─────────────────────────────────────────────────────────

/// Convênio cadastrado na conta (`GET Convenios/GetAllConvenios`).
#[derive(Debug, Clone, Deserialize)]
pub struct Convenio {
    #[serde(rename = "IddoConvenio", deserialize_with = "de_null_i64")]
    pub id: i64,
    #[serde(rename = "Convenio", deserialize_with = "de_null_str")]
    pub name: String,
    #[serde(default, rename = "Ativo", deserialize_with = "de_null_str")]
    pub active: String,
}

/// Procedimento vinculado a um convênio (`GET Convenios/GetProcedimentosByIdConvenio`).
#[derive(Debug, Clone, Deserialize)]
pub struct ConvenioProcedure {
    #[serde(rename = "IddoProcedimento", deserialize_with = "de_null_i64")]
    pub id: i64,
    #[serde(rename = "Procedimento", deserialize_with = "de_null_str")]
    pub name: String,
    #[serde(rename = "Valor", deserialize_with = "de_null_f64")]
    pub price: f64,
    #[serde(default, rename = "Sessoes", deserialize_with = "de_null_i64")]
    pub sessions: i64,
}

// ── Tipos — Procedimentos ─────────────────────────────────────────────────────

/// Procedimento global da conta (`GET Procedimentos/GetAllProcedimentos`).
#[derive(Debug, Clone, Deserialize)]
pub struct Procedure {
    #[serde(rename = "IddoProcedimento", deserialize_with = "de_null_i64")]
    pub id: i64,
    #[serde(rename = "Procedimento", deserialize_with = "de_null_str")]
    pub name: String,
    #[serde(default, rename = "Comissao", deserialize_with = "de_null_f64")]
    pub commission: f64,
    #[serde(rename = "PrecoBase", deserialize_with = "de_null_f64")]
    pub base_price: f64,
    #[serde(default, rename = "Sessoes", deserialize_with = "de_null_i64")]
    pub sessions: i64,
}

// ── Tipos — Formulários ───────────────────────────────────────────────────────

/// Formulário disponível na conta (`GET formularios/getformularios`).
#[derive(Debug, Clone, Deserialize)]
pub struct Form {
    #[serde(rename = "Id", deserialize_with = "de_null_i64")]
    pub id: i64,
    /// Nome / identificador do arquivo do formulário.
    #[serde(rename = "Arquivo", deserialize_with = "de_null_str")]
    pub name: String,
}

// ── Tipos — Módulos ───────────────────────────────────────────────────────────

/// Registro de módulo personalizado de um paciente (`GET modulos/GetRecords`).
#[derive(Debug, Clone, Deserialize)]
pub struct ModuleRecord {
    #[serde(default, rename = "Id", deserialize_with = "de_null_i64")]
    pub id: i64,
    #[serde(default, rename = "Modulo", deserialize_with = "de_null_str")]
    pub module: String,
    #[serde(default, rename = "Dados", deserialize_with = "de_null_str")]
    pub data: String,
    #[serde(default, rename = "Data", deserialize_with = "de_null_str")]
    pub date: String,
    #[serde(default, rename = "Id_do_Cliente", deserialize_with = "de_null_i64")]
    pub patient_id: i64,
}

// ── Tipos — Unidades de negócio ───────────────────────────────────────────────

/// DTO para anexar arquivos ao prontuário (`POST prontuario/AttachFiles`).
#[derive(Debug, Clone, Serialize)]
pub struct AttachFilesDto {
    #[serde(rename = "Id_do_Cliente")]
    pub patient_id: i64,
    #[serde(rename = "Descricao")]
    pub descricao: String,
    pub arquivos: Vec<ArquivoDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArquivoDto {
    pub file_base64: String,
    pub filename: String,
    pub filesize: u64,
    pub filetype: String,
}

impl AttachFilesDto {
    pub fn new(patient_id: i64, descricao: impl Into<String>, arquivos: Vec<ArquivoDto>) -> Self {
        AttachFilesDto { patient_id, descricao: descricao.into(), arquivos }
    }
}

impl ArquivoDto {
    /// Cria um `ArquivoDto` a partir dos bytes do arquivo.
    ///
    /// - `filename`: nome original do arquivo (a extensão é preservada).
    /// - `filetype`: MIME type (ex: `"application/pdf"`, `"image/jpeg"`).
    pub fn from_bytes(filename: impl Into<String>, filetype: impl Into<String>, data: &[u8]) -> Self {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        ArquivoDto {
            file_base64: STANDARD.encode(data),
            filename: filename.into(),
            filesize: data.len() as u64,
            filetype: filetype.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessUnit {
    #[serde(rename = "IddaUnidadedeNegocios", deserialize_with = "de_null_i64")]
    pub id: i64,
    #[serde(rename = "UnidadedeNegocios", deserialize_with = "de_null_str")]
    pub name: String,
    #[serde(default, rename = "CPFCNPJ", deserialize_with = "de_null_str")]
    pub cpf_cnpj: String,
    #[serde(default, rename = "Municipio", deserialize_with = "de_null_str")]
    pub city: String,
    #[serde(default, rename = "UF", deserialize_with = "de_null_str")]
    pub state: String,
}

// ── Métodos do MedxClient ─────────────────────────────────────────────────────

impl MedxClient {
    // ── Prontuário ────────────────────────────────────────────────────────────

    /// Retorna o sumário fixo do paciente (diagnóstico, HPP, medicamentos, alergias).
    pub fn medical_history_summary(&self, patient_id: i64) -> Result<MedicalHistorySummary, MedxError> {
        let text = self.get_text(&format!(
            "prontuario/GetMedicalHistorySummary?Pacid={patient_id}"
        ))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(MedicalHistorySummary::default());
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Cria ou atualiza o sumário fixo do paciente.
    pub fn upsert_medical_history_summary(
        &self,
        patient_id: i64,
        summary: &MedicalHistorySummary,
    ) -> Result<(), MedxError> {
        self.post::<_, serde_json::Value>(
            &format!("prontuario/InsertOrUpdateMedicalHistorySummary?intPacid={patient_id}"),
            summary,
        )?;
        Ok(())
    }

    /// Resolve a URL de acesso direto a um arquivo de prontuário (PDF, imagem, etc.).
    ///
    /// Chama `GET azure/getfileurl?blobname={classe}` — o mesmo endpoint que o frontend web usa.
    /// Retorna a URL pública autenticada do blob no Azure (`medxdata.blob.core.windows.net/...`).
    ///
    /// # Exemplo
    /// ```no_run
    /// # let c: medx::MedxClient = todo!();
    /// let url = c.resolve_file_url("4242-00000000-0000-4000-8000-000000000001.pdf").unwrap();
    /// println!("{url}"); // https://medxdata.blob.core.windows.net/...
    /// ```
    pub fn resolve_file_url(&self, classe: &str) -> Result<String, MedxError> {
        let raw = self.get_text(&format!("azure/getfileurl?blobname={classe}"))?;
        // A API retorna a URL crua (sem aspas JSON). Remove espaços e aspas extras.
        let url = raw.trim().trim_matches('"').to_string();
        if url.contains("medxdata.blob.core.windows.net") {
            Ok(url)
        } else {
            // URL relativa — prefixar com a origem do servidor
            let host = self.base_url.trim_end_matches('/').trim_end_matches("api").trim_end_matches('/');
            Ok(format!("{}/{}", host, url.trim_start_matches('/')))
        }
    }

    /// Retorna os registros de prontuário de um paciente.
    pub fn medical_records(&self, patient_id: i64) -> Result<Vec<MedicalRecord>, MedxError> {
        let text = self.get_text(&format!("prontuario/GetProntuario?PacId={patient_id}"))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Retorna os registros de foto/imagem do prontuário (galeria).
    ///
    /// Chama o mesmo endpoint que `medical_records` com `&galeria=true`.
    /// Retorna apenas registros com `TipoDoc == "img"`.
    pub fn photo_gallery(&self, patient_id: i64) -> Result<Vec<MedicalRecord>, MedxError> {
        let text = self.get_text(&format!(
            "prontuario/GetProntuario?PacId={patient_id}&galeria=true"
        ))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Busca registros de prontuário de um paciente por texto livre.
    pub fn search_medical_records(
        &self,
        patient_id: i64,
        query: &str,
    ) -> Result<Vec<MedicalRecord>, MedxError> {
        let text = self.get_text(&format!(
            "prontuario/GetProntuarioBusca?PacId={patient_id}&busca={}",
            urlencoding_simple(query)
        ))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Retorna as palavras-chave médicas cadastradas na conta.
    ///
    /// Retorna `MedicalKeywords::default()` se o endpoint não estiver disponível.
    pub fn medical_keywords(&self) -> Result<MedicalKeywords, MedxError> {
        match self.get_text("prontuario/GetMedicalKeywords") {
            Err(MedxError::Api { status: 500, .. }) => return Ok(MedicalKeywords::default()),
            Err(e) => return Err(e),
            Ok(text) => {
                if text.trim() == "null" || text.trim().is_empty() {
                    return Ok(MedicalKeywords::default());
                }
                // API retorna array de um único objeto: [{"Keywords": "..."}]
                let arr: Vec<MedicalKeywords> = serde_json::from_str(&text).map_err(MedxError::Json)?;
                Ok(arr.into_iter().next().unwrap_or_default())
            }
        }
    }

    /// Cria um novo registro de prontuário.
    pub fn create_medical_record(&self, dto: &MedicalRecordDto) -> Result<(), MedxError> {
        self.post::<_, serde_json::Value>("prontuario/InsertMedicalHistory", dto)?;
        Ok(())
    }

    /// Atualiza um registro de prontuário existente.
    ///
    /// O campo `dto.id` deve ser o ID do registro a ser atualizado.
    pub fn update_medical_record(&self, dto: &MedicalRecordDto) -> Result<(), MedxError> {
        self.put::<_, serde_json::Value>("prontuario/UpdateMedicalHistory", dto)?;
        Ok(())
    }

    // ── Convênios ─────────────────────────────────────────────────────────────

    /// Retorna todos os convênios cadastrados na conta.
    pub fn convenios(&self) -> Result<Vec<Convenio>, MedxError> {
        let text = self.get_text("Convenios/GetAllConvenios")?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Retorna os procedimentos associados a um convênio.
    pub fn convenio_procedures(&self, convenio_id: i64) -> Result<Vec<ConvenioProcedure>, MedxError> {
        let text = self.get_text(&format!(
            "Convenios/GetProcedimentosByIdConvenio?IddoConvenio={convenio_id}"
        ))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    // ── Procedimentos ─────────────────────────────────────────────────────────

    /// Retorna todos os procedimentos cadastrados na conta.
    pub fn procedures(&self) -> Result<Vec<Procedure>, MedxError> {
        let text = self.get_text("Procedimentos/GetAllProcedimentos")?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    // ── Formulários ───────────────────────────────────────────────────────────

    /// Retorna os formulários personalizados da conta.
    pub fn forms(&self) -> Result<Vec<Form>, MedxError> {
        let text = self.get_text("formularios/getformularios")?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Retorna o HTML de um formulário pelo ID.
    pub fn form_html(&self, form_id: i64) -> Result<String, MedxError> {
        self.get_text(&format!(
            "formularios/GetFormulariosHTML?iddoformulario={form_id}"
        ))
    }

    // ── Módulos ───────────────────────────────────────────────────────────────

    /// Retorna os registros de um módulo personalizado para um paciente.
    ///
    /// - `patient_id`: ID do paciente
    /// - `module`: nome do módulo (ex: `"anamnese"`, `"evolucao"`)
    pub fn module_records(&self, patient_id: i64, module: &str) -> Result<Vec<ModuleRecord>, MedxError> {
        let text = self.get_text(&format!(
            "modulos/GetRecords?pacid={patient_id}&modulo={}",
            urlencoding_simple(module)
        ))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    // ── Upload de arquivos ────────────────────────────────────────────────────

    /// Anexa arquivos ao prontuário de um paciente.
    ///
    /// Retorna `"Success"` em caso de sucesso.
    pub fn attach_files(&self, dto: &AttachFilesDto) -> Result<String, MedxError> {
        self.post::<_, String>("prontuario/AttachFiles", dto)
    }

    // ── Unidades de negócio ───────────────────────────────────────────────────

    /// Retorna todas as unidades de negócio da conta.
    pub fn business_units(&self) -> Result<Vec<BusinessUnit>, MedxError> {
        let text = self.get_text("UN/GetAllUN")?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }
}

// ── Helpers internos ──────────────────────────────────────────────────────────

/// Percent-encoding mínimo para parâmetros de query string.
fn urlencoding_simple(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const SUMMARY_JSON: &str = r#"{
        "diagnostico": "Hipertensão arterial",
        "hpp": "Cirurgia cardíaca em 2010",
        "medicamentos": "Losartana 50mg",
        "alergias": "Penicilina",
        "livre": "Paciente sedentário"
    }"#;

    pub const RECORD_JSON: &str = r#"{
        "Id_do_Historico": 1001,
        "Id_da_Assinatura": 4242,
        "Id_do_Cliente": 100002,
        "Historico": "<p>Paciente sem queixas.</p>",
        "Data": "2026-03-17T09:00:00",
        "Id_do_Usuario": 7,
        "Classe": "",
        "Palavraschave": "rotina"
    }"#;

    pub const CONVENIO_JSON: &str = r#"{
        "IddoConvenio": 1,
        "Convenio": "UNIMED",
        "Ativo": "true"
    }"#;

    pub const PROCEDURE_JSON: &str = r#"{
        "IddoProcedimento": 10,
        "Procedimento": "CONSULTA",
        "Comissao": 10.0,
        "PrecoBase": 150.0,
        "Sessoes": 1
    }"#;

    pub const CONVENIO_PROCEDURE_JSON: &str = r#"{
        "IddoProcedimento": 10,
        "Procedimento": "CONSULTA",
        "Valor": 200.0,
        "Sessoes": 1
    }"#;

    pub const FORM_JSON: &str = r#"{
        "Id": 5,
        "Arquivo": "formulariopadrao"
    }"#;

    pub const BUSINESS_UNIT_JSON: &str = r#"{
        "IddaUnidadedeNegocios": 1001,
        "UnidadedeNegocios": "CLINICA PRINCIPAL",
        "CPFCNPJ": "12345678000100",
        "Municipio": "GOIANIA",
        "UF": "GO"
    }"#;

    #[test]
    fn deserializa_medical_history_summary() {
        let s: MedicalHistorySummary = serde_json::from_str(SUMMARY_JSON).unwrap();
        assert_eq!(s.diagnostic, "Hipertensão arterial");
        assert_eq!(s.hpp, "Cirurgia cardíaca em 2010");
        assert_eq!(s.medications, "Losartana 50mg");
        assert_eq!(s.allergies, "Penicilina");
        assert_eq!(s.free_text, "Paciente sedentário");
    }

    #[test]
    fn summary_default_para_null() {
        let s: MedicalHistorySummary = serde_json::from_str(
            r#"{"diagnostico": null, "hpp": null, "medicamentos": null, "alergias": null, "livre": null}"#
        ).unwrap();
        assert!(s.diagnostic.is_empty());
        assert!(s.medications.is_empty());
    }

    #[test]
    fn deserializa_medical_record() {
        let r: MedicalRecord = serde_json::from_str(RECORD_JSON).unwrap();
        assert_eq!(r.id, 1001);
        assert_eq!(r.patient_id, 100002);
        assert_eq!(r.content, "<p>Paciente sem queixas.</p>");
        assert_eq!(r.keywords, "rotina");
        assert_eq!(r.user_id, 7);
    }

    #[test]
    fn record_into_dto_preserva_campos() {
        let r: MedicalRecord = serde_json::from_str(RECORD_JSON).unwrap();
        let dto: MedicalRecordDto = r.into();
        assert_eq!(dto.id, 1001);
        assert_eq!(dto.patient_id, 100002);
        assert_eq!(dto.content, "<p>Paciente sem queixas.</p>");
    }

    #[test]
    fn record_dto_new_defaults() {
        let dto = MedicalRecordDto::new(123, 42, "<p>ok</p>", "2026-03-17T09:00:00");
        assert_eq!(dto.id, 0);
        assert_eq!(dto.patient_id, 123);
        assert_eq!(dto.user_id, 42);
        assert_eq!(dto.content, "<p>ok</p>");
        assert!(dto.keywords.is_empty());
    }

    #[test]
    fn deserializa_convenio() {
        let c: Convenio = serde_json::from_str(CONVENIO_JSON).unwrap();
        assert_eq!(c.id, 1);
        assert_eq!(c.name, "UNIMED");
    }

    #[test]
    fn deserializa_procedure() {
        let p: Procedure = serde_json::from_str(PROCEDURE_JSON).unwrap();
        assert_eq!(p.id, 10);
        assert_eq!(p.name, "CONSULTA");
        assert!((p.base_price - 150.0).abs() < f64::EPSILON);
        assert_eq!(p.sessions, 1);
    }

    #[test]
    fn deserializa_convenio_procedure() {
        let p: ConvenioProcedure = serde_json::from_str(CONVENIO_PROCEDURE_JSON).unwrap();
        assert_eq!(p.id, 10);
        assert!((p.price - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deserializa_form() {
        let f: Form = serde_json::from_str(FORM_JSON).unwrap();
        assert_eq!(f.id, 5);
        assert_eq!(f.name, "formulariopadrao");
    }

    #[test]
    fn deserializa_business_unit() {
        let bu: BusinessUnit = serde_json::from_str(BUSINESS_UNIT_JSON).unwrap();
        assert_eq!(bu.id, 1001);
        assert_eq!(bu.name, "CLINICA PRINCIPAL");
        assert_eq!(bu.city, "GOIANIA");
        assert_eq!(bu.state, "GO");
    }

    #[test]
    fn urlencoding_caracteres_especiais() {
        assert_eq!(urlencoding_simple("JOAO SILVA"), "JOAO+SILVA");
        assert_eq!(urlencoding_simple("abc123"), "abc123");
        assert_eq!(urlencoding_simple("exame/resultado"), "exame%2Fresultado");
    }

    #[test]
    fn medical_keywords_as_list() {
        let kw = MedicalKeywords { raw: "hipertensão,diabetes,obesidade".to_string() };
        let list = kw.as_list();
        assert_eq!(list, vec!["hipertensão", "diabetes", "obesidade"]);

        let empty = MedicalKeywords::default();
        assert!(empty.as_list().is_empty());
    }

    #[test]
    fn medical_keywords_desserializa_formato_api() {
        // Formato real: [{"Keywords":""}]
        let arr: Vec<MedicalKeywords> = serde_json::from_str(r#"[{"Keywords":"a,b,c"}]"#).unwrap();
        assert_eq!(arr[0].as_list(), vec!["a", "b", "c"]);
    }

    #[test]
    fn serializa_dto_tem_campos_api() {
        let dto = MedicalRecordDto::new(1, 2, "x", "2026-01-01T00:00:00");
        let v = serde_json::to_value(&dto).unwrap();
        assert!(v.get("Id_do_Historico").is_some());
        assert!(v.get("Id_do_Cliente").is_some());
        assert!(v.get("Historico").is_some());
        assert!(v.get("Data").is_some());
    }
}
