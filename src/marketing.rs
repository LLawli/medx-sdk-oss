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

/// DTO para enviar questionário(s) a um contato via
/// `POST marketing/InsertQuests`.
///
/// O endpoint na verdade **envia os questionários por e-mail** ao paciente. O
/// contrato real é `{ Id_do_Cliente (string), FromAddresses, FromDisplayNames,
/// ToAddresses, ToDisplayNames, BodyQuest, Quests, Nascimento }`. `BodyQuest`
/// vem de `GET marketing/GetClienteSettings` → `Texto_Questionario`
/// ([`crate::ClienteSettings::quest_template`]). O campo `Quests` codifica os
/// questionários como `"Id;Arquivo|Id;Arquivo"` — use [`InsertQuestDto::add_quest`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertQuestDto {
    /// ID do contato/paciente no MedX (enviado como string).
    #[serde(rename = "Id_do_Cliente")]
    pub contact_id: String,

    /// Endereço de e-mail do remetente.
    #[serde(rename = "FromAddresses")]
    pub from_address: String,

    /// Nome de exibição do remetente.
    #[serde(rename = "FromDisplayNames")]
    pub from_name: String,

    /// Endereço de e-mail do destinatário.
    #[serde(rename = "ToAddresses")]
    pub to_address: String,

    /// Nome de exibição do destinatário.
    #[serde(rename = "ToDisplayNames")]
    pub to_name: String,

    /// Corpo do e-mail (template `Texto_Questionario`).
    #[serde(rename = "BodyQuest")]
    pub body_quest: String,

    /// Questionários codificados como `"Id;Arquivo|Id;Arquivo"`.
    #[serde(rename = "Quests")]
    pub quests: String,

    /// Data de nascimento do paciente (repassada do cadastro).
    #[serde(rename = "Nascimento")]
    pub nascimento: String,
}

impl InsertQuestDto {
    /// Cria um novo DTO de envio de questionário(s).
    ///
    /// - `contact_id`: ID do contato/paciente (serializado como string)
    /// - `from_address`/`from_name`: remetente
    /// - `to_address`/`to_name`: destinatário
    ///
    /// `BodyQuest`, `Quests` e `Nascimento` iniciam vazios. Adicione
    /// questionários com [`InsertQuestDto::add_quest`] e preencha
    /// `body_quest`/`nascimento` conforme necessário.
    pub fn new(
        contact_id: i64,
        from_address: &str,
        from_name: &str,
        to_address: &str,
        to_name: &str,
    ) -> Self {
        InsertQuestDto {
            contact_id: contact_id.to_string(),
            from_address: from_address.to_string(),
            from_name: from_name.to_string(),
            to_address: to_address.to_string(),
            to_name: to_name.to_string(),
            body_quest: String::new(),
            quests: String::new(),
            nascimento: String::new(),
        }
    }

    /// Adiciona um questionário ao campo `Quests` no formato `Id;Arquivo`,
    /// separando múltiplos itens por `|`.
    pub fn add_quest(&mut self, id: i64, arquivo: &str) {
        if !self.quests.is_empty() {
            self.quests.push('|');
        }
        self.quests.push_str(&format!("{id};{arquivo}"));
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
    fn insert_quest_dto_campos_api() {
        let dto = InsertQuestDto::new(42, "clinica@x.com", "Clínica X", "pac@y.com", "Paciente");
        let v = serde_json::to_value(&dto).unwrap();
        // Id_do_Cliente é serializado como string.
        assert_eq!(v["Id_do_Cliente"], "42");
        assert_eq!(v["FromAddresses"], "clinica@x.com");
        assert_eq!(v["ToAddresses"], "pac@y.com");
        assert_eq!(v["ToDisplayNames"], "Paciente");
        // Nomes antigos (inventados) não devem existir.
        assert!(v.get("IdContato").is_none());
        assert!(v.get("IdQuestionario").is_none());
        assert!(v.get("Respostas").is_none());
    }

    #[test]
    fn add_quest_concatena_com_pipe() {
        let mut dto = InsertQuestDto::new(1, "", "", "", "");
        dto.add_quest(123, "form_a.pdf");
        assert_eq!(dto.quests, "123;form_a.pdf");
        dto.add_quest(-999, "recordatorio.pdf");
        assert_eq!(dto.quests, "123;form_a.pdf|-999;recordatorio.pdf");
    }
}
