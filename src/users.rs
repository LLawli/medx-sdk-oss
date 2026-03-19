//! Etapa 2 — Usuário atual, lista de usuários e troca de senha.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    client::MedxClient,
    error::MedxError,
};

// ── Deserializador auxiliar ───────────────────────────────────────────────────

/// Converte `null` JSON em `String::new()`, permitindo campos opcionalmente nulos.
fn de_null_str<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let opt: Option<String> = Option::deserialize(d)?;
    Ok(opt.unwrap_or_default())
}

// ── Tipos ─────────────────────────────────────────────────────────────────────

/// Usuário atualmente autenticado (`GET security/getcurrentuser`).
#[derive(Debug, Clone, Deserialize)]
pub struct CurrentUser {
    #[serde(rename = "DbId")]
    pub db_id: i64,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "UserFullName")]
    pub full_name: String,
    #[serde(rename = "UserId")]
    pub user_id: i64,
    #[serde(rename = "LastLogin")]
    pub last_login: String,
    #[serde(rename = "Plano")]
    pub plan: String,
    #[serde(rename = "Classificacao")]
    pub classification: String,
    #[serde(rename = "BloqueioSMS")]
    pub sms_blocked: bool,
    #[serde(rename = "BloqueioInsert")]
    pub insert_blocked: bool,
    #[serde(rename = "BloqueioAcesso")]
    pub access_blocked: bool,
    #[serde(rename = "UserEmail")]
    pub email: String,
    #[serde(rename = "RdStationKey")]
    pub rd_station_key: String,
    /// Retornado como string `"True"` / `"False"` pela API.
    #[serde(rename = "EstoqueAtualizado")]
    pub stock_updated_raw: String,
    #[serde(rename = "isTrial")]
    pub is_trial: bool,
}

impl CurrentUser {
    /// Converte o campo `stock_updated_raw` para `bool`.
    pub fn stock_updated(&self) -> bool {
        self.stock_updated_raw.eq_ignore_ascii_case("true")
    }
}

/// Usuário da clínica com permissões (`GET Users/GetUsers`).
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    #[serde(rename = "Id_do_Usuario")]
    pub id: i64,
    #[serde(rename = "Usuario")]
    pub username: String,
    #[serde(rename = "Nome_Completo")]
    pub full_name: String,
    #[serde(rename = "Ativo")]
    pub active: bool,
    #[serde(rename = "CPF", deserialize_with = "de_null_str")]
    pub cpf: String,
    #[serde(rename = "Cidade", deserialize_with = "de_null_str")]
    pub city: String,
    #[serde(rename = "Conselho_Profissional", deserialize_with = "de_null_str")]
    pub professional_council: String,
    #[serde(rename = "Numero_do_Conselho", deserialize_with = "de_null_str")]
    pub council_number: String,
    #[serde(rename = "UF_Conselho", deserialize_with = "de_null_str")]
    pub council_uf: String,
    #[serde(rename = "Validade")]
    pub validity: Option<String>,
    #[serde(rename = "Id_da_Unidade_de_Negocios")]
    pub business_unit_id: i64,
    // Permissões de acesso
    #[serde(rename = "Agenda")]
    pub perm_agenda: bool,
    #[serde(rename = "Pacientes")]
    pub perm_patients: bool,
    #[serde(rename = "Prontuario")]
    pub perm_medical_records: bool,
    #[serde(rename = "Gerencia")]
    pub perm_management: bool,
    #[serde(rename = "Estatistica")]
    pub perm_stats: bool,
    #[serde(rename = "Usuarios")]
    pub perm_users: bool,
    #[serde(rename = "Configuracao")]
    pub perm_config: bool,
    #[serde(rename = "Financeiro")]
    pub perm_financials: bool,
    #[serde(rename = "Marketing")]
    pub perm_marketing: bool,
    #[serde(rename = "Dashboard")]
    pub perm_dashboard: bool,
    #[serde(rename = "Relatorios_Financeiros")]
    pub perm_financial_reports: bool,
    #[serde(rename = "Atendimentos")]
    pub perm_appointments: bool,
    #[serde(rename = "Prontuario_escreve")]
    pub perm_medical_records_write: bool,
    // Horários de trabalho (7 dias: seg–dom)
    #[serde(rename = "hora1i", deserialize_with = "de_null_str")] pub schedule_mon_start: String,
    #[serde(rename = "hora1f", deserialize_with = "de_null_str")] pub schedule_mon_end: String,
    #[serde(rename = "hora2i", deserialize_with = "de_null_str")] pub schedule_tue_start: String,
    #[serde(rename = "hora2f", deserialize_with = "de_null_str")] pub schedule_tue_end: String,
    #[serde(rename = "hora3i", deserialize_with = "de_null_str")] pub schedule_wed_start: String,
    #[serde(rename = "hora3f", deserialize_with = "de_null_str")] pub schedule_wed_end: String,
    #[serde(rename = "hora4i", deserialize_with = "de_null_str")] pub schedule_thu_start: String,
    #[serde(rename = "hora4f", deserialize_with = "de_null_str")] pub schedule_thu_end: String,
    #[serde(rename = "hora5i", deserialize_with = "de_null_str")] pub schedule_fri_start: String,
    #[serde(rename = "hora5f", deserialize_with = "de_null_str")] pub schedule_fri_end: String,
    #[serde(rename = "hora6i", deserialize_with = "de_null_str")] pub schedule_sat_start: String,
    #[serde(rename = "hora6f", deserialize_with = "de_null_str")] pub schedule_sat_end: String,
    #[serde(rename = "hora7i", deserialize_with = "de_null_str")] pub schedule_sun_start: String,
    #[serde(rename = "hora7f", deserialize_with = "de_null_str")] pub schedule_sun_end: String,
}

/// DTO enviado para `POST usuarios/ChangeMePassword`.
#[derive(Serialize)]
struct ChangePasswordDto {
    pwd: String,
    #[serde(rename = "KeyId")]
    key_id: String,
    #[serde(rename = "Uid")]
    uid: i64,
    #[serde(rename = "Cripto")]
    cripto: bool,
}

// ── Métodos do MedxClient ─────────────────────────────────────────────────────

impl MedxClient {
    /// Retorna os dados do usuário atualmente autenticado.
    pub fn current_user(&self) -> Result<CurrentUser, MedxError> {
        self.get("security/getcurrentuser")
    }

    /// Retorna todos os usuários da clínica com suas permissões.
    pub fn users(&self) -> Result<Vec<User>, MedxError> {
        self.get("Users/GetUsers")
    }

    /// Altera a senha do usuário autenticado.
    ///
    /// A nova senha é encriptada com RSA-OAEP antes do envio,
    /// usando a chave pública gerada pelo servidor no momento da chamada.
    ///
    /// Regras impostas pela API:
    /// - mínimo 8 caracteres, máximo ~20 (limite do RSA 512-bit + OAEP = 22 bytes)
    /// - deve conter pelo menos 1 número e 1 letra
    /// - não pode ser `abcd1234` ou `abc123`
    pub fn change_password(&self, new_password: &str) -> Result<(), MedxError> {
        // 1. Busca chave pública RSA
        let keys: serde_json::Value = self.get("security/getkeys")?;
        let key_id = keys["KeyId"]
            .as_str()
            .ok_or_else(|| MedxError::UnexpectedResponse("KeyId ausente".into()))?
            .to_string();
        let public_key_xml = keys["PublicKey"]
            .as_str()
            .ok_or_else(|| MedxError::UnexpectedResponse("PublicKey ausente".into()))?;

        // 2. Encripta nova senha com RSA-OAEP
        let rsa_key = crate::crypto::rsa_public_key_from_xml(public_key_xml)?;
        let encrypted_pwd = crate::crypto::rsa_oaep_encrypt(&rsa_key, new_password)?;

        // 3. Obtém UserId do usuário atual
        let user = self.current_user()?;

        // 4. Envia DTO
        let dto = ChangePasswordDto {
            pwd: encrypted_pwd,
            key_id,
            uid: user.user_id,
            cripto: true,
        };

        self.post::<_, serde_json::Value>("usuarios/ChangeMePassword", &dto)?;
        Ok(())
    }
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const CURRENT_USER_JSON: &str = r#"{
        "DbId": 4242,
        "Username": "USUARIO TESTE",
        "UserFullName": "CONTA DE TESTE",
        "UserId": -7,
        "LastLogin": "2026-03-17T13:24:04.8582041+00:00",
        "Plano": "FULL",
        "Classificacao": "GOLD",
        "BloqueioSMS": false,
        "BloqueioInsert": false,
        "BloqueioAcesso": false,
        "UserEmail": "conta@example.invalid",
        "RdStationKey": "",
        "EstoqueAtualizado": "True",
        "isTrial": false
    }"#;

    pub const USER_JSON: &str = r#"{
        "CPF": "12345678900",
        "Id_da_Assinatura": 0,
        "Id_do_Usuario": 42,
        "Usuario": "JOAO",
        "Id_do_Setor": 1,
        "Ativo": true,
        "Agenda": true,
        "Senha": "",
        "Carimbo": "",
        "Cidade": "GOIANIA",
        "Pacientes": true,
        "Prontuario": false,
        "Agenda_s": false,
        "Gerencia": true,
        "Estatistica": false,
        "Usuarios": false,
        "Configuracao": false,
        "Prontuario_exibe": false,
        "Prontuario_escreve": false,
        "Atendimentos": false,
        "Financeiro": false,
        "Id_da_Unidade_de_Negocios": 1001,
        "Estoque": 0,
        "hora1i": "08:00", "hora2i": "08:00", "hora3i": "08:00",
        "hora4i": "08:00", "hora5i": "08:00", "hora6i": "08:00", "hora7i": "00:00",
        "hora1f": "18:00", "hora2f": "18:00", "hora3f": "18:00",
        "hora4f": "18:00", "hora5f": "18:00", "hora6f": "18:00", "hora7f": "00:00",
        "Marketing": false,
        "Dashboard": true,
        "Qualidade": false,
        "Nome_Completo": "JOAO DA SILVA",
        "Conselho_Profissional": "CRM",
        "Numero_do_Conselho": "12345",
        "UF_Conselho": "GO",
        "Relatorios_Financeiros": false,
        "Cripto": true,
        "Validade": "2026-12-31T00:00:00",
        "Ultima_senha": "ABC",
        "Tentativas": 0,
        "Grupos": "",
        "E_Grupo": false,
        "Biometria": false,
        "Portal": false,
        "Faturamento_editar": false
    }"#;

    #[test]
    fn deserializa_current_user() {
        let user: CurrentUser = serde_json::from_str(CURRENT_USER_JSON).unwrap();
        assert_eq!(user.db_id, 4242);
        assert_eq!(user.username, "USUARIO TESTE");
        assert_eq!(user.email, "conta@example.invalid");
        assert_eq!(user.plan, "FULL");
        assert!(!user.is_trial);
        assert!(!user.sms_blocked);
        assert_eq!(user.stock_updated_raw, "True");
    }

    #[test]
    fn stock_updated_helper_funciona() {
        let user: CurrentUser = serde_json::from_str(CURRENT_USER_JSON).unwrap();
        assert!(user.stock_updated());

        let json_false = CURRENT_USER_JSON.replace("\"True\"", "\"False\"");
        let user2: CurrentUser = serde_json::from_str(&json_false).unwrap();
        assert!(!user2.stock_updated());
    }

    #[test]
    fn deserializa_user_com_permissoes() {
        let user: User = serde_json::from_str(USER_JSON).unwrap();
        assert_eq!(user.id, 42);
        assert_eq!(user.username, "JOAO");
        assert_eq!(user.full_name, "JOAO DA SILVA");
        assert!(user.active);
        assert!(user.perm_agenda);
        assert!(user.perm_patients);
        assert!(!user.perm_medical_records);
        assert!(user.perm_management);
        assert_eq!(user.city, "GOIANIA");
        assert_eq!(user.professional_council, "CRM");
        assert_eq!(user.schedule_mon_start, "08:00");
        assert_eq!(user.schedule_mon_end, "18:00");
        assert_eq!(user.schedule_sun_start, "00:00");
    }

    #[test]
    fn deserializa_lista_de_users() {
        let json = format!("[{}, {}]", USER_JSON, USER_JSON);
        let users: Vec<User> = serde_json::from_str(&json).unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, 42);
    }

    #[test]
    fn user_validade_none_para_null() {
        let json = USER_JSON.replace(
            r#""Validade": "2026-12-31T00:00:00""#,
            r#""Validade": null"#,
        );
        let user: User = serde_json::from_str(&json).unwrap();
        assert!(user.validity.is_none());
    }

    #[test]
    fn current_user_user_id_negativo_aceito() {
        // UserId pode ser negativo na API real
        let user: CurrentUser = serde_json::from_str(CURRENT_USER_JSON).unwrap();
        assert_eq!(user.user_id, -7);
    }
}
