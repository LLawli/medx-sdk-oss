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
#[derive(Debug, Clone, Deserialize)]
pub struct TrialInfo {
    /// `1` se a conta é trial, `0` se é conta regular.
    #[serde(default, rename = "Trial", deserialize_with = "de_null_i64")]
    pub is_trial_account: i64,

    /// Data de expiração do trial no formato ISO-8601.
    #[serde(default, rename = "DataExpiracao", deserialize_with = "de_null_str")]
    pub trial_expires_at: String,

    /// Dias restantes até o vencimento do trial.
    #[serde(default, rename = "DiasRestantes", deserialize_with = "de_null_i64")]
    pub days_remaining: i64,

    /// Nome do plano contratado (ex.: `"Pro"`).
    #[serde(default, rename = "Plano", deserialize_with = "de_null_str")]
    pub plan: String,
}

impl TrialInfo {
    /// Retorna `true` se a conta está em período trial.
    pub fn is_trial(&self) -> bool {
        self.is_trial_account != 0
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
/// Use [`InsertNotaDto::new`] para construir o payload; `DataNota` é
/// preenchido automaticamente com a data/hora atual.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertNotaDto {
    /// ID do usuário que está criando a nota.
    #[serde(rename = "IdUsuario")]
    pub user_id: i64,

    /// Texto da nota.
    #[serde(rename = "Nota")]
    pub text: String,

    /// Data/hora da nota no formato `YYYY-MM-DDTHH:MM:SS` (UTC).
    #[serde(rename = "DataNota")]
    pub date: String,
}

impl InsertNotaDto {
    /// Cria um novo DTO para inserção de nota.
    ///
    /// - `user_id`: ID do usuário autor
    /// - `text`: conteúdo textual da nota
    ///
    /// `DataNota` é preenchido com a data/hora atual UTC.
    pub fn new(user_id: i64, text: &str) -> Self {
        InsertNotaDto {
            user_id,
            text: text.to_string(),
            date: current_datetime_str(),
        }
    }
}

/// DTO para atualizar uma nota existente via `PUT hoje/UpdateNota`.
///
/// Use [`UpdateNotaDto::new`] para construir o payload; `DataNota` é
/// preenchido automaticamente com a data/hora atual.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotaDto {
    /// ID da nota a ser atualizada.
    #[serde(rename = "Id")]
    pub id: i64,

    /// ID do usuário que está fazendo a atualização.
    #[serde(rename = "IdUsuario")]
    pub user_id: i64,

    /// Novo texto da nota.
    #[serde(rename = "Nota")]
    pub text: String,

    /// Data/hora da atualização no formato `YYYY-MM-DDTHH:MM:SS` (UTC).
    #[serde(rename = "DataNota")]
    pub date: String,
}

impl UpdateNotaDto {
    /// Cria um novo DTO para atualização de nota.
    ///
    /// - `id`: ID da nota a ser atualizada
    /// - `user_id`: ID do usuário autor
    /// - `text`: novo conteúdo textual da nota
    ///
    /// `DataNota` é preenchido com a data/hora atual UTC.
    pub fn new(id: i64, user_id: i64, text: &str) -> Self {
        UpdateNotaDto {
            id,
            user_id,
            text: text.to_string(),
            date: current_datetime_str(),
        }
    }
}

/// DTO para criar uma nota vinculada a um paciente via
/// `POST hoje/InsertNotaCliente`.
///
/// Use [`InsertNotaClienteDto::new`] para construir o payload; `DataNota` é
/// preenchido automaticamente com a data/hora atual.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertNotaClienteDto {
    /// ID do paciente ao qual a nota será vinculada.
    #[serde(rename = "IdCliente")]
    pub patient_id: i64,

    /// ID do usuário que está criando a nota.
    #[serde(rename = "IdUsuario")]
    pub user_id: i64,

    /// Texto da nota.
    #[serde(rename = "Nota")]
    pub text: String,

    /// Data/hora da nota no formato `YYYY-MM-DDTHH:MM:SS` (UTC).
    #[serde(rename = "DataNota")]
    pub date: String,
}

impl InsertNotaClienteDto {
    /// Cria um novo DTO para inserção de nota vinculada a paciente.
    ///
    /// - `patient_id`: ID do paciente no MedX
    /// - `user_id`: ID do usuário autor
    /// - `text`: conteúdo textual da nota
    ///
    /// `DataNota` é preenchido com a data/hora atual UTC.
    pub fn new(patient_id: i64, user_id: i64, text: &str) -> Self {
        InsertNotaClienteDto {
            patient_id,
            user_id,
            text: text.to_string(),
            date: current_datetime_str(),
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
    pub fn trial_info(&self) -> Result<TrialInfo, MedxError> {
        self.get("adm/infosTrial")
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

/// Retorna a data/hora atual formatada como `"YYYY-MM-DDTHH:MM:SS"`.
///
/// Usa apenas `std::time::SystemTime` para evitar dependências externas.
/// A precisão é de segundos; o fuso horário é UTC.
fn current_datetime_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Decompõe segundos Unix em componentes de data/hora UTC (sem chrono).
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;

    // Dias desde a epoch (1970-01-01)
    let days = (secs / 86400) as i64;

    // Algoritmo de conversão de dias para data (Gregorian proleptic)
    // Baseado no algoritmo público de domínio de Howard Hinnant.
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        year, month, day, h, m, s
    )
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

    const TRIAL_INFO_JSON: &str = r#"{
        "Trial": 1,
        "DataExpiracao": "2026-12-31",
        "DiasRestantes": 289,
        "Plano": "Pro"
    }"#;

    const TRIAL_INFO_REGULAR_JSON: &str = r#"{
        "Trial": 0,
        "DataExpiracao": "2099-01-01",
        "DiasRestantes": 0,
        "Plano": "Standard"
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
    fn deserializa_trial_info_e_verifica_is_trial() {
        let info: TrialInfo = serde_json::from_str(TRIAL_INFO_JSON).unwrap();
        assert_eq!(info.is_trial_account, 1);
        assert_eq!(info.trial_expires_at, "2026-12-31");
        assert_eq!(info.days_remaining, 289);
        assert_eq!(info.plan, "Pro");
        assert!(info.is_trial(), "is_trial_account=1 deve retornar true");

        let regular: TrialInfo = serde_json::from_str(TRIAL_INFO_REGULAR_JSON).unwrap();
        assert_eq!(regular.is_trial_account, 0);
        assert!(!regular.is_trial(), "is_trial_account=0 deve retornar false");
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

        assert_eq!(v["IdUsuario"], 2);
        assert_eq!(v["Nota"], "Texto da nota de teste.");
        // DataNota deve estar presente e ter formato correto
        let data_nota = v["DataNota"].as_str().unwrap();
        assert_eq!(data_nota.len(), 19, "DataNota deve ter 19 chars: '{data_nota}'");
        assert_eq!(&data_nota[10..11], "T");
    }

    #[test]
    fn serializa_update_nota_dto_e_insert_nota_cliente_dto() {
        // UpdateNotaDto
        let upd = UpdateNotaDto::new(5, 2, "Texto atualizado.");
        let v = serde_json::to_value(&upd).unwrap();
        assert_eq!(v["Id"], 5);
        assert_eq!(v["IdUsuario"], 2);
        assert_eq!(v["Nota"], "Texto atualizado.");
        let data = v["DataNota"].as_str().unwrap();
        assert_eq!(data.len(), 19);
        assert_eq!(&data[10..11], "T");

        // InsertNotaClienteDto
        let cli = InsertNotaClienteDto::new(123, 2, "Nota do paciente.");
        let vc = serde_json::to_value(&cli).unwrap();
        assert_eq!(vc["IdCliente"], 123);
        assert_eq!(vc["IdUsuario"], 2);
        assert_eq!(vc["Nota"], "Nota do paciente.");
        let datac = vc["DataNota"].as_str().unwrap();
        assert_eq!(datac.len(), 19);
        assert_eq!(&datac[10..11], "T");
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
    fn current_datetime_str_formato_correto() {
        let dt = current_datetime_str();
        assert_eq!(dt.len(), 19, "data/hora deve ter 19 chars: '{dt}'");
        assert_eq!(&dt[10..11], "T", "posição 10 deve ser 'T'");
        assert_eq!(&dt[4..5], "-");
        assert_eq!(&dt[7..8], "-");
        assert_eq!(&dt[13..14], ":");
        assert_eq!(&dt[16..17], ":");
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
