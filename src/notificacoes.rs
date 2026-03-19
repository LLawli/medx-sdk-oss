//! Etapa 7 — Notificações: configurações do cliente e log de e-mails enviados.
//!
//! Endpoints mapeados:
//! - `GET  marketing/GetClienteSettings`        — configurações de notificação da clínica
//! - `POST notifications/InsertMailLogger`      — registra um e-mail enviado no log

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

/// Configurações de marketing da clínica retornadas por `GetClienteSettings`.
///
/// Contém textos de templates (SMS, WhatsApp, e-mail), links sociais e
/// configurações de logotipo/imagem usados no módulo de marketing.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClienteSettings {
    /// Código interno da clínica (campo `Codigo_do_Cliente`).
    #[serde(default, rename = "Codigo_do_Cliente", deserialize_with = "de_null_i64")]
    pub id: i64,

    /// ID do software/instância.
    #[serde(default, rename = "SoftwareId", deserialize_with = "de_null_i64")]
    pub software_id: i64,

    /// Caminho do logotipo da clínica.
    #[serde(default, rename = "Logotipo", deserialize_with = "de_null_str")]
    pub logo: String,

    /// URL do Instagram da clínica (pode ser vazio).
    #[serde(default, rename = "Instagram", deserialize_with = "de_null_str")]
    pub instagram: String,

    /// URL do Facebook da clínica.
    #[serde(default, rename = "Facebook", deserialize_with = "de_null_str")]
    pub facebook: String,

    /// Site da clínica.
    #[serde(default, rename = "Website", deserialize_with = "de_null_str")]
    pub website: String,

    /// Template de texto SMS para lembretes.
    #[serde(default, rename = "Texto_SMS", deserialize_with = "de_null_str")]
    pub sms_template: String,

    /// Template de texto WhatsApp para lembretes.
    #[serde(default, rename = "Texto_Whatsapp", deserialize_with = "de_null_str")]
    pub whatsapp_template: String,

    /// Template do e-mail de pré-cadastro enviado ao paciente.
    #[serde(default, rename = "Texto_PreCadastro", deserialize_with = "de_null_str")]
    pub pre_registration_template: String,
}

impl ClienteSettings {
    /// Retorna `true` se a clínica tem um template SMS configurado.
    pub fn has_sms_template(&self) -> bool {
        !self.sms_template.is_empty()
    }

    /// Retorna `true` se a clínica tem um template WhatsApp configurado.
    pub fn has_whatsapp_template(&self) -> bool {
        !self.whatsapp_template.is_empty()
    }
}

/// DTO para registrar um e-mail enviado via `POST notifications/InsertMailLogger`.
///
/// Use [`MailLogDto::new`] para construir o payload com os campos obrigatórios;
/// os campos `De`, `NomeDe` e `DataEnvio` são preenchidos com valores padrão.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailLogDto {
    /// ID do contato/paciente destinatário.
    #[serde(rename = "IdContato")]
    pub contact_id: i64,

    /// Endereço de e-mail do destinatário.
    #[serde(rename = "Para")]
    pub recipient: String,

    /// Assunto do e-mail.
    #[serde(rename = "Assunto")]
    pub subject: String,

    /// Corpo do e-mail (pode conter HTML).
    #[serde(rename = "Corpo")]
    pub body: String,

    /// Endereço de e-mail do remetente (deixar vazio para usar o configurado na clínica).
    #[serde(rename = "De")]
    pub from: String,

    /// Nome de exibição do remetente (deixar vazio para usar o configurado na clínica).
    #[serde(rename = "NomeDe")]
    pub from_name: String,

    /// Data e hora do envio no formato `%Y-%m-%dT%H:%M:%S`.
    #[serde(rename = "DataEnvio")]
    pub sent_at: String,
}

impl MailLogDto {
    /// Cria um novo registro de log de e-mail.
    ///
    /// - `contact_id`: ID do contato/paciente no MedX
    /// - `recipient`: endereço de e-mail do destinatário
    /// - `subject`: assunto da mensagem
    /// - `body`: corpo da mensagem (texto ou HTML)
    ///
    /// Os campos `De` e `NomeDe` ficam vazios (a API usa as configurações da clínica).
    /// `DataEnvio` é preenchido com a data/hora atual no formato ISO-8601.
    pub fn new(contact_id: i64, recipient: &str, subject: &str, body: &str) -> Self {
        MailLogDto {
            contact_id,
            recipient: recipient.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            from: String::new(),
            from_name: String::new(),
            sent_at: current_datetime_str(),
        }
    }
}

// ── Métodos do cliente ────────────────────────────────────────────────────────

impl MedxClient {
    /// Retorna as configurações de marketing da clínica autenticada.
    ///
    /// `GET marketing/GetClienteSettings`
    pub fn client_settings(&self) -> Result<ClienteSettings, MedxError> {
        // A API retorna um array com um único elemento.
        let text = self.get_text("marketing/GetClienteSettings")?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(ClienteSettings::default());
        }
        // Tenta deserializar como array primeiro, depois como objeto.
        if let Ok(list) = serde_json::from_str::<Vec<ClienteSettings>>(&text) {
            return Ok(list.into_iter().next().unwrap_or_default());
        }
        serde_json::from_str::<ClienteSettings>(&text).map_err(MedxError::Json)
    }

    /// Registra um e-mail enviado no log de notificações.
    ///
    /// `POST notifications/InsertMailLogger`
    pub fn log_email(&self, dto: &MailLogDto) -> Result<(), MedxError> {
        let _raw: serde_json::Value = self.post("notifications/InsertMailLogger", dto)?;
        Ok(())
    }
}

// ── Helpers internos ──────────────────────────────────────────────────────────

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

    const SETTINGS_JSON: &str = r#"{
        "Codigo_do_Cliente": 7167,
        "SoftwareId": 4242,
        "Logotipo": "/medxdata/4242/logo.jpg",
        "Instagram": "@clinicaxyz",
        "Facebook": null,
        "Website": null,
        "Texto_SMS": "{{CLINICA}}: sua consulta está confirmada para {{DATA}}.",
        "Texto_Whatsapp": "Lembrete: consulta confirmada para {{DATA}}.",
        "Texto_PreCadastro": "Caro(a) Cliente, clique no link: {{LINK}}"
    }"#;

    #[test]
    fn deserializa_cliente_settings_completo() {
        let s: ClienteSettings = serde_json::from_str(SETTINGS_JSON).unwrap();
        assert_eq!(s.id, 7167);
        assert_eq!(s.software_id, 4242);
        assert_eq!(s.logo, "/medxdata/4242/logo.jpg");
        assert_eq!(s.instagram, "@clinicaxyz");
        assert_eq!(s.facebook, "");
        assert_eq!(s.website, "");
        assert!(!s.sms_template.is_empty());
        assert!(!s.whatsapp_template.is_empty());
        assert!(!s.pre_registration_template.is_empty());
    }

    #[test]
    fn deserializa_cliente_settings_com_campos_nulos_ausentes() {
        let json = r#"{
            "Codigo_do_Cliente": 7,
            "SoftwareId": null,
            "Logotipo": null,
            "Instagram": null,
            "Texto_SMS": null,
            "Texto_Whatsapp": null
        }"#;
        let s: ClienteSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.id, 7);
        assert_eq!(s.software_id, 0);
        assert_eq!(s.logo, "");
        assert_eq!(s.instagram, "");
        assert_eq!(s.sms_template, "");
        assert_eq!(s.whatsapp_template, "");
    }

    #[test]
    fn helpers_has_templates() {
        let s: ClienteSettings = serde_json::from_str(SETTINGS_JSON).unwrap();
        assert!(s.has_sms_template(), "deve ter template SMS");
        assert!(s.has_whatsapp_template(), "deve ter template WhatsApp");

        let empty = ClienteSettings::default();
        assert!(!empty.has_sms_template());
        assert!(!empty.has_whatsapp_template());
    }

    #[test]
    fn deserializa_como_array_e_retorna_primeiro() {
        let json = format!("[{}]", SETTINGS_JSON);
        let list: Vec<ClienteSettings> = serde_json::from_str(&json).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, 7167);
    }

    #[test]
    fn serializa_mail_log_dto_campos_api() {
        let dto = MailLogDto::new(
            123,
            "paciente@email.com",
            "Lembrete de consulta",
            "<p>Sua consulta está confirmada.</p>",
        );
        let v = serde_json::to_value(&dto).unwrap();

        assert_eq!(v["IdContato"], 123);
        assert_eq!(v["Para"], "paciente@email.com");
        assert_eq!(v["Assunto"], "Lembrete de consulta");
        assert_eq!(v["Corpo"], "<p>Sua consulta está confirmada.</p>");
        assert_eq!(v["De"], "");
        assert_eq!(v["NomeDe"], "");
        // DataEnvio deve estar presente e não vazio
        assert!(v.get("DataEnvio").is_some());
        let data_envio = v["DataEnvio"].as_str().unwrap();
        assert!(!data_envio.is_empty(), "DataEnvio não deve ser vazio");
    }

    #[test]
    fn current_datetime_str_formato_correto() {
        let dt = current_datetime_str();
        // Deve ter exatamente 19 caracteres: "YYYY-MM-DDTHH:MM:SS"
        assert_eq!(dt.len(), 19, "data/hora deve ter 19 chars: '{dt}'");
        // Deve conter o separador T na posição correta
        assert_eq!(&dt[10..11], "T", "posição 10 deve ser 'T'");
        // Separadores de data
        assert_eq!(&dt[4..5], "-");
        assert_eq!(&dt[7..8], "-");
        // Separadores de hora
        assert_eq!(&dt[13..14], ":");
        assert_eq!(&dt[16..17], ":");
    }
}
