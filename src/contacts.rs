//! Etapa 3 — Contatos (entidade central do sistema).
//!
//! Todos os outros módulos (agenda, prontuário, finanças) referenciam contatos.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{client::MedxClient, error::MedxError, util::encode_query_value};

// ── Deserializadores auxiliares ───────────────────────────────────────────────

fn de_null_str<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    use serde::de::Visitor;
    struct AnyToStr;
    impl<'de> Visitor<'de> for AnyToStr {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "string, número, ou null")
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
            write!(f, "i64, string de número, ou null")
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<i64, E> { Ok(v) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<i64, E> { Ok(v as i64) }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<i64, E> { Ok(v as i64) }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<i64, E> {
            v.trim().parse::<i64>().map_err(|_| E::invalid_value(
                serde::de::Unexpected::Str(v), &self,
            ))
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<i64, E> { Ok(0) }
        fn visit_none<E: serde::de::Error>(self) -> Result<i64, E> { Ok(0) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<i64, D2::Error> {
            d.deserialize_any(I64OrStr)
        }
    }
    d.deserialize_any(I64OrStr)
}

fn de_null_bool<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    let opt: Option<bool> = Option::deserialize(d)?;
    Ok(opt.unwrap_or(false))
}

// ── Tipos ─────────────────────────────────────────────────────────────────────

/// Ficha completa de um contato/paciente (`GET contatos/GetContatosFichaById`).
///
/// A API retorna um array com um único elemento; este tipo representa esse elemento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    #[serde(rename(deserialize = "Id_do_Cliente"))]
    pub id: i64,
    #[serde(rename(deserialize = "Nome"), deserialize_with = "de_null_str")]
    pub name: String,
    #[serde(rename(deserialize = "Nome_Social"), deserialize_with = "de_null_str")]
    pub social_name: String,
    #[serde(rename(deserialize = "Sexo"), deserialize_with = "de_null_str")]
    pub gender: String,
    #[serde(rename(deserialize = "Nascimento"))]
    pub birth_date: Option<String>,
    #[serde(rename(deserialize = "CPF_CGC"), deserialize_with = "de_null_str")]
    pub cpf: String,
    #[serde(rename(deserialize = "RG"), deserialize_with = "de_null_str")]
    pub rg: String,
    #[serde(rename(deserialize = "Email"), deserialize_with = "de_null_str")]
    pub email: String,
    #[serde(rename(deserialize = "Celular"), deserialize_with = "de_null_str")]
    pub mobile: String,
    #[serde(rename(deserialize = "Telefone_Residencial"), deserialize_with = "de_null_str")]
    pub phone_home: String,
    #[serde(rename(deserialize = "Telefone_Residencial_1"), deserialize_with = "de_null_str")]
    pub phone_home_2: String,
    #[serde(rename(deserialize = "Telefone_Comercial"), deserialize_with = "de_null_str")]
    pub phone_work: String,
    #[serde(rename(deserialize = "Endereco_Residencial"), deserialize_with = "de_null_str")]
    pub address_home: String,
    #[serde(rename(deserialize = "Bairro_Residencial"), deserialize_with = "de_null_str")]
    pub neighborhood_home: String,
    #[serde(rename(deserialize = "Cidade_Residencial"), deserialize_with = "de_null_str")]
    pub city_home: String,
    #[serde(rename(deserialize = "Estado_Residencial"), deserialize_with = "de_null_str")]
    pub state_home: String,
    #[serde(rename(deserialize = "Cep_Residencial"), deserialize_with = "de_null_str")]
    pub zip_home: String,
    #[serde(rename(deserialize = "Pais_Residencial"), deserialize_with = "de_null_str")]
    pub country_home: String,
    #[serde(rename(deserialize = "Endereco_Comercial"), deserialize_with = "de_null_str")]
    pub address_work: String,
    #[serde(rename(deserialize = "Bairro_Comercial"), deserialize_with = "de_null_str")]
    pub neighborhood_work: String,
    #[serde(rename(deserialize = "Cidade_Comercial"), deserialize_with = "de_null_str")]
    pub city_work: String,
    #[serde(rename(deserialize = "Estado_Comercial"), deserialize_with = "de_null_str")]
    pub state_work: String,
    #[serde(rename(deserialize = "Cep_Comercial"), deserialize_with = "de_null_str")]
    pub zip_work: String,
    #[serde(rename(deserialize = "Pais_Comercial"), deserialize_with = "de_null_str")]
    pub country_work: String,
    #[serde(rename(deserialize = "Profissao"), deserialize_with = "de_null_str")]
    pub profession: String,
    #[serde(rename(deserialize = "Empresa"), deserialize_with = "de_null_str")]
    pub company: String,
    #[serde(rename(deserialize = "Estado_Civil"), deserialize_with = "de_null_str")]
    pub marital_status: String,
    #[serde(rename(deserialize = "Tipo"), deserialize_with = "de_null_str")]
    pub contact_type: String,
    #[serde(rename(deserialize = "Observacoes"), deserialize_with = "de_null_str")]
    pub notes: String,
    #[serde(rename(deserialize = "Mae"), deserialize_with = "de_null_str")]
    pub mother: String,
    #[serde(rename(deserialize = "Pai"), deserialize_with = "de_null_str")]
    pub father: String,
    #[serde(rename(deserialize = "Conjugue"), deserialize_with = "de_null_str")]
    pub spouse: String,
    #[serde(rename(deserialize = "Acompanhante"), deserialize_with = "de_null_str")]
    pub companion: String,
    #[serde(rename(deserialize = "Contato"), deserialize_with = "de_null_str")]
    pub emergency_contact: String,
    #[serde(rename(deserialize = "Filhos"), deserialize_with = "de_null_i64")]
    pub children_count: i64,
    #[serde(rename(deserialize = "Id_do_Convenio"), deserialize_with = "de_null_i64")]
    pub insurance_id: i64,
    #[serde(rename(deserialize = "Numero_da_Matricula"), deserialize_with = "de_null_str")]
    pub insurance_number: String,
    #[serde(rename(deserialize = "Mala_Direta"), deserialize_with = "de_null_bool")]
    pub mailing_list: bool,
    #[serde(rename(deserialize = "VIP"), deserialize_with = "de_null_bool")]
    pub vip: bool,
    #[serde(rename(deserialize = "Exclui_Mkt"), deserialize_with = "de_null_i64")]
    pub exclude_marketing: i64,
    #[serde(rename(deserialize = "Tags"), deserialize_with = "de_null_str")]
    pub tags: String,
    #[serde(rename(deserialize = "Como_conheceu"))]
    pub how_found: Option<String>,
    #[serde(rename(deserialize = "Indicado_por"), deserialize_with = "de_null_str")]
    pub referred_by: String,
    #[serde(rename(deserialize = "Escolaridade"), deserialize_with = "de_null_str")]
    pub education: String,
    #[serde(rename(deserialize = "Religiao"), deserialize_with = "de_null_str")]
    pub religion: String,
    #[serde(rename(deserialize = "Regiao"), deserialize_with = "de_null_str")]
    pub region: String,
    #[serde(rename(deserialize = "Co_Morbidade"), deserialize_with = "de_null_str")]
    pub comorbidities: String,
    #[serde(rename(deserialize = "Fadiga"), deserialize_with = "de_null_str")]
    pub fatigue: String,
    #[serde(rename(deserialize = "Fumante"), deserialize_with = "de_null_str")]
    pub smoker: String,
    #[serde(rename(deserialize = "Historico_Familiar_IAM_AVC_antes_50_anos"), deserialize_with = "de_null_str")]
    pub family_history_cardio: String,
    #[serde(rename(deserialize = "Referencias"), deserialize_with = "de_null_str")]
    pub references: String,
    #[serde(rename(deserialize = "PaginadaWeb"), deserialize_with = "de_null_str")]
    pub web_page: String,
    #[serde(rename(deserialize = "Opcional1"))]
    pub optional_1: Option<String>,
    #[serde(rename(deserialize = "Opcional2"))]
    pub optional_2: Option<String>,
    #[serde(rename(deserialize = "LastEditDate"))]
    pub last_edit_date: Option<String>,
    #[serde(rename(deserialize = "CreationDate"))]
    pub creation_date: Option<String>,
}

/// Convênio/plano de saúde disponível na clínica (`GET contatos/GetContatosConvenios`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsurancePlan {
    #[serde(rename(deserialize = "Id_do_Convenio"))]
    pub id: i64,
    #[serde(rename(deserialize = "Convenio"), deserialize_with = "de_null_str")]
    pub name: String,
}

/// Resultado resumido de busca de contatos (`GET contatos/GetContatosGridBySearch`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactSummary {
    #[serde(rename(deserialize = "Id_do_Cliente"))]
    pub id: i64,
    #[serde(rename(deserialize = "Nome"), deserialize_with = "de_null_str")]
    pub name: String,
    #[serde(rename(deserialize = "Nome_Social"))]
    pub social_name: Option<String>,
    #[serde(rename(deserialize = "Celular"), deserialize_with = "de_null_str")]
    pub mobile: String,
    #[serde(rename(deserialize = "Telefone_Residencial"), deserialize_with = "de_null_str")]
    pub phone_home: String,
    #[serde(rename(deserialize = "Email"), deserialize_with = "de_null_str")]
    pub email: String,
    #[serde(rename(deserialize = "CPF_CGC"), deserialize_with = "de_null_str")]
    pub cpf: String,
    #[serde(rename(deserialize = "IddoConvenio"), deserialize_with = "de_null_i64")]
    pub insurance_id: i64,
    #[serde(rename(deserialize = "Convenio"))]
    pub insurance_name: Option<String>,
    /// Total de registros correspondentes à busca (para paginação).
    #[serde(rename(deserialize = "total"), deserialize_with = "de_null_i64")]
    pub total: i64,
}

/// Contato homônimo retornado em verificação de duplicatas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomonymContact {
    #[serde(rename(deserialize = "Id_do_Cliente"))]
    pub id: i64,
    #[serde(rename(deserialize = "Nome"), deserialize_with = "de_null_str")]
    pub name: String,
    #[serde(rename(deserialize = "Nascimento"))]
    pub birth_date: Option<String>,
    #[serde(rename(deserialize = "Sexo"), deserialize_with = "de_null_str")]
    pub gender: String,
}

/// DTO para criar ou atualizar um contato.
///
/// Use [`ContactBuilder`] para construir este tipo com campos opcionais.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ContactDto {
    #[serde(rename = "Id_do_Cliente")]
    pub id: i64,
    #[serde(rename = "Nome")]
    pub name: String,
    #[serde(rename = "Nome_Social")]
    pub social_name: String,
    #[serde(rename = "Sexo")]
    pub gender: String,
    #[serde(rename = "Nascimento")]
    pub birth_date: Option<String>,
    #[serde(rename = "CPF_CGC")]
    pub cpf: String,
    #[serde(rename = "RG")]
    pub rg: String,
    #[serde(rename = "Email")]
    pub email: String,
    #[serde(rename = "Celular")]
    pub mobile: String,
    #[serde(rename = "Telefone_Residencial")]
    pub phone_home: String,
    #[serde(rename = "Telefone_Residencial_1")]
    pub phone_home_2: String,
    #[serde(rename = "Telefone_Comercial")]
    pub phone_work: String,
    #[serde(rename = "Endereco_Residencial")]
    pub address_home: String,
    #[serde(rename = "Bairro_Residencial")]
    pub neighborhood_home: String,
    #[serde(rename = "Cidade_Residencial")]
    pub city_home: String,
    #[serde(rename = "Estado_Residencial")]
    pub state_home: String,
    #[serde(rename = "Cep_Residencial")]
    pub zip_home: String,
    #[serde(rename = "Pais_Residencial")]
    pub country_home: String,
    #[serde(rename = "Endereco_Comercial")]
    pub address_work: String,
    #[serde(rename = "Bairro_Comercial")]
    pub neighborhood_work: String,
    #[serde(rename = "Cidade_Comercial")]
    pub city_work: String,
    #[serde(rename = "Estado_Comercial")]
    pub state_work: String,
    #[serde(rename = "Cep_Comercial")]
    pub zip_work: String,
    #[serde(rename = "Pais_Comercial")]
    pub country_work: String,
    #[serde(rename = "Profissao")]
    pub profession: String,
    #[serde(rename = "Empresa")]
    pub company: String,
    #[serde(rename = "Estado_Civil")]
    pub marital_status: String,
    #[serde(rename = "Tipo")]
    pub contact_type: String,
    #[serde(rename = "Observacoes")]
    pub notes: String,
    #[serde(rename = "Mae")]
    pub mother: String,
    #[serde(rename = "Pai")]
    pub father: String,
    #[serde(rename = "Conjugue")]
    pub spouse: String,
    #[serde(rename = "Acompanhante")]
    pub companion: String,
    #[serde(rename = "Contato")]
    pub emergency_contact: String,
    #[serde(rename = "Filhos")]
    pub children_count: i64,
    #[serde(rename = "Id_do_Convenio")]
    pub insurance_id: i64,
    #[serde(rename = "Numero_da_Matricula")]
    pub insurance_number: String,
    #[serde(rename = "Mala_Direta")]
    pub mailing_list: bool,
    #[serde(rename = "VIP")]
    pub vip: bool,
    #[serde(rename = "Exclui_Mkt")]
    pub exclude_marketing: i64,
    #[serde(rename = "Tags")]
    pub tags: String,
    #[serde(rename = "Como_conheceu")]
    pub how_found: Option<String>,
    #[serde(rename = "Indicado_por")]
    pub referred_by: String,
    #[serde(rename = "Escolaridade")]
    pub education: String,
    #[serde(rename = "Religiao")]
    pub religion: String,
    #[serde(rename = "Regiao")]
    pub region: String,
    #[serde(rename = "Co_Morbidade")]
    pub comorbidities: String,
    #[serde(rename = "Fadiga")]
    pub fatigue: String,
    #[serde(rename = "Fumante")]
    pub smoker: String,
    #[serde(rename = "Historico_Familiar_IAM_AVC_antes_50_anos")]
    pub family_history_cardio: String,
    #[serde(rename = "Referencias")]
    pub references: String,
    #[serde(rename = "PaginadaWeb")]
    pub web_page: String,
    #[serde(rename = "Opcional1")]
    pub optional_1: Option<String>,
    #[serde(rename = "Opcional2")]
    pub optional_2: Option<String>,
}

impl ContactDto {
    /// Cria um DTO mínimo com apenas nome obrigatório.
    /// Use os campos públicos para preencher os demais antes de enviar.
    pub fn new(name: impl Into<String>) -> Self {
        ContactDto {
            id: 0,
            name: name.into(),
            social_name: String::new(),
            gender: String::new(),
            birth_date: None,
            cpf: String::new(),
            rg: String::new(),
            email: String::new(),
            mobile: String::new(),
            phone_home: String::new(),
            phone_home_2: String::new(),
            phone_work: String::new(),
            address_home: String::new(),
            neighborhood_home: String::new(),
            city_home: String::new(),
            state_home: String::new(),
            zip_home: String::new(),
            country_home: String::new(),
            address_work: String::new(),
            neighborhood_work: String::new(),
            city_work: String::new(),
            state_work: String::new(),
            zip_work: String::new(),
            country_work: String::new(),
            profession: String::new(),
            company: String::new(),
            marital_status: String::new(),
            contact_type: String::new(),
            notes: String::new(),
            mother: String::new(),
            father: String::new(),
            spouse: String::new(),
            companion: String::new(),
            emergency_contact: String::new(),
            children_count: 0,
            insurance_id: 0,
            insurance_number: String::new(),
            mailing_list: false,
            vip: false,
            exclude_marketing: 0,
            tags: String::new(),
            how_found: None,
            referred_by: String::new(),
            education: String::new(),
            religion: String::new(),
            region: String::new(),
            comorbidities: String::new(),
            fatigue: String::new(),
            smoker: String::new(),
            family_history_cardio: String::new(),
            references: String::new(),
            web_page: String::new(),
            optional_1: None,
            optional_2: None,
        }
    }
}

impl From<Contact> for ContactDto {
    fn from(c: Contact) -> Self {
        ContactDto {
            id: c.id,
            name: c.name,
            social_name: c.social_name,
            gender: c.gender,
            birth_date: c.birth_date,
            cpf: c.cpf,
            rg: c.rg,
            email: c.email,
            mobile: c.mobile,
            phone_home: c.phone_home,
            phone_home_2: c.phone_home_2,
            phone_work: c.phone_work,
            address_home: c.address_home,
            neighborhood_home: c.neighborhood_home,
            city_home: c.city_home,
            state_home: c.state_home,
            zip_home: c.zip_home,
            country_home: c.country_home,
            address_work: c.address_work,
            neighborhood_work: c.neighborhood_work,
            city_work: c.city_work,
            state_work: c.state_work,
            zip_work: c.zip_work,
            country_work: c.country_work,
            profession: c.profession,
            company: c.company,
            marital_status: c.marital_status,
            contact_type: c.contact_type,
            notes: c.notes,
            mother: c.mother,
            father: c.father,
            spouse: c.spouse,
            companion: c.companion,
            emergency_contact: c.emergency_contact,
            children_count: c.children_count,
            insurance_id: c.insurance_id,
            insurance_number: c.insurance_number,
            mailing_list: c.mailing_list,
            vip: c.vip,
            exclude_marketing: c.exclude_marketing,
            tags: c.tags,
            how_found: c.how_found,
            referred_by: c.referred_by,
            education: c.education,
            religion: c.religion,
            region: c.region,
            comorbidities: c.comorbidities,
            fatigue: c.fatigue,
            smoker: c.smoker,
            family_history_cardio: c.family_history_cardio,
            references: c.references,
            web_page: c.web_page,
            optional_1: c.optional_1,
            optional_2: c.optional_2,
        }
    }
}

// ── Helpers internos ──────────────────────────────────────────────────────────

/// Gera um ID único para uso como `Id_do_Cliente` ao criar contatos.
///
/// A API MedX espera que o cliente aloque o ID antes da inserção (via `Settings/GetNewId`,
/// que pode não estar disponível em todos os planos). Este fallback usa hash de tempo
/// para gerar um ID positivo dentro do range i32.
fn generate_contact_id() -> i64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    // Mantém dentro de i32 positivo para compatibilidade com a API
    (h.finish() as i64).unsigned_abs() as i64 % 2_000_000_000
}

/// Grupo de filtro para busca de contatos.
///
/// Os valores numéricos correspondem ao parâmetro `Group` de
/// `contatos/GetContatosGridBySearch`. `Group=0` retorna `null` na API;
/// `Group=1` (All) é o valor correto para busca geral.
#[derive(Debug, Clone, Copy)]
pub enum ContactSearchGroup {
    /// Todos os contatos ativos (padrão).
    All = 1,
    /// Pacientes aniversariantes no mês.
    BirthdayThisMonth = 2,
}

// ── Métodos do MedxClient ─────────────────────────────────────────────────────

impl MedxClient {
    /// Retorna a ficha completa de um contato pelo ID.
    pub fn contact(&self, id: i64) -> Result<Contact, MedxError> {
        let text = self.get_text(&format!("contatos/GetContatosFichaById?Id={id}"))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Err(MedxError::UnexpectedResponse(format!("contato {id} não encontrado")));
        }
        let list: Vec<Contact> = serde_json::from_str(&text).map_err(MedxError::Json)?;
        list.into_iter()
            .next()
            .ok_or_else(|| MedxError::UnexpectedResponse(format!("contato {id} não encontrado")))
    }

    /// Retorna a foto do contato em Base64 (string vazia se sem foto).
    pub fn contact_photo_base64(&self, id: i64) -> Result<String, MedxError> {
        self.get_text(&format!("contatos/GetFotoBase64?Id={id}"))
    }

    /// Busca contatos por nome e grupo de filtro.
    ///
    /// `group_value` é uma categoria de filtro da API — **não** é um limite de
    /// resultados. Use `1` como valor padrão para busca geral; valores acima de
    /// ~20 podem retornar lista vazia.
    /// Retorna lista vazia se nenhum contato for encontrado (a API retorna `null`).
    pub fn search_contacts(
        &self,
        name: &str,
        group: ContactSearchGroup,
        group_value: u32,
    ) -> Result<Vec<ContactSummary>, MedxError> {
        let path = format!(
            "contatos/GetContatosGridBySearch?Group={}&GroupValue={}&Name={}",
            group as u32,
            group_value,
            encode_query_value(name),
        );
        let text = self.get_text(&path)?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Verifica se existem contatos homônimos (mesmo nome/sexo/nascimento).
    pub fn homonym_contacts(
        &self,
        name: &str,
        gender: &str,
        birth_date: &str,
    ) -> Result<Vec<HomonymContact>, MedxError> {
        self.get(&format!(
            "contatos/GetContatosHomonimos?Name={}&Gender={}&Birth={}",
            encode_query_value(name),
            encode_query_value(gender),
            encode_query_value(birth_date)
        ))
    }

    /// Retorna a lista de convênios/planos de saúde disponíveis.
    pub fn insurance_plans(&self) -> Result<Vec<InsurancePlan>, MedxError> {
        self.get("contatos/GetContatosConvenios")
    }

    /// Cria um novo contato. Retorna o ID do contato criado.
    ///
    /// A API MedX espera que o ID seja fornecido pelo chamador (campo `Id_do_Cliente`).
    /// Se `dto.id == 0`, um ID único é gerado automaticamente via hash de tempo.
    ///
    /// # Exemplo
    ///
    /// ```no_run
    /// # let client = medx::MedxClient::login("email", "senha").unwrap();
    /// let mut dto = medx::ContactDto::new("Maria da Silva");
    /// dto.mobile = "62999999999".to_string();
    /// dto.gender = "F".to_string();
    /// let id = client.create_contact(&dto).unwrap();
    /// ```
    pub fn create_contact(&self, dto: &ContactDto) -> Result<i64, MedxError> {
        let mut send_dto = dto.clone();
        if send_dto.id == 0 {
            send_dto.id = generate_contact_id();
        }
        let final_id = send_dto.id;
        let resp: serde_json::Value = self.post("contatos/InsertContato", &send_dto)?;
        // A API retorna a string "Success" em caso de sucesso
        if resp.as_str().map(|s| s.eq_ignore_ascii_case("success")).unwrap_or(false) {
            return Ok(final_id);
        }
        Err(MedxError::UnexpectedResponse(
            format!("InsertContato retornou resposta inesperada: {resp}")
        ))
    }

    /// Atualiza um contato existente.
    ///
    /// O campo `dto.id` deve ser o ID do contato a ser atualizado.
    pub fn update_contact(&self, dto: &ContactDto) -> Result<(), MedxError> {
        self.put::<_, serde_json::Value>("contatos/UpdateContato", dto)?;
        Ok(())
    }

    /// Remove um contato pelo ID.
    ///
    /// A API responde `200` com o corpo literal `"negado"` quando o paciente
    /// possui dados de prontuário e não pode ser excluído; nesse caso este
    /// método retorna `Err(MedxError::Api { status: 200, .. })` em vez de `Ok`.
    pub fn delete_contact(&self, id: i64) -> Result<(), MedxError> {
        let body = self.delete_text(&format!("contatos/DeleteContatoById?Id={id}"))?;
        if is_delete_denied(&body) {
            return Err(MedxError::Api {
                status: 200,
                message: "exclusão negada: o contato possui dados no prontuário".into(),
            });
        }
        Ok(())
    }

    /// Gera um novo ID para uso ao criar contatos (antes de chamar `create_contact`).
    pub fn new_contact_id(&self) -> Result<i64, MedxError> {
        let resp: serde_json::Value = self.get("Settings/GetNewId?qtd=1")?;
        resp.as_i64()
            .or_else(|| resp.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
            .or_else(|| resp.as_array().and_then(|a| a.first()).and_then(|v| v.as_i64()))
            .or_else(|| resp.as_array().and_then(|a| a.first()).and_then(|v| v.as_str()).and_then(|s| s.trim().parse::<i64>().ok()))
            .ok_or_else(|| MedxError::UnexpectedResponse(
                format!("GetNewId retornou formato inesperado: {resp}")
            ))
    }

    /// Retorna a URL de um arquivo no Azure Blob Storage.
    pub fn azure_file_url(&self, blob_name: &str) -> Result<String, MedxError> {
        let encoded = encode_query_value(blob_name);
        self.get_text(&format!("azure/getfileurl?blobname={encoded}"))
    }
}

/// Retorna `true` se o corpo de `DeleteContatoById` indica recusa da exclusão.
///
/// A API responde `200` com o corpo `"negado"` (às vezes envolto em aspas
/// JSON) quando o contato possui dados de prontuário.
fn is_delete_denied(body: &str) -> bool {
    body.trim().trim_matches('"').eq_ignore_ascii_case("negado")
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn is_delete_denied_detecta_negado() {
        assert!(is_delete_denied("negado"));
        assert!(is_delete_denied("\"negado\""));
        assert!(is_delete_denied("  Negado  "));
        assert!(!is_delete_denied("true"));
        assert!(!is_delete_denied(""));
        assert!(!is_delete_denied("\"12345\""));
    }

    pub const CONTACT_JSON: &str = r#"{
        "Id_do_Cliente": 12345,
        "Nome": "MARIA DA SILVA",
        "Nome_Social": "",
        "Sexo": "F",
        "Nascimento": "1990-05-15T00:00:00",
        "CPF_CGC": "12345678900",
        "RG": "1234567",
        "Email": "maria@email.com",
        "Celular": "62999998888",
        "Telefone_Residencial": "6233331111",
        "Telefone_Residencial_1": "",
        "Telefone_Comercial": "",
        "Endereco_Residencial": "RUA DAS FLORES, 123",
        "Bairro_Residencial": "CENTRO",
        "Cidade_Residencial": "GOIANIA",
        "Estado_Residencial": "GO",
        "Cep_Residencial": "74000000",
        "Pais_Residencial": "BRASIL",
        "Endereco_Comercial": "",
        "Bairro_Comercial": "",
        "Cidade_Comercial": "",
        "Estado_Comercial": "",
        "Cep_Comercial": "",
        "Pais_Comercial": "",
        "Profissao": "PROFESSORA",
        "Empresa": "",
        "Estado_Civil": "SOLTEIRO",
        "Tipo": "Paciente",
        "Observacoes": "",
        "Mae": "JOANA",
        "Pai": "",
        "Conjugue": "",
        "Acompanhante": "",
        "Contato": "",
        "Filhos": 0,
        "Id_do_Convenio": 1,
        "Numero_da_Matricula": "",
        "Mala_Direta": true,
        "VIP": false,
        "Exclui_Mkt": 0,
        "Tags": "",
        "Como_conheceu": null,
        "Indicado_por": "",
        "Escolaridade": "",
        "Religiao": "",
        "Regiao": "",
        "Co_Morbidade": "",
        "Fadiga": "",
        "Fumante": "",
        "Historico_Familiar_IAM_AVC_antes_50_anos": "",
        "Referencias": "",
        "PaginadaWeb": "",
        "Opcional1": null,
        "Opcional2": null,
        "LastEditDate": "2026-01-10T12:00:00",
        "CreationDate": "2020-03-01T09:30:00"
    }"#;

    pub const INSURANCE_JSON: &str = r#"[
        {"Id_do_Convenio": 1, "Convenio": "PARTICULAR"},
        {"Id_do_Convenio": 2, "Convenio": "UNIMED"},
        {"Id_do_Convenio": 3, "Convenio": "BRADESCO SAUDE"}
    ]"#;

    pub const CONTACT_SUMMARY_JSON: &str = r#"[
        {
            "Id_do_Cliente": 12345,
            "Nome": "MARIA DA SILVA",
            "Nome_Social": null,
            "Celular": "62999998888",
            "Telefone_Residencial": null,
            "Email": "maria@email.com",
            "CPF_CGC": "12345678900",
            "IddoConvenio": 1,
            "Convenio": "PARTICULAR",
            "total": 5
        },
        {
            "Id_do_Cliente": 67890,
            "Nome": "MARIA SOUZA",
            "Nome_Social": null,
            "Celular": "",
            "Telefone_Residencial": null,
            "Email": "",
            "CPF_CGC": null,
            "IddoConvenio": 0,
            "Convenio": null,
            "total": 5
        }
    ]"#;

    #[test]
    fn deserializa_contact_completo() {
        let c: Contact = serde_json::from_str(CONTACT_JSON).unwrap();
        assert_eq!(c.id, 12345);
        assert_eq!(c.name, "MARIA DA SILVA");
        assert_eq!(c.gender, "F");
        assert_eq!(c.cpf, "12345678900");
        assert_eq!(c.mobile, "62999998888");
        assert_eq!(c.city_home, "GOIANIA");
        assert_eq!(c.state_home, "GO");
        assert!(c.mailing_list);
        assert!(!c.vip);
        assert_eq!(c.children_count, 0);
        assert_eq!(c.insurance_id, 1);
        assert!(c.how_found.is_none());
    }

    #[test]
    fn contact_campos_null_viram_strings_vazias() {
        // Como_conheceu é null → Option::None; campos de texto null → ""
        let c: Contact = serde_json::from_str(CONTACT_JSON).unwrap();
        assert!(c.how_found.is_none());
        assert!(c.optional_1.is_none());
        assert_eq!(c.social_name, "");
        assert_eq!(c.city_work, "");
    }

    #[test]
    fn deserializa_insurance_plans() {
        let plans: Vec<InsurancePlan> = serde_json::from_str(INSURANCE_JSON).unwrap();
        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].id, 1);
        assert_eq!(plans[0].name, "PARTICULAR");
        assert_eq!(plans[2].name, "BRADESCO SAUDE");
    }

    #[test]
    fn deserializa_contact_summary_com_nulls() {
        let list: Vec<ContactSummary> = serde_json::from_str(CONTACT_SUMMARY_JSON).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, 12345);
        assert_eq!(list[0].name, "MARIA DA SILVA");
        assert_eq!(list[0].mobile, "62999998888");
        assert_eq!(list[0].total, 5);
        assert_eq!(list[1].id, 67890);
        // CPF null → string vazia
        assert_eq!(list[1].cpf, "");
        // Convenio null → None
        assert!(list[1].insurance_name.is_none());
    }

    #[test]
    fn contact_dto_new_tem_defaults_corretos() {
        let dto = ContactDto::new("JOAO SILVA");
        assert_eq!(dto.name, "JOAO SILVA");
        assert_eq!(dto.id, 0);
        assert_eq!(dto.gender, "");
        assert!(!dto.mailing_list);
        assert!(!dto.vip);
        assert_eq!(dto.insurance_id, 0);
        assert_eq!(dto.children_count, 0);
        assert!(dto.birth_date.is_none());
    }

    #[test]
    fn contact_into_dto_preserva_campos() {
        let c: Contact = serde_json::from_str(CONTACT_JSON).unwrap();
        let dto: ContactDto = c.into();
        assert_eq!(dto.id, 12345);
        assert_eq!(dto.name, "MARIA DA SILVA");
        assert_eq!(dto.mobile, "62999998888");
        assert!(dto.mailing_list);
        assert_eq!(dto.insurance_id, 1);
    }

    #[test]
    fn serializa_dto_tem_campos_api() {
        let dto = ContactDto::new("TESTE");
        let json = serde_json::to_value(&dto).unwrap();
        assert!(json.get("Nome").is_some());
        assert!(json.get("Id_do_Cliente").is_some());
        assert!(json.get("Celular").is_some());
        assert!(json.get("CPF_CGC").is_some());
    }
}
