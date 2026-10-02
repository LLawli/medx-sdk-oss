//! Etapa 6 — Finanças: atendimentos (faturas), pagamentos e cobrança Stone/Pagar.me.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{client::MedxClient, error::MedxError, util::encode_query_value};

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

/// Registro financeiro de atendimento retornado pelos endpoints `GetAtendimentos*`.
///
/// Cada atendimento corresponde a uma fatura emitida para um paciente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendance {
    /// ID do atendimento (string — pode ser negativo representado como texto).
    #[serde(rename(deserialize = "Iddoatendimento"), deserialize_with = "de_null_str")]
    pub id: String,
    #[serde(rename(deserialize = "Iddocliente"), deserialize_with = "de_null_str")]
    pub patient_id: String,
    #[serde(rename(deserialize = "NomeCliente"), deserialize_with = "de_null_str")]
    pub patient_name: String,
    #[serde(rename(deserialize = "Data"), deserialize_with = "de_null_str")]
    pub date: String,
    #[serde(default, rename(deserialize = "IddoConvenio"), deserialize_with = "de_null_str")]
    pub convenio_id: String,
    #[serde(default, rename(deserialize = "Valor_Fatura"), deserialize_with = "de_null_f64")]
    pub invoice_value: f64,
    #[serde(default, rename(deserialize = "TotPgto"), deserialize_with = "de_null_f64")]
    pub total_paid: f64,
    #[serde(default, rename(deserialize = "Subtotal"), deserialize_with = "de_null_f64")]
    pub subtotal: f64,
    #[serde(default, rename(deserialize = "Desc_real"), deserialize_with = "de_null_f64")]
    pub discount_real: f64,
    #[serde(default, rename(deserialize = "Desc_perc"))]
    pub discount_pct: Option<f64>,
    /// `1` = cancelado, `2` = pendente.
    #[serde(default, rename(deserialize = "Desconto"), deserialize_with = "de_null_i64")]
    pub discount_status: i64,
    /// `0` = aberto, `1` = fechado.
    #[serde(default, rename(deserialize = "Fechado"), deserialize_with = "de_null_i64")]
    pub closed: i64,
    /// `0` = fatura, `1` = orçamento.
    #[serde(default, rename(deserialize = "Orcamento"), deserialize_with = "de_null_i64")]
    pub budget: i64,
    #[serde(default, rename(deserialize = "NF"), deserialize_with = "de_null_str")]
    pub nf: String,
    #[serde(default, rename(deserialize = "NumerodaGuia"), deserialize_with = "de_null_str")]
    pub guide_number: String,
    #[serde(default, rename(deserialize = "Recibo"), deserialize_with = "de_null_str")]
    pub receipt: String,
    #[serde(default, rename(deserialize = "Filial"), deserialize_with = "de_null_str")]
    pub branch: String,
    #[serde(default, rename(deserialize = "Tabela"), deserialize_with = "de_null_str")]
    pub table: String,
    #[serde(default, rename(deserialize = "Observacoes"), deserialize_with = "de_null_str")]
    pub notes: String,
    #[serde(default, rename(deserialize = "Iddousuario"), deserialize_with = "de_null_i64")]
    pub user_id: i64,
    #[serde(default, rename(deserialize = "IddaUnidadedeNegocios"), deserialize_with = "de_null_str")]
    pub business_unit_id: String,
    #[serde(default, rename(deserialize = "IddaAssinatura"))]
    pub subscription_id: Option<i64>,
}

impl Attendance {
    /// Retorna `true` se a fatura está fechada/liquidada.
    pub fn is_closed(&self) -> bool {
        self.closed != 0
    }

    /// Retorna `true` se este registro é um orçamento (não fatura confirmada).
    pub fn is_budget(&self) -> bool {
        self.budget != 0
    }

    /// Saldo pendente = valor da fatura - total pago.
    pub fn balance_due(&self) -> f64 {
        self.invoice_value - self.total_paid
    }
}

/// Item de procedimento dentro de uma fatura (`procedimentos` array em `AttendanceDto`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttendanceProcedure {
    /// `"INSERT"` para novos itens, `"UPDATE"` para atualizar.
    #[serde(rename = "Acao")]
    pub action: String,
    #[serde(rename = "Iddoatendimento")]
    pub attendance_id: String,
    /// ID único deste item (use `generate_item_id()` se não souber).
    #[serde(rename = "Iddoitemdoatendimento")]
    pub item_id: String,
    #[serde(rename = "Iddoprocedimento")]
    pub procedure_id: String,
    #[serde(rename = "Procedimento")]
    pub procedure_name: String,
    #[serde(rename = "Qtd")]
    pub quantity: i64,
    #[serde(rename = "ValorUnitario")]
    pub unit_value: f64,
    #[serde(rename = "Comissao")]
    pub commission: f64,
    #[serde(rename = "Sessoes")]
    pub sessions: i64,
    /// Percentual de rateio para o 1º profissional (padrão 100).
    #[serde(rename = "RateioPerc1")]
    pub split_pct_1: f64,
    #[serde(rename = "RateioPerc2")]
    pub split_pct_2: f64,
    #[serde(rename = "RateioPerc3")]
    pub split_pct_3: f64,
    #[serde(rename = "RateioPerc4")]
    pub split_pct_4: f64,
    #[serde(rename = "RateioProf1")]
    pub split_prof_1: String,
    #[serde(rename = "RateioProf2")]
    pub split_prof_2: String,
    #[serde(rename = "RateioProf3")]
    pub split_prof_3: String,
    #[serde(rename = "RateioProf4")]
    pub split_prof_4: String,
    #[serde(rename = "ItensEstoque")]
    pub stock_items: Vec<serde_json::Value>,
    #[serde(rename = "index")]
    pub index: i64,
}

impl AttendanceProcedure {
    /// Cria um item de procedimento com defaults mínimos para inserção.
    ///
    /// - `attendance_id`: ID do atendimento pai
    /// - `procedure_id`: ID do procedimento (de `Procedure.id`)
    /// - `procedure_name`: nome do procedimento
    /// - `unit_value`: valor unitário
    /// - `prof_id`: ID do profissional responsável (string)
    pub fn new(
        attendance_id: impl Into<String>,
        procedure_id: impl Into<String>,
        procedure_name: impl Into<String>,
        unit_value: f64,
        prof_id: impl Into<String>,
    ) -> Self {
        AttendanceProcedure {
            action: "INSERT".to_string(),
            attendance_id: attendance_id.into(),
            item_id: generate_item_id(),
            procedure_id: procedure_id.into(),
            procedure_name: procedure_name.into(),
            quantity: 1,
            unit_value,
            commission: 0.0,
            sessions: 1,
            split_pct_1: 100.0,
            split_prof_1: prof_id.into(),
            index: -1,
            ..Default::default()
        }
    }
}

/// DTO para criar ou atualizar uma fatura (`POST Atendimentos/UpdateFaturaGeral`).
///
/// Apesar do nome "Update", este endpoint cria e atualiza faturas conforme o campo `action`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttendanceDto {
    /// `"INSERT"` para criar, `"UPDATE"` para atualizar.
    #[serde(rename = "Acao")]
    pub action: String,
    #[serde(rename = "Iddoatendimento")]
    pub id: String,
    #[serde(rename = "Iddocliente")]
    pub patient_id: String,
    #[serde(rename = "NomeCliente")]
    pub patient_name: String,
    #[serde(rename = "Data")]
    pub date: String,
    #[serde(rename = "IddoConvenio")]
    pub convenio_id: String,
    #[serde(rename = "Iddousuario")]
    pub user_id: i64,
    #[serde(rename = "IddaUnidadedeNegocios")]
    pub business_unit_id: String,
    #[serde(rename = "Valor_Fatura")]
    pub invoice_value: f64,
    #[serde(rename = "Subtotal")]
    pub subtotal: f64,
    #[serde(rename = "TotPgto")]
    pub total_paid: f64,
    #[serde(rename = "Desc_real")]
    pub discount_real: f64,
    #[serde(rename = "Desc_perc")]
    pub discount_pct: Option<f64>,
    /// `1` = cancelado, `2` = pendente.
    #[serde(rename = "Desconto")]
    pub discount_status: i64,
    /// `0` = aberto, `1` = fechado.
    #[serde(rename = "Fechado")]
    pub closed: i64,
    /// `0` = fatura, `1` = orçamento.
    #[serde(rename = "Orcamento")]
    pub budget: i64,
    #[serde(rename = "NF")]
    pub nf: String,
    #[serde(rename = "NumerodaGuia")]
    pub guide_number: String,
    #[serde(rename = "Recibo")]
    pub receipt: String,
    #[serde(rename = "Filial")]
    pub branch: String,
    #[serde(rename = "Tabela")]
    pub table: String,
    #[serde(rename = "Observacoes")]
    pub notes: String,
    #[serde(rename = "Converteu")]
    pub converted: i64,
    #[serde(rename = "IddaAssinatura")]
    pub subscription_id: Option<i64>,
    #[serde(rename = "procedimentos")]
    pub procedures: Vec<AttendanceProcedure>,
    #[serde(rename = "pagamentos")]
    pub payments: Vec<serde_json::Value>,
}

impl AttendanceDto {
    /// Cria um DTO mínimo para nova fatura.
    ///
    /// - `patient_id`: ID do paciente (string)
    /// - `patient_name`: nome do paciente
    /// - `convenio_id`: ID do convênio (string)
    /// - `user_id`: ID do profissional responsável
    /// - `date`: ISO 8601, ex: `"2026-03-20T09:00:00"`
    pub fn new(
        patient_id: impl Into<String>,
        patient_name: impl Into<String>,
        convenio_id: impl Into<String>,
        user_id: i64,
        date: impl Into<String>,
    ) -> Self {
        AttendanceDto {
            action: "INSERT".to_string(),
            id: generate_item_id(),
            patient_id: patient_id.into(),
            patient_name: patient_name.into(),
            convenio_id: convenio_id.into(),
            user_id,
            date: date.into(),
            discount_status: 2, // pendente
            ..Default::default()
        }
    }
}

impl From<Attendance> for AttendanceDto {
    fn from(a: Attendance) -> Self {
        AttendanceDto {
            action: "UPDATE".to_string(),
            id: a.id,
            patient_id: a.patient_id,
            patient_name: a.patient_name,
            date: a.date,
            convenio_id: a.convenio_id,
            user_id: a.user_id,
            business_unit_id: a.business_unit_id,
            invoice_value: a.invoice_value,
            subtotal: a.subtotal,
            total_paid: a.total_paid,
            discount_real: a.discount_real,
            discount_pct: a.discount_pct,
            discount_status: a.discount_status,
            closed: a.closed,
            budget: a.budget,
            nf: a.nf,
            guide_number: a.guide_number,
            receipt: a.receipt,
            branch: a.branch,
            table: a.table,
            notes: a.notes,
            subscription_id: a.subscription_id,
            ..Default::default()
        }
    }
}

/// DTO para gerar cobrança via Stone/Pagar.me (`POST stone/CreatePrePayment`).
#[derive(Debug, Clone, Serialize)]
pub struct PrePaymentDto {
    #[serde(rename = "PatientId")]
    pub patient_id: String,
    /// Valor em reais (ex: `150.0`).
    #[serde(rename = "Value")]
    pub value: f64,
    #[serde(rename = "Clinica")]
    pub clinic: String,
    #[serde(rename = "EmailPaciente")]
    pub patient_email: String,
    #[serde(rename = "NomedoPaciente")]
    pub patient_name: String,
    /// Número de parcelas (padrão `1`).
    #[serde(rename = "Installments")]
    pub installments: i64,
}

impl PrePaymentDto {
    pub fn new(
        patient_id: impl Into<String>,
        value: f64,
        clinic: impl Into<String>,
        patient_email: impl Into<String>,
        patient_name: impl Into<String>,
    ) -> Self {
        PrePaymentDto {
            patient_id: patient_id.into(),
            value,
            clinic: clinic.into(),
            patient_email: patient_email.into(),
            patient_name: patient_name.into(),
            installments: 1,
        }
    }
}

// ── Métodos do MedxClient ─────────────────────────────────────────────────────

impl MedxClient {
    /// Retorna todos os atendimentos (faturas) de um paciente.
    pub fn attendances_by_patient(&self, patient_id: i64) -> Result<Vec<Attendance>, MedxError> {
        let text = self.get_text(&format!(
            "atendimentos/GetAtendimentosByIdPac?Id={patient_id}"
        ))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Retorna todos os atendimentos da conta com filtro opcional.
    ///
    /// - `filter`: período/status do dropdown, um de `""`, `"Últimos 7 Dias"`,
    ///   `"Pendências"`, `"Faturas Canceladas"`, `"Orçamentos em aberto"`.
    /// - `filterstring`: texto de busca livre.
    pub fn all_attendances(
        &self,
        filter: &str,
        filterstring: &str,
    ) -> Result<Vec<Attendance>, MedxError> {
        let text = self.get_text(&format!(
            "atendimentos/GetAllAtendimentos?filterstring={}&filter={}",
            encode_query_value(filterstring),
            encode_query_value(filter),
        ))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Cria ou atualiza uma fatura de atendimento.
    ///
    /// Use `AttendanceDto::new()` para criar, ou `AttendanceDto::from(attendance)` para atualizar.
    pub fn update_invoice(&self, dto: &AttendanceDto) -> Result<(), MedxError> {
        self.post::<_, serde_json::Value>("Atendimentos/UpdateFaturaGeral", dto)?;
        Ok(())
    }

    /// Gera uma cobrança via Stone/Pagar.me e retorna o localizador da transação.
    ///
    /// Retorna `None` se o campo `localizador` não estiver presente na resposta.
    pub fn create_pre_payment(&self, dto: &PrePaymentDto) -> Result<Option<String>, MedxError> {
        let resp: serde_json::Value = self.post("stone/CreatePrePayment", dto)?;
        Ok(resp
            .get("localizador")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()))
    }
}

// ── Helpers internos ──────────────────────────────────────────────────────────

/// Gera um ID de item baseado em hash de tempo (mesmo padrão de `generate_contact_id`).
pub fn generate_item_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    let id = (h.finish() as i64).unsigned_abs() as i64 % 2_000_000_000;
    id.to_string()
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const ATTENDANCE_JSON: &str = r#"{
        "Iddoatendimento": "123456789",
        "Iddocliente": "100002",
        "NomeCliente": "MARIA SILVA",
        "Data": "2026-03-17T09:00:00",
        "IddoConvenio": "1",
        "Valor_Fatura": 150.0,
        "TotPgto": 150.0,
        "Subtotal": 150.0,
        "Desc_real": 0.0,
        "Desconto": 2,
        "Fechado": 1,
        "Orcamento": 0,
        "NF": "",
        "NumerodaGuia": "",
        "Recibo": "REC001",
        "Filial": "",
        "Tabela": "",
        "Observacoes": "",
        "Iddousuario": 7,
        "IddaUnidadedeNegocios": "1001"
    }"#;

    #[test]
    fn deserializa_attendance() {
        let a: Attendance = serde_json::from_str(ATTENDANCE_JSON).unwrap();
        assert_eq!(a.id, "123456789");
        assert_eq!(a.patient_id, "100002");
        assert_eq!(a.patient_name, "MARIA SILVA");
        assert!((a.invoice_value - 150.0).abs() < f64::EPSILON);
        assert!(a.is_closed());
        assert!(!a.is_budget());
        assert!((a.balance_due()).abs() < f64::EPSILON);
    }

    #[test]
    fn attendance_into_dto() {
        let a: Attendance = serde_json::from_str(ATTENDANCE_JSON).unwrap();
        let dto: AttendanceDto = a.into();
        assert_eq!(dto.action, "UPDATE");
        assert_eq!(dto.id, "123456789");
        assert_eq!(dto.patient_name, "MARIA SILVA");
        assert!((dto.invoice_value - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn attendance_dto_new_defaults() {
        let dto = AttendanceDto::new("123", "JOAO SILVA", "1", 42, "2026-03-17T09:00:00");
        assert_eq!(dto.action, "INSERT");
        assert_eq!(dto.patient_id, "123");
        assert_eq!(dto.convenio_id, "1");
        assert_eq!(dto.user_id, 42);
        assert_eq!(dto.discount_status, 2);
        assert!(dto.procedures.is_empty());
        assert!(!dto.id.is_empty(), "id deve ser gerado automaticamente");
    }

    #[test]
    fn attendance_procedure_new_defaults() {
        let p = AttendanceProcedure::new("att-1", "10", "CONSULTA", 150.0, "42");
        assert_eq!(p.action, "INSERT");
        assert_eq!(p.attendance_id, "att-1");
        assert_eq!(p.procedure_name, "CONSULTA");
        assert!((p.unit_value - 150.0).abs() < f64::EPSILON);
        assert_eq!(p.split_pct_1, 100.0);
        assert_eq!(p.split_prof_1, "42");
        assert_eq!(p.quantity, 1);
        assert_eq!(p.index, -1);
    }

    #[test]
    fn serializa_attendance_dto_campos_api() {
        let dto = AttendanceDto::new("1", "TESTE", "1", 1, "2026-01-01T00:00:00");
        let v = serde_json::to_value(&dto).unwrap();
        assert!(v.get("Acao").is_some());
        assert!(v.get("Iddoatendimento").is_some());
        assert!(v.get("Iddocliente").is_some());
        assert!(v.get("procedimentos").is_some());
        assert!(v.get("pagamentos").is_some());
    }

    #[test]
    fn serializa_pre_payment_dto() {
        let dto = PrePaymentDto::new("123", 200.0, "CLINICA TESTE", "p@test.com", "JOAO");
        let v = serde_json::to_value(&dto).unwrap();
        assert_eq!(v["PatientId"], "123");
        assert_eq!(v["Value"], 200.0);
        assert_eq!(v["Installments"], 1);
    }

    #[test]
    fn balance_due_calcula_corretamente() {
        let a: Attendance = serde_json::from_str(ATTENDANCE_JSON).unwrap();
        // invoice_value=150 total_paid=150 → balance=0
        assert!((a.balance_due()).abs() < f64::EPSILON);
    }
}
