//! Etapa 8 — Marketing: eventos, questionários, diagnóstico QP e local de atendimento.
//!
//! Endpoints mapeados:
//! - `GET  eventos/getAllEventos`                              — lista de eventos
//! - `GET  settings/XML_GetQuests`                            — lista de questionários
//! - `POST marketing/InsertQuests`                            — associa questionário a contato
//! - `GET  diagnosticoqp/GetAllDiagnosticoQP`                 — lista de diagnósticos QP
//! - `POST marketing/UpdateLocalAtendimentoNomeClinica`       — atualiza nome da clínica no atendimento

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

#[allow(dead_code)]
fn de_null_f64<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    use serde::de::Visitor;
    struct F64OrStr;
    impl<'de> Visitor<'de> for F64OrStr {
        type Value = f64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "f64, string ou null")
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

// ── Tipos ─────────────────────────────────────────────────────────────────────

/// Evento retornado por `GET eventos/getAllEventos`.
///
/// Representa um evento agendado ou recorrente associado a um contato, como
/// aniversários ou lembretes de retorno.
#[derive(Debug, Clone, Deserialize)]
pub struct Evento {
    /// ID do boleto associado ao evento (0 se não houver).
    #[serde(default, rename = "IddoBoleto", deserialize_with = "de_null_i64")]
    pub id: i64,

    /// ID do evento interno.
    #[serde(default, rename = "IddoEvento", deserialize_with = "de_null_i64")]
    pub event_id: i64,

    /// Conteúdo HTML da notificação do evento.
    #[serde(default, rename = "Notificacao", deserialize_with = "de_null_str")]
    pub name: String,

    /// Tipo/status do evento (ex.: `"Event"`).
    #[serde(default, rename = "Status", deserialize_with = "de_null_str")]
    pub type_name: String,

    /// URL associada ao evento.
    #[serde(default, rename = "URLdoEvento", deserialize_with = "de_null_str")]
    pub url: String,
}

/// Questionário retornado por `GET settings/XML_GetQuests`.
#[derive(Debug, Clone, Deserialize)]
pub struct Quest {
    /// ID interno do questionário.
    #[serde(rename = "Id", deserialize_with = "de_null_i64")]
    pub id: i64,

    /// Nome/arquivo do questionário.
    #[serde(default, rename = "Arquivo", deserialize_with = "de_null_str")]
    pub name: String,

    /// XML do questionário (pode ser vazio).
    #[serde(default, rename = "XML", deserialize_with = "de_null_str")]
    pub xml: String,
}

/// DTO para associar um questionário a um contato via
/// `POST marketing/InsertQuests`.
///
/// Use [`InsertQuestDto::new`] para construir o payload; o campo `answers`
/// fica vazio por padrão e pode ser preenchido manualmente se necessário.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertQuestDto {
    /// ID do contato/paciente no MedX.
    #[serde(rename = "IdContato")]
    pub contact_id: i64,

    /// ID do questionário a ser associado.
    #[serde(rename = "IdQuestionario")]
    pub quest_id: i64,

    /// Respostas pré-preenchidas (vazio por padrão).
    #[serde(rename = "Respostas")]
    pub answers: String,
}

impl InsertQuestDto {
    /// Cria um novo DTO para inserção de questionário.
    ///
    /// - `contact_id`: ID do contato/paciente no MedX
    /// - `quest_id`: ID do questionário a ser associado
    ///
    /// O campo `answers` é inicializado com string vazia.
    pub fn new(contact_id: i64, quest_id: i64) -> Self {
        InsertQuestDto {
            contact_id,
            quest_id,
            answers: String::new(),
        }
    }
}

/// Diagnóstico QP retornado por `GET diagnosticoqp/GetAllDiagnosticoQP`.
#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosticoQP {
    /// ID interno do diagnóstico.
    #[serde(default, rename = "IddoDiagnosticoQP", deserialize_with = "de_null_i64")]
    pub id: i64,

    /// Nome do diagnóstico (ex.: `"Consulta"`, `"Procedimento"`).
    #[serde(default, rename = "StrDiagnosticoQP", deserialize_with = "de_null_str")]
    pub name: String,

    /// Tempo associado ao diagnóstico (pode ser null).
    #[serde(default, rename = "Tempo", deserialize_with = "de_null_str")]
    pub tempo: String,
}

/// DTO para atualizar o nome da clínica no atendimento via
/// `POST marketing/UpdateLocalAtendimentoNomeClinica`.
///
/// Use [`UpdateLocalAtendimentoDto::new`] para construir o payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLocalAtendimentoDto {
    /// ID do atendimento (string, ex.: `"ATD-001"`).
    #[serde(rename = "IdAtendimento")]
    pub attendance_id: String,

    /// Nome da clínica a ser gravado no atendimento.
    #[serde(rename = "NomeClinica")]
    pub clinic_name: String,
}

impl UpdateLocalAtendimentoDto {
    /// Cria um novo DTO para atualização do local de atendimento.
    ///
    /// - `attendance_id`: identificador do atendimento no MedX
    /// - `clinic_name`: nome da clínica a ser associado ao atendimento
    pub fn new(attendance_id: &str, clinic_name: &str) -> Self {
        UpdateLocalAtendimentoDto {
            attendance_id: attendance_id.to_string(),
            clinic_name: clinic_name.to_string(),
        }
    }
}

// ── Métodos do cliente ────────────────────────────────────────────────────────

impl MedxClient {
    /// Retorna a lista de eventos cadastrados na plataforma.
    ///
    /// `GET eventos/getAllEventos`
    ///
    /// Trata respostas que não são array (objeto vazio `{}` ou string)
    /// retornando `Vec::new()` sem erro.
    pub fn events(&self) -> Result<Vec<Evento>, MedxError> {
        let raw: serde_json::Value = self.get("eventos/getAllEventos")?;
        parse_vec(raw)
    }

    /// Retorna a lista de questionários disponíveis.
    ///
    /// `GET settings/XML_GetQuests`
    ///
    /// Trata respostas que não são array retornando `Vec::new()` sem erro.
    pub fn quests(&self) -> Result<Vec<Quest>, MedxError> {
        let raw: serde_json::Value = self.get("settings/XML_GetQuests")?;
        parse_vec(raw)
    }

    /// Associa um questionário a um contato.
    ///
    /// `POST marketing/InsertQuests`
    ///
    /// A resposta da API é descartada; erros de rede ou HTTP são propagados.
    pub fn insert_quest(&self, dto: &InsertQuestDto) -> Result<(), MedxError> {
        let _: serde_json::Value = self.post("marketing/InsertQuests", dto)?;
        Ok(())
    }

    /// Retorna a lista de diagnósticos QP cadastrados.
    ///
    /// `GET diagnosticoqp/GetAllDiagnosticoQP`
    ///
    /// Trata respostas que não são array retornando `Vec::new()` sem erro.
    pub fn diagnostico_qp(&self) -> Result<Vec<DiagnosticoQP>, MedxError> {
        let raw: serde_json::Value = self.get("diagnosticoqp/GetAllDiagnosticoQP")?;
        parse_vec(raw)
    }

    /// Atualiza o nome da clínica associado a um atendimento.
    ///
    /// `POST marketing/UpdateLocalAtendimentoNomeClinica`
    ///
    /// A resposta da API é descartada; erros de rede ou HTTP são propagados.
    pub fn update_local_atendimento(&self, dto: &UpdateLocalAtendimentoDto) -> Result<(), MedxError> {
        let _: serde_json::Value = self.post("marketing/UpdateLocalAtendimentoNomeClinica", dto)?;
        Ok(())
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

    const EVENTOS_JSON: &str = r#"[
        {
            "IddoBoleto": 1,
            "IddoEvento": 10,
            "Notificacao": "Aniversário",
            "Status": "Aniversario",
            "URLdoEvento": "https://exemplo.com/ev/10"
        },
        {
            "IddoBoleto": 2,
            "IddoEvento": 0,
            "Notificacao": "Retorno",
            "Status": "Retorno",
            "URLdoEvento": ""
        }
    ]"#;

    const QUESTS_JSON: &str = r#"[
        {
            "Id": 10,
            "Arquivo": "Questionário de Satisfação",
            "XML": "<quest>...</quest>"
        },
        {
            "Id": 11,
            "Arquivo": "Anamnese Geral",
            "XML": null
        }
    ]"#;

    const DIAGNOSTICO_JSON: &str = r#"[
        {
            "IddoDiagnosticoQP": 5,
            "StrDiagnosticoQP": "Bruxismo",
            "Tempo": "30min"
        },
        {
            "IddoDiagnosticoQP": 6,
            "StrDiagnosticoQP": "Cárie dentária",
            "Tempo": null
        }
    ]"#;

    #[test]
    fn deserializa_vec_evento() {
        let eventos: Vec<Evento> = serde_json::from_str(EVENTOS_JSON).unwrap();
        assert_eq!(eventos.len(), 2);

        let e0 = &eventos[0];
        assert_eq!(e0.id, 1);
        assert_eq!(e0.event_id, 10);
        assert_eq!(e0.name, "Aniversário");
        assert_eq!(e0.type_name, "Aniversario");
        assert_eq!(e0.url, "https://exemplo.com/ev/10");

        let e1 = &eventos[1];
        assert_eq!(e1.id, 2);
        assert_eq!(e1.name, "Retorno");
        assert_eq!(e1.event_id, 0);
        assert_eq!(e1.url, "");
    }

    #[test]
    fn deserializa_vec_quest() {
        let quests: Vec<Quest> = serde_json::from_str(QUESTS_JSON).unwrap();
        assert_eq!(quests.len(), 2);

        let q0 = &quests[0];
        assert_eq!(q0.id, 10);
        assert_eq!(q0.name, "Questionário de Satisfação");
        assert_eq!(q0.xml, "<quest>...</quest>");

        // XML nulo deve virar string vazia
        let q1 = &quests[1];
        assert_eq!(q1.id, 11);
        assert_eq!(q1.xml, "");
    }

    #[test]
    fn deserializa_vec_diagnostico_qp() {
        let diags: Vec<DiagnosticoQP> = serde_json::from_str(DIAGNOSTICO_JSON).unwrap();
        assert_eq!(diags.len(), 2);

        let d0 = &diags[0];
        assert_eq!(d0.id, 5);
        assert_eq!(d0.name, "Bruxismo");
        assert_eq!(d0.tempo, "30min");

        // Tempo nulo deve virar string vazia
        let d1 = &diags[1];
        assert_eq!(d1.id, 6);
        assert_eq!(d1.name, "Cárie dentária");
        assert_eq!(d1.tempo, "");
    }

    #[test]
    fn serializa_insert_quest_dto_campos_api() {
        let dto = InsertQuestDto::new(123, 10);
        let v = serde_json::to_value(&dto).unwrap();

        assert_eq!(v["IdContato"], 123);
        assert_eq!(v["IdQuestionario"], 10);
        assert_eq!(v["Respostas"], "");
    }

    #[test]
    fn serializa_update_local_atendimento_dto_campos_api() {
        let dto = UpdateLocalAtendimentoDto::new("ATD-001", "Clínica XYZ");
        let v = serde_json::to_value(&dto).unwrap();

        assert_eq!(v["IdAtendimento"], "ATD-001");
        assert_eq!(v["NomeClinica"], "Clínica XYZ");
    }

    #[test]
    fn parse_vec_retorna_vazio_para_objeto_vazio() {
        // A API pode retornar {} quando não há resultados — não deve ser erro.
        let raw: serde_json::Value = serde_json::from_str("{}").unwrap();
        let result: Vec<Evento> = parse_vec(raw).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_vec_retorna_vazio_para_string() {
        // A API pode retornar "" em vez de [] — não deve ser erro.
        let raw: serde_json::Value = serde_json::from_str(r#""""#).unwrap();
        let result: Vec<Quest> = parse_vec(raw).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn insert_quest_dto_answers_pode_ser_sobrescrito() {
        let mut dto = InsertQuestDto::new(42, 7);
        dto.answers = "Sim;Não;Talvez".to_string();
        let v = serde_json::to_value(&dto).unwrap();
        assert_eq!(v["Respostas"], "Sim;Não;Talvez");
    }
}
