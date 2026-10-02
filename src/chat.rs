//! Etapa 7 — Chat interno: mensagens entre profissionais da clínica.
//!
//! Endpoints mapeados a partir de `newChat.js` (v65.medx.med.br):
//! - `GET  chat/GetUsers`                       — lista de usuários com contagem de não-lidas
//! - `GET  chat/LoadMessageHistory?IddoRemetente=<id>` — histórico com um usuário
//! - `GET  chat/GetChatCount`                   — total de mensagens não-lidas
//! - `GET  chat/GetIncomingMessage`             — mensagens recebidas ainda não-lidas
//! - `POST chat/SendChatMessage`                — envia nova mensagem
//! - `PUT  chat/UpdateLida`                     — marca mensagens como lidas

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

fn de_bool_or_int<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    use serde::de::Visitor;
    struct BoolOrInt;
    impl<'de> Visitor<'de> for BoolOrInt {
        type Value = bool;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "bool, 0, 1 ou null")
        }
        fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<bool, E> { Ok(v) }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<bool, E> { Ok(v != 0) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<bool, E> { Ok(v != 0) }
        fn visit_unit<E: serde::de::Error>(self) -> Result<bool, E> { Ok(false) }
        fn visit_none<E: serde::de::Error>(self) -> Result<bool, E> { Ok(false) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<bool, D2::Error> {
            d.deserialize_any(BoolOrInt)
        }
    }
    d.deserialize_any(BoolOrInt)
}

// ── Tipos ─────────────────────────────────────────────────────────────────────

/// Usuário do chat interno retornado por `GetUsers`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUser {
    #[serde(rename(deserialize = "UserId"), deserialize_with = "de_null_i64")]
    pub id: i64,

    #[serde(rename(deserialize = "UserFullName"), deserialize_with = "de_null_str")]
    pub full_name: String,

    #[serde(rename(deserialize = "UserName"), deserialize_with = "de_null_str")]
    pub username: String,

    #[serde(rename(deserialize = "IsOnLine"), default, deserialize_with = "de_bool_or_int")]
    pub online: bool,

    /// Quantidade de mensagens não-lidas desta conversa.
    #[serde(rename(deserialize = "Total"), deserialize_with = "de_null_i64", default)]
    pub unread: i64,
}

/// Mensagem de chat retornada pelo histórico ou por `GetIncomingMessage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    #[serde(rename(deserialize = "MessageId"), deserialize_with = "de_null_i64")]
    pub id: i64,

    /// ID do remetente.
    #[serde(rename(deserialize = "De"), deserialize_with = "de_null_i64")]
    pub from_id: i64,

    /// ID do destinatário.
    #[serde(rename(deserialize = "Para"), deserialize_with = "de_null_i64")]
    pub to_id: i64,

    #[serde(rename(deserialize = "MessageText"), deserialize_with = "de_null_str")]
    pub text: String,

    #[serde(rename(deserialize = "MessageDate"), deserialize_with = "de_null_str")]
    pub date: String,

    /// `0` = não-lida, `1` = lida.
    #[serde(rename(deserialize = "Lida"), deserialize_with = "de_null_i64", default)]
    pub read: i64,

    /// `0` = não-exibida, `1` = exibida.
    #[serde(rename(deserialize = "Exibida"), deserialize_with = "de_null_i64", default)]
    pub shown: i64,

    /// Nome de exibição do remetente.
    #[serde(rename(deserialize = "strDe"), deserialize_with = "de_null_str", default)]
    pub from_name: String,
}

impl ChatMessage {
    pub fn is_read(&self) -> bool { self.read != 0 }
    pub fn is_shown(&self) -> bool { self.shown != 0 }
}

/// Payload para enviar uma nova mensagem via `SendChatMessage`.
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageDto {
    #[serde(rename = "MessageId")]
    pub message_id: i64,

    /// Data/hora em formato ISO-8601, ex: `"2026-03-17T14:30:00"`.
    #[serde(rename = "MessageDate")]
    pub date: String,

    /// ID do remetente (usuário atual).
    #[serde(rename = "De")]
    pub from_id: i64,

    /// ID do destinatário.
    #[serde(rename = "Para")]
    pub to_id: i64,

    #[serde(rename = "MessageText")]
    pub text: String,

    /// Deixar vazio `""` ao enviar.
    #[serde(rename = "Lida")]
    pub read: String,

    /// Nome de exibição do remetente.
    #[serde(rename = "strDe")]
    pub from_name: String,

    /// Nome de exibição do destinatário (pode ficar vazio).
    #[serde(rename = "strPara")]
    pub to_name: String,
}

impl SendMessageDto {
    pub fn new(
        from_id: i64,
        from_name: impl Into<String>,
        to_id: i64,
        to_name: impl Into<String>,
        text: impl Into<String>,
        date: impl Into<String>,
    ) -> Self {
        Self {
            message_id: 0,
            date: date.into(),
            from_id,
            to_id,
            text: text.into(),
            read: String::new(),
            from_name: from_name.into(),
            to_name: to_name.into(),
        }
    }
}

/// Resposta de `GetChatCount`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCount {
    #[serde(rename(deserialize = "Total"), deserialize_with = "de_null_i64", default)]
    pub total: i64,
}

// ── Métodos do cliente ────────────────────────────────────────────────────────

impl MedxClient {
    /// Lista todos os usuários disponíveis no chat com contagem de não-lidas.
    ///
    /// `GET chat/GetUsers`
    pub fn chat_users(&self) -> Result<Vec<ChatUser>, MedxError> {
        self.get("chat/GetUsers")
    }

    /// Carrega o histórico de mensagens com um usuário específico.
    ///
    /// `GET chat/LoadMessageHistory?IddoRemetente=<sender_id>`
    pub fn chat_history(&self, sender_id: i64) -> Result<Vec<ChatMessage>, MedxError> {
        self.get(&format!("chat/LoadMessageHistory?IddoRemetente={sender_id}"))
    }

    /// Retorna o total de mensagens não-lidas.
    ///
    /// `GET chat/GetChatCount`
    pub fn chat_unread_count(&self) -> Result<i64, MedxError> {
        let raw: serde_json::Value = self.get("chat/GetChatCount")?;
        // A API pode retornar um objeto `{"Total": N}` ou diretamente um número.
        if let Some(n) = raw.get("Total").and_then(|v| v.as_i64()) {
            return Ok(n);
        }
        if let Some(n) = raw.as_i64() {
            return Ok(n);
        }
        // Array de um elemento
        if let Some(obj) = raw.as_array().and_then(|a| a.first()) {
            if let Some(n) = obj.get("Total").and_then(|v| v.as_i64()) {
                return Ok(n);
            }
        }
        Ok(0)
    }

    /// Retorna mensagens recebidas ainda não-lidas.
    ///
    /// `GET chat/GetIncomingMessage`
    pub fn chat_incoming(&self) -> Result<Vec<ChatMessage>, MedxError> {
        let text = self.get_text("chat/GetIncomingMessage")?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Envia uma nova mensagem de chat.
    ///
    /// `POST chat/SendChatMessage`
    pub fn send_chat_message(&self, msg: &SendMessageDto) -> Result<(), MedxError> {
        let _raw: serde_json::Value = self.post("chat/SendChatMessage", msg)?;
        Ok(())
    }

    /// Marca mensagens como lidas pelo IDs.
    ///
    /// `PUT chat/UpdateLida`
    pub fn mark_messages_read(&self, ids: &[i64]) -> Result<(), MedxError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "Ids")]
            ids: &'a [i64],
        }
        let _raw: serde_json::Value = self.put("chat/UpdateLida", &Body { ids })?;
        Ok(())
    }
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const USERS_JSON: &str = r#"[
        {"UserId":1,"UserFullName":"Dr. João Silva","UserName":"joao","IsOnLine":true,"Total":3},
        {"UserId":2,"UserFullName":"Dra. Maria Santos","UserName":"maria","IsOnLine":false,"Total":0}
    ]"#;

    const MESSAGES_JSON: &str = r#"[
        {"MessageId":101,"De":2,"Para":1,"MessageText":"Olá, pode atender?","MessageDate":"2026-03-17T10:00:00","Lida":0,"Exibida":0,"strDe":"Dra. Maria Santos"},
        {"MessageId":102,"De":1,"Para":2,"MessageText":"Pode sim!","MessageDate":"2026-03-17T10:01:00","Lida":1,"Exibida":1,"strDe":"Dr. João Silva"}
    ]"#;

    #[test]
    fn deserializa_chat_users() {
        let users: Vec<ChatUser> = serde_json::from_str(USERS_JSON).unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, 1);
        assert_eq!(users[0].full_name, "Dr. João Silva");
        assert!(users[0].online);
        assert_eq!(users[0].unread, 3);
        assert!(!users[1].online);
        assert_eq!(users[1].unread, 0);
    }

    #[test]
    fn deserializa_chat_messages() {
        let msgs: Vec<ChatMessage> = serde_json::from_str(MESSAGES_JSON).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].id, 101);
        assert_eq!(msgs[0].from_id, 2);
        assert_eq!(msgs[0].to_id, 1);
        assert_eq!(msgs[0].text, "Olá, pode atender?");
        assert!(!msgs[0].is_read());
        assert!(msgs[1].is_read());
        assert!(msgs[1].is_shown());
        assert_eq!(msgs[1].from_name, "Dr. João Silva");
    }

    #[test]
    fn send_message_dto_serializa_corretamente() {
        let dto = SendMessageDto::new(1, "Dr. João", 2, "Dra. Maria", "Boa tarde!", "2026-03-17T14:00:00");
        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["De"], 1);
        assert_eq!(json["Para"], 2);
        assert_eq!(json["MessageText"], "Boa tarde!");
        assert_eq!(json["strDe"], "Dr. João");
        assert_eq!(json["strPara"], "Dra. Maria");
        assert_eq!(json["Lida"], "");
        assert_eq!(json["MessageId"], 0);
    }

    #[test]
    fn chat_user_com_campos_nulos() {
        let json = r#"{"UserId":5,"UserFullName":null,"UserName":null,"IsOnLine":false}"#;
        let user: ChatUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, 5);
        assert_eq!(user.full_name, "");
        assert_eq!(user.username, "");
        assert_eq!(user.unread, 0);
    }

    #[test]
    fn chat_message_lida_como_bool() {
        // Testa resiliência caso a API envie boolean em vez de int
        let json = r#"[{"MessageId":1,"De":1,"Para":2,"MessageText":"Teste","MessageDate":"2026-03-17T09:00:00","Lida":false,"Exibida":true,"strDe":"User"}]"#;
        let msgs: Vec<ChatMessage> = serde_json::from_str(json).unwrap();
        assert!(!msgs[0].is_read());
        assert!(msgs[0].is_shown());
    }
}
