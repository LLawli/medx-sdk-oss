//! Etapa 9 — Dashboard (Hoje): notificações, últimos atendidos, trial, notas e utilitários.
//!
//! Endpoints mapeados:
//! - `GET  hoje/GetHojeNotificacoes`          — notificações do dia
//! - `GET  hoje/GetUltimosAtendidos`          — últimos pacientes atendidos
//! - `GET  adm/infosTrial`                    — informações da conta trial
//! - `GET  hoje/GetNotas`                     — notas do usuário
//! - `POST hoje/InsertNota`                   — cria nova nota
//! - `POST hoje/InsertNotaCliente`            — cria nota vinculada a paciente
//! - `PUT  hoje/UpdateNota`                   — atualiza nota existente
//! - `DELETE hoje/DeleteNotaById?Id={id}`     — remove nota pelo ID
//! - `GET  hoje/IsOTPOrExpired`               — verifica se sessão é OTP ou expirada
//! - `GET  SyncVersion/SyncData55To60`        — dados brutos de sincronização de versão

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
        fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<i64, E> { Ok(v as i64) }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<i64, E> {
            v.trim().parse::<i64>()
                .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<i64, E> { Ok(0) }
        fn visit_none<E: serde::de::Error>(self) -> Result<i64, E> { Ok(0) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<i64, D2::Error> {
            d.deserialize_any(I64OrStr)
        }
    }
    d.deserialize_any(I64OrStr)
}

fn de_lida<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    use serde::de::Visitor;
    struct BoolOrInt;
    impl<'de> Visitor<'de> for BoolOrInt {
        type Value = bool;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "bool, inteiro ou null")
        }
        fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<bool, E> { Ok(v) }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<bool, E> { Ok(v != 0) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<bool, E> { Ok(v != 0) }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<bool, E> { Ok(v != 0.0) }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<bool, E> {
            match v.trim().to_ascii_lowercase().as_str() {
                "true" | "1" => Ok(true),
                _ => Ok(false),
            }
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<bool, E> { Ok(false) }
        fn visit_none<E: serde::de::Error>(self) -> Result<bool, E> { Ok(false) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<bool, D2::Error> {
            d.deserialize_any(BoolOrInt)
        }
    }
    d.deserialize_any(BoolOrInt)
}

// ── Tipos ─────────────────────────────────────────────────────────────────────

/// Notificação retornada por `GET hoje/GetHojeNotificacoes`.
///
/// Representa alertas e lembretes exibidos no painel inicial do MedX,
/// como aniversários de pacientes ou vencimentos de planos.
#[derive(Debug, Clone, Deserialize)]
pub struct HojeNotificacao {
    /// ID do boleto associado (0 se não houver).
    #[serde(default, rename = "IddoBoleto", deserialize_with = "de_null_i64")]
    pub id: i64,

    /// Conteúdo HTML da notificação.
    #[serde(default, rename = "Notificacao", deserialize_with = "de_null_str")]
    pub message: String,

    /// Tipo da notificação (ex.: `"Event"`, `"Aniversario"`).
    #[serde(default, rename = "Status", deserialize_with = "de_null_str")]
    pub tipo: String,

    /// ID do evento associado (0 se não houver).
    #[serde(default, rename = "IddoEvento", deserialize_with = "de_null_i64")]
    pub event_id: i64,

    /// URL associada ao evento.
    #[serde(default, rename = "URLdoEvento", deserialize_with = "de_null_str")]
    pub event_url: String,
}

/// Último paciente atendido, retornado por `GET hoje/GetUltimosAtendidos`.
#[derive(Debug, Clone, Deserialize)]
pub struct UltimoAtendido {
    /// Nome completo do paciente.
    #[serde(default, rename = "Nome", deserialize_with = "de_null_str")]
    pub patient_name: String,

    /// ID interno do paciente.
    #[serde(default, rename = "Id_do_Cliente", deserialize_with = "de_null_i64")]
    pub patient_id: i64,

    /// Data/hora do último atendimento no formato ISO-8601.
    #[serde(default, rename = "Ultimo", deserialize_with = "de_null_str")]
    pub date: String,
}

/// Informações da conta trial retornadas por `GET adm/infosTrial`.
///
/// A resposta real vem envelopada em `{ "status": 200, "message": { ... } }`;
/// [`MedxClient::trial_info`] já extrai o objeto interno. Se `message` estiver
/// ausente/nulo, retorna [`TrialInfo::default`].
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TrialInfo {
    /// `true` se a conta está em período trial.
    #[serde(default, rename = "isTrial", deserialize_with = "de_lida")]
    pub is_trial: bool,

    /// Data de vigência/expiração da conta no formato ISO-8601.
    #[serde(default, rename = "vigencia", deserialize_with = "de_null_str")]
    pub vigencia: String,

    /// Telefone de contato do assinante. A API devolve `false` (não uma string)
    /// quando não há telefone; nesse caso vem como `"false"`.
    #[serde(default, rename = "celular", deserialize_with = "de_null_str")]
    pub celular: String,

    /// `true` se o assinante já passou pelo onboarding.
    #[serde(default, rename = "conheceu", deserialize_with = "de_lida")]
    pub conheceu: bool,
}

/// Extrai o `TrialInfo` do envelope `{ "message": { ... } }` de `adm/infosTrial`.
///
/// Se `message` estiver ausente ou for nulo, retorna [`TrialInfo::default`].
fn parse_trial_info(body: &serde_json::Value) -> Result<TrialInfo, MedxError> {
    match body.get("message") {
        Some(msg) if !msg.is_null() => {
            serde_json::from_value(msg.clone()).map_err(MedxError::Json)
        }
        _ => Ok(TrialInfo::default()),
    }
}

/// Nota do painel, retornada por `GET hoje/GetNotas`.
#[derive(Debug, Clone, Deserialize)]
pub struct Nota {
    /// ID interno da nota (pode ser negativo).
    #[serde(rename = "Id", deserialize_with = "de_null_i64")]
    pub id: i64,

    /// Texto da nota.
    #[serde(default, rename = "Memo", deserialize_with = "de_null_str")]
    pub text: String,

    /// Data/hora da nota no formato ISO-8601.
    #[serde(default, rename = "Data", deserialize_with = "de_null_str")]
    pub date: String,

    /// ID do usuário autor da nota.
    #[serde(default, rename = "IddoUsuario", deserialize_with = "de_null_i64")]
    pub user_id: i64,
}

/// DTO para criar uma nova nota via `POST hoje/InsertNota`.
///
/// O contrato real da API é `{ Id, Data, Memo, Concluida, IddoUsuario }`
/// (o mesmo shape do `PUT hoje/UpdateNota`). Use [`InsertNotaDto::new`]; `Data`
/// é preenchido com a data/hora local atual e `Id` fica `0` (nova nota).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertNotaDto {
    /// ID da nota; `0` ao inserir.
    #[serde(rename = "Id")]
    pub id: i64,

    /// Data/hora da nota no formato local `YYYY-MM-DDTHH:MM:SS`.
    #[serde(rename = "Data")]
    pub date: String,

    /// Texto da nota.
    #[serde(rename = "Memo")]
    pub text: String,

    /// Marca a nota como concluída.
    #[serde(rename = "Concluida")]
    pub done: bool,

    /// ID do usuário autor da nota.
    #[serde(rename = "IddoUsuario")]
    pub user_id: i64,
}

impl InsertNotaDto {
    /// Cria um novo DTO para inserção de nota.
    ///
    /// - `user_id`: ID do usuário autor
    /// - `text`: conteúdo textual da nota
    ///
    /// `Data` recebe a data/hora local atual; `Id` fica `0` e `Concluida` `false`.
    pub fn new(user_id: i64, text: &str) -> Self {
        InsertNotaDto {
            id: 0,
            date: crate::util::current_datetime_str(),
            text: text.to_string(),
            done: false,
            user_id,
        }
    }
}

/// DTO para atualizar uma nota existente via `PUT hoje/UpdateNota`.
///
/// Mesmo shape do insert (`{ Id, Data, Memo, Concluida, IddoUsuario }`), com
/// `Id` referenciando a nota. Use [`UpdateNotaDto::new`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotaDto {
    /// ID da nota a ser atualizada.
    #[serde(rename = "Id")]
    pub id: i64,

    /// Data/hora da atualização no formato local `YYYY-MM-DDTHH:MM:SS`.
    #[serde(rename = "Data")]
    pub date: String,

    /// Novo texto da nota.
    #[serde(rename = "Memo")]
    pub text: String,

    /// Marca a nota como concluída.
    #[serde(rename = "Concluida")]
    pub done: bool,

    /// ID do usuário autor da nota.
    #[serde(rename = "IddoUsuario")]
    pub user_id: i64,
}

impl UpdateNotaDto {
    /// Cria um novo DTO para atualização de nota.
    ///
    /// - `id`: ID da nota a ser atualizada
    /// - `user_id`: ID do usuário autor
    /// - `text`: novo conteúdo textual da nota
    ///
    /// `Data` recebe a data/hora local atual; `Concluida` fica `false`.
    pub fn new(id: i64, user_id: i64, text: &str) -> Self {
        UpdateNotaDto {
            id,
            date: crate::util::current_datetime_str(),
            text: text.to_string(),
            done: false,
            user_id,
        }
    }
}

/// DTO de **feedback** de cliente via `POST hoje/InsertNotaCliente`.
///
/// Apesar do nome do endpoint, este fluxo é de feedback: envia uma avaliação
/// numérica (`nota`) e uma observação em texto (`observacao`). Não é uma nota
/// vinculada a paciente. Use [`InsertNotaClienteDto::new`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertNotaClienteDto {
    /// Avaliação numérica (rating) do feedback.
    #[serde(rename = "nota")]
    pub rating: i64,

    /// Observação/comentário em texto (o legado limita a 255 caracteres).
    #[serde(rename = "observacao")]
    pub feedback: String,
}

impl InsertNotaClienteDto {
    /// Cria um novo DTO de feedback.
    ///
    /// - `rating`: avaliação numérica
    /// - `feedback`: observação em texto
    pub fn new(rating: i64, feedback: &str) -> Self {
        InsertNotaClienteDto {
            rating,
            feedback: feedback.to_string(),
        }
    }
}

// ── Métodos do cliente ────────────────────────────────────────────────────────

impl MedxClient {
    /// Retorna as notificações do painel do dia.
    ///
    /// `GET hoje/GetHojeNotificacoes`
    ///
    /// Trata respostas não-array retornando `Vec::new()` sem erro.
    pub fn hoje_notificacoes(&self) -> Result<Vec<HojeNotificacao>, MedxError> {
        let raw: serde_json::Value = self.get("hoje/GetHojeNotificacoes")?;
        parse_vec(raw)
    }

    /// Retorna a lista dos últimos pacientes atendidos.
    ///
    /// `GET hoje/GetUltimosAtendidos`
    ///
    /// Trata respostas não-array retornando `Vec::new()` sem erro.
    pub fn ultimos_atendidos(&self) -> Result<Vec<UltimoAtendido>, MedxError> {
        let raw: serde_json::Value = self.get("hoje/GetUltimosAtendidos")?;
        parse_vec(raw)
    }

    /// Retorna as informações da conta trial da clínica.
    ///
    /// `GET adm/infosTrial`
    ///
    /// A resposta vem envelopada em `{ "status", "message": { ... } }`; o
    /// objeto interno é extraído por [`parse_trial_info`].
    pub fn trial_info(&self) -> Result<TrialInfo, MedxError> {
        let body: serde_json::Value = self.get("adm/infosTrial")?;
        parse_trial_info(&body)
    }

    /// Retorna as notas do painel do usuário autenticado.
    ///
    /// `GET hoje/GetNotas`
    ///
    /// Trata respostas não-array retornando `Vec::new()` sem erro.
    pub fn notas(&self) -> Result<Vec<Nota>, MedxError> {
        let raw: serde_json::Value = self.get("hoje/GetNotas")?;
        parse_vec(raw)
    }

    /// Cria uma nova nota e retorna o ID gerado pela API.
    ///
    /// `POST hoje/InsertNota`
    ///
    /// A resposta pode ser um inteiro, uma string numérica ou `{"Id": N}`;
    /// todas as formas são tratadas. Retorna `0` se não for possível extrair o ID.
    pub fn insert_nota(&self, dto: &InsertNotaDto) -> Result<i64, MedxError> {
        let raw: serde_json::Value = self.post("hoje/InsertNota", dto)?;
        if let Some(n) = raw.as_i64() { return Ok(n); }
        if let Some(n) = raw.get("Id").and_then(|v| v.as_i64()) { return Ok(n); }
        if let Some(s) = raw.as_str() {
            if let Ok(n) = s.trim().parse::<i64>() { return Ok(n); }
        }
        Ok(0)
    }

    /// Cria uma nota vinculada a um paciente.
    ///
    /// `POST hoje/InsertNotaCliente`
    ///
    /// A resposta da API é descartada; erros de rede ou HTTP são propagados.
    pub fn insert_nota_cliente(&self, dto: &InsertNotaClienteDto) -> Result<(), MedxError> {
        let _raw: serde_json::Value = self.post("hoje/InsertNotaCliente", dto)?;
        Ok(())
    }

    /// Atualiza uma nota existente.
    ///
    /// `PUT hoje/UpdateNota`
    ///
    /// A resposta da API é descartada; erros de rede ou HTTP são propagados.
    pub fn update_nota(&self, dto: &UpdateNotaDto) -> Result<(), MedxError> {
        let _raw: serde_json::Value = self.put("hoje/UpdateNota", dto)?;
        Ok(())
    }

    /// Remove uma nota pelo seu ID.
    ///
    /// `DELETE hoje/DeleteNotaById?Id={id}`
    pub fn delete_nota(&self, id: i64) -> Result<(), MedxError> {
        self.delete(&format!("hoje/DeleteNotaById?Id={id}"))
    }

    /// Verifica se a sessão atual é OTP ou está expirada.
    ///
    /// `GET hoje/IsOTPOrExpired`
    ///
    /// A resposta pode ser um booleano, um inteiro ou um objeto com chave
    /// `IsOTP` ou `Expired`; todas as formas são tratadas.
    pub fn is_otp_or_expired(&self) -> Result<bool, MedxError> {
        let raw: serde_json::Value = self.get("hoje/IsOTPOrExpired")?;
        if let Some(b) = raw.as_bool() { return Ok(b); }
        if let Some(n) = raw.as_i64() { return Ok(n != 0); }
        if let Some(v) = raw.get("IsOTP").or_else(|| raw.get("Expired")) {
            if let Some(b) = v.as_bool() { return Ok(b); }
            if let Some(n) = v.as_i64() { return Ok(n != 0); }
        }
        Ok(false)
    }

    /// Retorna os dados brutos de sincronização de versão.
    ///
    /// `GET SyncVersion/SyncData55To60`
    ///
    /// A estrutura de resposta é desconhecida/variável; retorna o `Value` bruto
    /// para que o chamador possa inspecioná-lo conforme necessário.
    pub fn sync_version(&self) -> Result<serde_json::Value, MedxError> {
        self.get("SyncVersion/SyncData55To60")
    }
}

// ── Helpers internos ──────────────────────────────────────────────────────────

/// Converte um `serde_json::Value` para `Vec<T>` de forma resiliente.
///
/// - Se o valor for um array JSON, desserializa cada elemento como `T`.
/// - Se o valor for um objeto vazio `{}` ou qualquer outro tipo não-array,
///   retorna `Vec::new()` sem erro (comportamento defensivo para APIs que
///   retornam `{}` ou `""` quando não há resultados).
fn parse_vec<T>(value: serde_json::Value) -> Result<Vec<T>, MedxError>
where
    T: serde::de::DeserializeOwned,
{
    match value {
        serde_json::Value::Array(arr) => {
            let items = arr
                .into_iter()
                .map(|v| serde_json::from_value::<T>(v))
                .collect::<Result<Vec<T>, _>>()
                .map_err(MedxError::Json)?;
            Ok(items)
        }
        _ => Ok(Vec::new()),
    }
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const NOTIFICACOES_JSON: &str = r#"[
        {
            "IddoBoleto": 1,
            "Notificacao": "Aniversário de João Silva",
            "Status": "Aniversario",
            "IddoEvento": 10,
            "URLdoEvento": "https://exemplo.com/evento/10"
        },
        {
            "IddoBoleto": 2,
            "Notificacao": "Retorno de Maria Souza",
            "Status": "Retorno",
            "IddoEvento": 0,
            "URLdoEvento": ""
        }
    ]"#;

    const ULTIMOS_ATENDIDOS_JSON: &str = r#"[
        {
            "Nome": "João Silva",
            "Id_do_Cliente": 123,
            "Ultimo": "2026-03-17"
        },
        {
            "Nome": "Maria Souza",
            "Id_do_Cliente": 456,
            "Ultimo": "2026-03-16"
        }
    ]"#;

    // A API envelopa a resposta em { "status", "message": { ... } }.
    const TRIAL_INFO_JSON: &str = r#"{
        "status": 200,
        "message": {
            "isTrial": true,
            "conheceu": true,
            "celular": "11999998888",
            "vigencia": "2026-12-31T00:00:00"
        }
    }"#;

    const TRIAL_INFO_REGULAR_JSON: &str = r#"{
        "status": 200,
        "message": {
            "isTrial": false,
            "conheceu": true,
            "celular": false,
            "vigencia": "2099-01-01T00:00:00"
        }
    }"#;

    const NOTAS_JSON: &str = r#"[
        {
            "Id": 5,
            "Memo": "Paciente relatou dor de cabeça.",
            "Data": "2026-03-17T10:00:00",
            "IddoUsuario": 2
        },
        {
            "Id": 6,
            "Memo": "Retorno marcado para próxima semana.",
            "Data": "2026-03-17T11:30:00",
            "IddoUsuario": 2
        }
    ]"#;

    #[test]
    fn deserializa_vec_hoje_notificacao() {
        let notifs: Vec<HojeNotificacao> = serde_json::from_str(NOTIFICACOES_JSON).unwrap();
        assert_eq!(notifs.len(), 2);

        let n0 = &notifs[0];
        assert_eq!(n0.id, 1);
        assert_eq!(n0.message, "Aniversário de João Silva");
        assert_eq!(n0.tipo, "Aniversario");
        assert_eq!(n0.event_id, 10);
        assert_eq!(n0.event_url, "https://exemplo.com/evento/10");

        let n1 = &notifs[1];
        assert_eq!(n1.id, 2);
        assert_eq!(n1.message, "Retorno de Maria Souza");
        assert_eq!(n1.tipo, "Retorno");
        assert_eq!(n1.event_id, 0);
    }

    #[test]
    fn deserializa_vec_ultimo_atendido() {
        let atendidos: Vec<UltimoAtendido> = serde_json::from_str(ULTIMOS_ATENDIDOS_JSON).unwrap();
        assert_eq!(atendidos.len(), 2);

        let a0 = &atendidos[0];
        assert_eq!(a0.patient_name, "João Silva");
        assert_eq!(a0.patient_id, 123);
        assert_eq!(a0.date, "2026-03-17");

        let a1 = &atendidos[1];
        assert_eq!(a1.patient_name, "Maria Souza");
        assert_eq!(a1.patient_id, 456);
    }

    #[test]
    fn parse_trial_info_extrai_envelope_message() {
        let body: serde_json::Value = serde_json::from_str(TRIAL_INFO_JSON).unwrap();
        let info = parse_trial_info(&body).unwrap();
        assert!(info.is_trial, "isTrial=true deve mapear para is_trial");
        assert_eq!(info.vigencia, "2026-12-31T00:00:00");
        assert_eq!(info.celular, "11999998888");
        assert!(info.conheceu);

        let body_reg: serde_json::Value = serde_json::from_str(TRIAL_INFO_REGULAR_JSON).unwrap();
        let regular = parse_trial_info(&body_reg).unwrap();
        assert!(!regular.is_trial, "isTrial=false deve mapear para false");
        // celular ausente vem como bool false → string "false"
        assert_eq!(regular.celular, "false");
    }

    #[test]
    fn parse_trial_info_sem_message_retorna_default() {
        let body: serde_json::Value = serde_json::from_str(r#"{"status": 500}"#).unwrap();
        let info = parse_trial_info(&body).unwrap();
        assert!(!info.is_trial);
        assert_eq!(info.vigencia, "");

        let body_null: serde_json::Value = serde_json::from_str(r#"{"message": null}"#).unwrap();
        assert!(!parse_trial_info(&body_null).unwrap().is_trial);
    }

    #[test]
    fn deserializa_vec_nota() {
        let notas: Vec<Nota> = serde_json::from_str(NOTAS_JSON).unwrap();
        assert_eq!(notas.len(), 2);

        let n0 = &notas[0];
        assert_eq!(n0.id, 5);
        assert_eq!(n0.text, "Paciente relatou dor de cabeça.");
        assert_eq!(n0.date, "2026-03-17T10:00:00");
        assert_eq!(n0.user_id, 2);

        let n1 = &notas[1];
        assert_eq!(n1.id, 6);
        assert_eq!(n1.user_id, 2);
    }

    #[test]
    fn serializa_insert_nota_dto_campos_api() {
        let dto = InsertNotaDto::new(2, "Texto da nota de teste.");
        let v = serde_json::to_value(&dto).unwrap();

        // Contrato real: { Id, Data, Memo, Concluida, IddoUsuario }.
        assert_eq!(v["Id"], 0, "nova nota deve ter Id 0");
        assert_eq!(v["Memo"], "Texto da nota de teste.");
        assert_eq!(v["IddoUsuario"], 2);
        assert_eq!(v["Concluida"], false);
        // Os nomes antigos (inventados) não devem existir.
        assert!(v.get("IdUsuario").is_none(), "IdUsuario não deve existir");
        assert!(v.get("Nota").is_none(), "Nota não deve existir");
        assert!(v.get("DataNota").is_none(), "DataNota não deve existir");
        let data = v["Data"].as_str().unwrap();
        assert_eq!(data.len(), 19, "Data deve ter 19 chars: '{data}'");
        assert_eq!(&data[10..11], "T");
    }

    #[test]
    fn serializa_update_nota_dto_e_insert_nota_cliente_dto() {
        // UpdateNotaDto — mesmo shape do insert, com Id populado.
        let upd = UpdateNotaDto::new(5, 2, "Texto atualizado.");
        let v = serde_json::to_value(&upd).unwrap();
        assert_eq!(v["Id"], 5);
        assert_eq!(v["Memo"], "Texto atualizado.");
        assert_eq!(v["IddoUsuario"], 2);
        assert_eq!(v["Concluida"], false);
        assert!(v.get("Nota").is_none());
        assert!(v.get("DataNota").is_none());
        let data = v["Data"].as_str().unwrap();
        assert_eq!(data.len(), 19);
        assert_eq!(&data[10..11], "T");

        // InsertNotaClienteDto — feedback: { nota (rating), observacao }.
        let fb = InsertNotaClienteDto::new(5, "Ótimo atendimento.");
        let vc = serde_json::to_value(&fb).unwrap();
        assert_eq!(vc["nota"], 5);
        assert_eq!(vc["observacao"], "Ótimo atendimento.");
        assert!(vc.get("IdCliente").is_none(), "IdCliente não deve existir");
        assert!(vc.get("DataNota").is_none(), "DataNota não deve existir");
    }

    #[test]
    fn parse_vec_retorna_vazio_para_nao_array() {
        // Objeto vazio
        let raw: serde_json::Value = serde_json::from_str("{}").unwrap();
        let result: Vec<HojeNotificacao> = parse_vec(raw).unwrap();
        assert!(result.is_empty());

        // String vazia
        let raw2: serde_json::Value = serde_json::from_str(r#""""#).unwrap();
        let result2: Vec<Nota> = parse_vec(raw2).unwrap();
        assert!(result2.is_empty());
    }

    #[test]
    fn notificacao_campos_opcionais_ausentes() {
        // A API pode omitir campos opcionais
        let json = r#"[{"Notificacao": "Teste"}]"#;
        let notifs: Vec<HojeNotificacao> = serde_json::from_str(json).unwrap();
        assert_eq!(notifs[0].message, "Teste");
        assert_eq!(notifs[0].id, 0);
        assert_eq!(notifs[0].event_id, 0);
        assert_eq!(notifs[0].event_url, "");
    }
}
