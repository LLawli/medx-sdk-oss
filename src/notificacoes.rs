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

    /// Template do e-mail de questionário (`BodyQuest` do `InsertQuests`).
    #[serde(default, rename = "Texto_Questionario", deserialize_with = "de_null_str")]
    pub quest_template: String,
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

/// DTO para enviar um e-mail via `POST notifications/InsertMailLogger`.
///
/// O contrato real da API é `{ FromAddresses, FromDisplayNames, ToAddresses,
/// ToDisplayNames, Subject, Body, BodyIsHTML, responseCaptcha }`. Os campos
/// `FromAddresses`/`ToAddresses` são endereços únicos (apesar do nome no
/// plural), não listas.
///
/// **O endpoint exige um token reCAPTCHA** (`responseCaptcha`): o frontend usa
/// grecaptcha invisível em todos os caminhos. Sem token, o servidor tende a
/// recusar o envio. Preencha [`MailLogDto::response_captcha`] com um token
/// válido antes de chamar [`MedxClient::log_email`]. Use [`MailLogDto::new`]
/// para o destinatário/assunto/corpo; o remetente vem das configurações da
/// clínica no servidor (deixado vazio aqui).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailLogDto {
    /// Endereço de e-mail do remetente (vazio = usa o configurado na clínica).
    #[serde(rename = "FromAddresses")]
    pub from_address: String,

    /// Nome de exibição do remetente (vazio = usa o configurado na clínica).
    #[serde(rename = "FromDisplayNames")]
    pub from_name: String,

    /// Endereço de e-mail do destinatário.
    #[serde(rename = "ToAddresses")]
    pub to_address: String,

    /// Nome de exibição do destinatário.
    #[serde(rename = "ToDisplayNames")]
    pub to_name: String,

    /// Assunto do e-mail.
    #[serde(rename = "Subject")]
    pub subject: String,

    /// Corpo do e-mail (pode conter HTML).
    #[serde(rename = "Body")]
    pub body: String,

    /// `1` se o corpo é HTML, `0` caso contrário.
    #[serde(rename = "BodyIsHTML")]
    pub body_is_html: i64,

    /// Token reCAPTCHA (`grecaptcha`). Obrigatório no servidor; omitido da
    /// serialização quando `None`.
    #[serde(rename = "responseCaptcha", default, skip_serializing_if = "Option::is_none")]
    pub response_captcha: Option<String>,
}

impl MailLogDto {
    /// Cria um novo envio de e-mail.
    ///
    /// - `to_address`: endereço de e-mail do destinatário
    /// - `subject`: assunto da mensagem
    /// - `body`: corpo da mensagem (HTML)
    ///
    /// O remetente fica vazio (a API usa as configurações da clínica) e
    /// `BodyIsHTML` é `1`. Defina [`MailLogDto::response_captcha`] com um token
    /// válido antes de enviar; sem ele o servidor recusa a requisição.
    pub fn new(to_address: &str, subject: &str, body: &str) -> Self {
        MailLogDto {
            from_address: String::new(),
            from_name: String::new(),
            to_address: to_address.to_string(),
            to_name: String::new(),
            subject: subject.to_string(),
            body: body.to_string(),
            body_is_html: 1,
            response_captcha: None,
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

    /// Envia um e-mail via `POST notifications/InsertMailLogger`.
    ///
    /// O endpoint exige um token reCAPTCHA em `dto.response_captcha`; sem ele o
    /// servidor tende a recusar a requisição. A resposta é descartada; erros de
    /// rede ou HTTP são propagados.
    pub fn log_email(&self, dto: &MailLogDto) -> Result<(), MedxError> {
        let _raw: serde_json::Value = self.post("notifications/InsertMailLogger", dto)?;
        Ok(())
    }
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
        "Texto_PreCadastro": "Caro(a) Cliente, clique no link: {{LINK}}",
        "Texto_Questionario": "Responda ao questionário: {{LINK}}"
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
        assert_eq!(s.quest_template, "Responda ao questionário: {{LINK}}");
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
            "paciente@email.com",
            "Lembrete de consulta",
            "<p>Sua consulta está confirmada.</p>",
        );
        let v = serde_json::to_value(&dto).unwrap();

        // Contrato real do InsertMailLogger.
        assert_eq!(v["ToAddresses"], "paciente@email.com");
        assert_eq!(v["Subject"], "Lembrete de consulta");
        assert_eq!(v["Body"], "<p>Sua consulta está confirmada.</p>");
        assert_eq!(v["FromAddresses"], "");
        assert_eq!(v["FromDisplayNames"], "");
        assert_eq!(v["ToDisplayNames"], "");
        assert_eq!(v["BodyIsHTML"], 1);
        // Sem token, responseCaptcha é omitido da serialização.
        assert!(v.get("responseCaptcha").is_none(), "captcha ausente não deve serializar");
        // Nomes antigos (inventados) não devem existir.
        assert!(v.get("IdContato").is_none());
        assert!(v.get("Para").is_none());
        assert!(v.get("DataEnvio").is_none());
    }

    #[test]
    fn serializa_mail_log_dto_com_captcha() {
        let mut dto = MailLogDto::new("x@y.com", "Assunto", "Corpo");
        dto.response_captcha = Some("token-abc".to_string());
        let v = serde_json::to_value(&dto).unwrap();
        assert_eq!(v["responseCaptcha"], "token-abc");
    }
}
