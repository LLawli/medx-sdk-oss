//! Etapa 4 — Agenda (agendamentos, setores, parâmetros, relatórios).

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
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_string<E: serde::de::Error>(self, v: String) -> Result<String, E> {
            Ok(v)
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<String, E> {
            Ok(String::new())
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<String, E> {
            Ok(String::new())
        }
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
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<i64, E> {
            Ok(v)
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<i64, E> {
            Ok(v as i64)
        }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<i64, E> {
            Ok(v as i64)
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<i64, E> {
            v.trim()
                .parse::<i64>()
                .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<i64, E> {
            Ok(0)
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<i64, E> {
            Ok(0)
        }
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

/// Mapeamento de status → cor e label (`colorandlabels` em `GetAllParametersAgenda`).
///
/// O índice corresponde ao valor numérico do campo `Status` de um `Appointment`.
/// Use `AgendaColorLabels::label_for(status)` e `AgendaColorLabels::color_for(status)`
/// para resolver status → texto e cor hex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaColorLabels {
    #[serde(rename(deserialize = "Colors"))]
    pub colors: Vec<String>,
    #[serde(rename(deserialize = "Labels"))]
    pub labels: Vec<String>,
}

impl AgendaColorLabels {
    /// Retorna o label textual para um dado `status`, ou `None` se fora do intervalo.
    pub fn label_for(&self, status: i64) -> Option<&str> {
        self.labels.get(status as usize).map(|s| s.as_str())
    }

    /// Retorna a cor hex para um dado `status`, ou `None` se fora do intervalo.
    pub fn color_for(&self, status: i64) -> Option<&str> {
        self.colors.get(status as usize).map(|s| s.as_str())
    }
}

/// Parâmetros completos da agenda retornados por `GET agenda/GetAllParametersAgenda`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaParameters {
    #[serde(rename(deserialize = "colorandlabels"))]
    pub color_labels: AgendaColorLabels,
    #[serde(rename(deserialize = "agendausuarios"))]
    pub users: Vec<AgendaUser>,
    #[serde(rename(deserialize = "agendasetores"))]
    pub sectors: Vec<AgendaSector>,
    #[serde(rename(deserialize = "parametrosgerais"))]
    pub general: AgendaGeneralParams,
}

/// Parâmetros gerais de configuração da agenda (primeiro elemento de `parametrosgerais`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaGeneralParams {
    /// Duração padrão de cada slot, ex: `"0030"` = 30 min.
    #[serde(rename(deserialize = "duracao"), deserialize_with = "de_null_str")]
    pub slot_duration: String,
    /// Horário de início do expediente, ex: `"0800"`.
    #[serde(rename(deserialize = "horariode"), deserialize_with = "de_null_str")]
    pub start_time: String,
    /// Horário de fim do expediente, ex: `"2200"`.
    #[serde(rename(deserialize = "horarioate"), deserialize_with = "de_null_str")]
    pub end_time: String,
}

/// Usuário habilitado para a agenda (`GET agenda/GetAllParametersAgenda` → `agendausuarios`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaUser {
    #[serde(
        rename(deserialize = "Id_do_Usuario"),
        deserialize_with = "de_null_i64"
    )]
    pub id: i64,
    #[serde(rename(deserialize = "Usuario"), deserialize_with = "de_null_str")]
    pub username: String,
    #[serde(rename(deserialize = "Id_do_Setor"), deserialize_with = "de_null_i64")]
    pub sector_id: i64,
    #[serde(rename(deserialize = "Agenda"), deserialize_with = "de_null_bool")]
    pub has_agenda: bool,
    // Horários de trabalho (seg–dom)
    #[serde(rename(deserialize = "hora1i"), deserialize_with = "de_null_str")]
    pub mon_start: String,
    #[serde(rename(deserialize = "hora1f"), deserialize_with = "de_null_str")]
    pub mon_end: String,
    #[serde(rename(deserialize = "hora2i"), deserialize_with = "de_null_str")]
    pub tue_start: String,
    #[serde(rename(deserialize = "hora2f"), deserialize_with = "de_null_str")]
    pub tue_end: String,
    #[serde(rename(deserialize = "hora3i"), deserialize_with = "de_null_str")]
    pub wed_start: String,
    #[serde(rename(deserialize = "hora3f"), deserialize_with = "de_null_str")]
    pub wed_end: String,
    #[serde(rename(deserialize = "hora4i"), deserialize_with = "de_null_str")]
    pub thu_start: String,
    #[serde(rename(deserialize = "hora4f"), deserialize_with = "de_null_str")]
    pub thu_end: String,
    #[serde(rename(deserialize = "hora5i"), deserialize_with = "de_null_str")]
    pub fri_start: String,
    #[serde(rename(deserialize = "hora5f"), deserialize_with = "de_null_str")]
    pub fri_end: String,
    #[serde(rename(deserialize = "hora6i"), deserialize_with = "de_null_str")]
    pub sat_start: String,
    #[serde(rename(deserialize = "hora6f"), deserialize_with = "de_null_str")]
    pub sat_end: String,
    #[serde(rename(deserialize = "hora7i"), deserialize_with = "de_null_str")]
    pub sun_start: String,
    #[serde(rename(deserialize = "hora7f"), deserialize_with = "de_null_str")]
    pub sun_end: String,
}

/// Setor da agenda (`GET agenda/getagendasetores`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaSector {
    #[serde(rename(deserialize = "Id_do_Setor"), deserialize_with = "de_null_i64")]
    pub id: i64,
    #[serde(rename(deserialize = "Setor"), deserialize_with = "de_null_str")]
    pub name: String,
}

/// Agendamento retornado por `GET hoje/GetAgendaDiaUsuario`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    /// Campo retornado como `Id_Do_Agendamento` (D maiúsculo) pela API.
    #[serde(
        rename(deserialize = "Id_Do_Agendamento"),
        deserialize_with = "de_null_i64"
    )]
    pub id: i64,
    /// Pode estar ausente em alguns endpoints; usa 0 como padrão.
    #[serde(
        default,
        rename(deserialize = "Id_do_Usuario"),
        deserialize_with = "de_null_i64"
    )]
    pub user_id: i64,
    #[serde(default, rename(deserialize = "Id_da_Assinatura"))]
    pub subscription_id: Option<i64>,
    /// Datetime ISO 8601 de início.
    #[serde(rename(deserialize = "Inicio"), deserialize_with = "de_null_str")]
    pub start: String,
    /// Datetime ISO 8601 de fim.
    #[serde(rename(deserialize = "Final"), deserialize_with = "de_null_str")]
    pub end: String,
    /// Nome ou descrição do agendamento.
    #[serde(rename(deserialize = "Descricao"), deserialize_with = "de_null_str")]
    pub description: String,
    /// Status numérico (índice em `AgendaColorLabels`).
    #[serde(rename(deserialize = "Status"), deserialize_with = "de_null_i64")]
    pub status: i64,
    /// ID do procedimento vinculado.
    #[serde(default, rename(deserialize = "Id_do_Procedimento"))]
    pub procedure_id: Option<i64>,
    /// ID do contato (paciente) vinculado.
    #[serde(default, rename(deserialize = "Vinculado_a"))]
    pub contact_id: Option<i64>,
    #[serde(
        default,
        rename(deserialize = "Id_do_Diagnostico_QP"),
        deserialize_with = "de_null_i64"
    )]
    pub diagnostic_id: i64,
    /// Datetime de confirmação via WhatsApp/SMS.
    #[serde(default, rename(deserialize = "Confirmacao"))]
    pub confirmed_at: Option<String>,
    /// Datetime de chegada na clínica (`"0001-01-01T00:00:00"` = não chegou).
    #[serde(rename(deserialize = "Chegada"), deserialize_with = "de_null_str")]
    pub arrived_at: String,
    /// Datetime de início do atendimento.
    #[serde(rename(deserialize = "Atendido_As"), deserialize_with = "de_null_str")]
    pub attended_at: String,
    /// Número de telefone para SMS.
    #[serde(default, rename(deserialize = "SMS"))]
    pub sms: Option<String>,
    /// Datetime de saída da clínica.
    #[serde(default, rename(deserialize = "Saiu_as"))]
    pub left_at: Option<String>,
}

impl Appointment {
    /// Retorna `true` se o paciente chegou (arrived_at não é a data sentinela).
    pub fn has_arrived(&self) -> bool {
        !self.arrived_at.starts_with("0001-01-01")
    }

    /// Retorna `true` se o atendimento começou.
    pub fn is_attended(&self) -> bool {
        !self.attended_at.starts_with("0001-01-01")
    }

    /// Retorna `true` se este registro é um bloqueio de agenda (status == -1).
    /// Retorna `true` se este registro é um bloqueio de agenda (sem paciente vinculado).
    pub fn is_block(&self) -> bool {
        self.contact_id.map(|id| id == 0).unwrap_or(true)
    }
}

/// DTO para criar ou atualizar um agendamento.
///
/// Use `AppointmentDto::new()` para obter um objeto com defaults seguros.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentDto {
    #[serde(rename = "Id_do_Agendamento")]
    pub id: i64,
    #[serde(rename = "Id_do_Usuario")]
    pub user_id: i64,
    #[serde(rename = "Id_da_Assinatura")]
    pub subscription_id: Option<i64>,
    #[serde(rename = "Inicio")]
    pub start: String,
    #[serde(rename = "Final")]
    pub end: String,
    #[serde(rename = "Descricao")]
    pub description: String,
    #[serde(rename = "Status")]
    pub status: i64,
    #[serde(rename = "Id_do_Procedimento")]
    pub procedure_id: Option<i64>,
    #[serde(rename = "Vinculado_a")]
    pub contact_id: Option<i64>,
    #[serde(rename = "Id_do_Diagnostico_QP")]
    pub diagnostic_id: Option<i64>,
    #[serde(rename = "Confirmacao")]
    pub confirmed_at: Option<String>,
    #[serde(rename = "Chegada")]
    pub arrived_at: Option<String>,
    #[serde(rename = "SMS")]
    pub sms: String,
    #[serde(rename = "Atendido_as")]
    pub attended_at: Option<String>,
    #[serde(rename = "Saiu_as")]
    pub left_at: Option<String>,
    // Campos para agendamento recorrente
    #[serde(rename = "RepetirSemanas")]
    pub repeat_weeks: i64,
    #[serde(rename = "Segunda")]
    pub monday: i64,
    #[serde(rename = "Terca")]
    pub tuesday: i64,
    #[serde(rename = "Quarta")]
    pub wednesday: i64,
    #[serde(rename = "Quinta")]
    pub thursday: i64,
    #[serde(rename = "Sexta")]
    pub friday: i64,
    #[serde(rename = "Sabado")]
    pub saturday: i64,
    #[serde(rename = "Domingo")]
    pub sunday: i64,
    #[serde(rename = "OutrosProfissionais")]
    pub other_professionals: Vec<serde_json::Value>,
}

impl AppointmentDto {
    /// Cria um DTO mínimo para novo agendamento.
    ///
    /// - `user_id`: ID do profissional responsável
    /// - `start`/`end`: strings ISO 8601, ex: `"2026-03-20T08:00:00"`
    pub fn new(user_id: i64, start: impl Into<String>, end: impl Into<String>) -> Self {
        AppointmentDto {
            id: 0,
            user_id,
            subscription_id: None,
            start: start.into(),
            end: end.into(),
            description: String::new(),
            status: 0,
            procedure_id: None,
            contact_id: None,
            diagnostic_id: None,
            confirmed_at: None,
            arrived_at: None,
            sms: String::new(),
            attended_at: None,
            left_at: None,
            repeat_weeks: 0,
            monday: 0,
            tuesday: 0,
            wednesday: 0,
            thursday: 0,
            friday: 0,
            saturday: 0,
            sunday: 0,
            other_professionals: vec![],
        }
    }

    /// Adiciona um profissional extra ao agendamento (agendamento compartilhado).
    ///
    /// Preencher `other_professionals` faz `create_appointment` rotear para o
    /// endpoint de agendamento recorrente/múltiplo, como no frontend legado.
    /// O legado envia apenas os IDs numéricos dos profissionais.
    pub fn add_other_professional(&mut self, user_id: i64) {
        self.other_professionals
            .push(serde_json::Value::from(user_id));
    }
}

impl From<Appointment> for AppointmentDto {
    fn from(a: Appointment) -> Self {
        AppointmentDto {
            id: a.id,
            user_id: a.user_id,
            subscription_id: a.subscription_id,
            start: a.start,
            end: a.end,
            description: a.description,
            status: a.status,
            procedure_id: a.procedure_id,
            contact_id: a.contact_id,
            diagnostic_id: if a.diagnostic_id == 0 {
                None
            } else {
                Some(a.diagnostic_id)
            },
            confirmed_at: a.confirmed_at,
            arrived_at: if a.arrived_at.starts_with("0001-01-01") {
                None
            } else {
                Some(a.arrived_at)
            },
            sms: a.sms.unwrap_or_default(),
            attended_at: if a.attended_at.starts_with("0001-01-01") {
                None
            } else {
                Some(a.attended_at)
            },
            left_at: a.left_at,
            repeat_weeks: 0,
            monday: 0,
            tuesday: 0,
            wednesday: 0,
            thursday: 0,
            friday: 0,
            saturday: 0,
            sunday: 0,
            other_professionals: vec![],
        }
    }
}

/// DTO para atualização de status de agendamento (`PUT agenda/updatestatus`).
#[derive(Debug, Clone, Serialize)]
pub struct AppointmentStatusDto {
    #[serde(rename = "Id_do_Agendamento")]
    pub id: i64,
    #[serde(rename = "Status")]
    pub status: i64,
}

/// DTO para relatório de agenda (`POST report/ReportAgenda`).
///
/// Use `AgendaReportDto::new(start, end, user_id)` para construir com defaults sensatos.
/// `user_id = 0` retorna todos os profissionais.
#[derive(Debug, Clone, Serialize)]
pub struct AgendaReportDto {
    #[serde(rename = "inicio")]
    pub start_date: String,
    #[serde(rename = "final")]
    pub end_date: String,
    /// ID do profissional; 0 = todos.
    #[serde(rename = "idUsuario")]
    pub user_id: i64,
    /// Mesmo valor de `user_id` encapsulado em array (campo exigido pela API).
    #[serde(rename = "Usuarioid")]
    pub user_ids: Vec<i64>,
    #[serde(rename = "ExportaXLS")]
    pub export_xls: bool,
    /// Incluir agendamentos desmarcados no relatório.
    #[serde(rename = "exibedesmarcados")]
    pub show_cancelled: bool,
    #[serde(rename = "textobusca")]
    pub search_text: String,
    /// Filtro por status; -1 = todos.
    #[serde(rename = "idStatus")]
    pub status_id: i64,
}

impl AgendaReportDto {
    pub fn new(start_date: &str, end_date: &str, user_id: i64) -> Self {
        Self {
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            user_ids: vec![user_id],
            user_id,
            export_xls: false,
            show_cancelled: false,
            search_text: String::new(),
            status_id: -1,
        }
    }
}

/// DTO para relatório de no-show.
///
/// Use `NoShowReportDto::new(start, end, user_id)` para construir.
/// `user_id = 0` usa o endpoint `ReportAgendaNoShowGeral` (todos os profissionais).
#[derive(Debug, Clone, Serialize)]
pub struct NoShowReportDto {
    #[serde(rename = "inicio")]
    pub start_date: String,
    #[serde(rename = "final")]
    pub end_date: String,
    /// ID do profissional; 0 = todos (usa endpoint Geral).
    #[serde(rename = "idUsuario")]
    pub user_id: i64,
    #[serde(rename = "Usuarioid")]
    pub user_ids: Vec<i64>,
    #[serde(rename = "ExportaXLS")]
    pub export_xls: bool,
    #[serde(rename = "exibedesmarcados")]
    pub show_cancelled: bool,
    #[serde(rename = "textobusca")]
    pub search_text: String,
    #[serde(rename = "idStatus")]
    pub status_id: i64,
}

impl NoShowReportDto {
    pub fn new(start_date: &str, end_date: &str, user_id: i64) -> Self {
        Self {
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            user_ids: vec![user_id],
            user_id,
            export_xls: false,
            show_cancelled: false,
            search_text: String::new(),
            status_id: -1,
        }
    }
}

/// Resposta dos endpoints de relatório de agenda.
/// `file_url` contém a URL do PDF/XLS gerado. Vazio indica ausência de dados no período.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportResponse {
    #[serde(rename(deserialize = "arquivo"), default)]
    pub file_url: String,
    #[serde(rename(deserialize = "mensagem"), default)]
    pub message: String,
}

/// Resultado de criação de agendamento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppointmentCreated {
    #[serde(rename(deserialize = "StatusAgendado"))]
    pub scheduled: bool,
    #[serde(rename(deserialize = "iddousuario"), deserialize_with = "de_null_i64")]
    pub user_id: i64,
}

/// Interpreta a resposta de inserção de agendamento como sucesso booleano.
///
/// A API MedX responde de três formas conforme o endpoint:
/// - string `"Success"` (case-insensitive) — `agenda/InsertAgendamento`;
/// - array `[{ "StatusAgendado": bool, ... }]`;
/// - objeto `{ "retorno": [{ "StatusAgendado", "iddousuario" }, ...] }` —
///   `agenda/insertagendamentorecorrente`.
///
/// Retorna `true` apenas se **todas** as ocorrências foram agendadas.
fn parse_insert_response(resp: &serde_json::Value) -> bool {
    if resp
        .as_str()
        .map(|s| s.eq_ignore_ascii_case("success"))
        .unwrap_or(false)
    {
        return true;
    }
    // Array direto ou envelopado em "retorno".
    let arr = resp
        .as_array()
        .or_else(|| resp.get("retorno").and_then(|v| v.as_array()));
    if let Some(arr) = arr {
        if arr.is_empty() {
            return false;
        }
        return arr.iter().all(|v| {
            v.get("StatusAgendado")
                .and_then(|s| s.as_bool())
                .unwrap_or(false)
        });
    }
    false
}

// ── Métodos do MedxClient ─────────────────────────────────────────────────────

impl MedxClient {
    /// Retorna parâmetros completos da agenda (usuários, setores, horários, cores/labels).
    pub fn agenda_parameters(&self) -> Result<AgendaParameters, MedxError> {
        self.get("agenda/GetAllParametersAgenda")
    }

    /// Retorna apenas os profissionais habilitados para a agenda.
    ///
    /// Equivale a `agenda_parameters().users` filtrado por `has_agenda == true`.
    pub fn agenda_users(&self) -> Result<Vec<AgendaUser>, MedxError> {
        let params = self.agenda_parameters()?;
        Ok(params.users.into_iter().filter(|u| u.has_agenda).collect())
    }

    /// Retorna a lista de setores da agenda.
    pub fn agenda_sectors(&self) -> Result<Vec<AgendaSector>, MedxError> {
        let text = self.get_text("agenda/getagendasetores")?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Retorna os agendamentos de um profissional em uma data específica.
    ///
    /// - `user_id`: ID do profissional
    /// - `date`: data no formato `"YYYY-MM-DD"` (ex: `"2026-03-20"`)
    pub fn daily_agenda(&self, user_id: i64, date: &str) -> Result<Vec<Appointment>, MedxError> {
        let text = self.get_text(&format!(
            "hoje/GetAgendaDiaUsuario?Id={user_id}&Dt={}",
            encode_query_value(date)
        ))?;
        if text.trim() == "null" || text.trim().is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&text).map_err(MedxError::Json)
    }

    /// Cria um novo agendamento.
    ///
    /// Retorna `true` se o agendamento foi inserido com sucesso.
    ///
    /// Se o DTO for novo (`id == 0`) e carregar recorrência (`repeat_weeks > 0`)
    /// ou profissionais extras (`other_professionals` não vazio), roteia para
    /// `agenda/insertagendamentorecorrente`, como o frontend legado; caso
    /// contrário usa `agenda/InsertAgendamento`.
    ///
    /// Pré-condição do endpoint recorrente: se `repeat_weeks > 0`, ao menos um
    /// dia da semana (`monday`..`sunday`) deve estar marcado — o servidor
    /// depende disso. Em sucesso parcial (nem todas as ocorrências agendadas)
    /// o retorno é `false`, ainda que algumas ocorrências tenham sido criadas.
    pub fn create_appointment(&self, dto: &AppointmentDto) -> Result<bool, MedxError> {
        let path = if dto.id == 0 && (dto.repeat_weeks > 0 || !dto.other_professionals.is_empty()) {
            "agenda/insertagendamentorecorrente"
        } else {
            "agenda/InsertAgendamento"
        };
        let resp: serde_json::Value = self.post(path, dto)?;
        Ok(parse_insert_response(&resp))
    }

    /// Cria um bloqueio de agenda para um profissional em um período.
    ///
    /// Um bloqueio é um agendamento sem paciente vinculado (`contact_id = None`).
    /// Para remover, use `remove_agenda_block(id)` que faz soft-delete (status=0).
    pub fn create_agenda_block(
        &self,
        user_id: i64,
        start: &str,
        end: &str,
    ) -> Result<bool, MedxError> {
        let mut dto = AppointmentDto::new(user_id, start, end);
        dto.status = 1;
        let resp: serde_json::Value = self.post("agenda/InsertAgendamento", &dto)?;
        Ok(parse_insert_response(&resp))
    }

    /// Remove um bloqueio de agenda (soft-delete: marca status=0 / DESMARCADO).
    ///
    /// A API não suporta deleção real de agendamentos sem paciente — o soft-delete
    /// é o único mecanismo disponível.
    pub fn remove_agenda_block(&self, id: i64) -> Result<(), MedxError> {
        self.update_appointment_status(id, 0)
    }

    /// Retorna os bloqueios de agenda de um profissional em uma data.
    ///
    /// Filtra a agenda diária por agendamentos sem paciente vinculado.
    pub fn daily_blocks(&self, user_id: i64, date: &str) -> Result<Vec<Appointment>, MedxError> {
        let all = self.daily_agenda(user_id, date)?;
        Ok(all.into_iter().filter(|a| a.is_block()).collect())
    }

    /// Atualiza um agendamento existente.
    ///
    /// O campo `dto.id` deve ser o ID do agendamento a ser atualizado.
    pub fn update_appointment(&self, dto: &AppointmentDto) -> Result<(), MedxError> {
        self.put::<_, serde_json::Value>("agenda/UpdateAgendamento", dto)?;
        Ok(())
    }

    /// Atualiza apenas o status de um agendamento.
    pub fn update_appointment_status(&self, id: i64, status: i64) -> Result<(), MedxError> {
        let dto = AppointmentStatusDto { id, status };
        self.put::<_, serde_json::Value>("agenda/updatestatus", &dto)?;
        Ok(())
    }

    /// Confirma o agendamento via WhatsApp (envia link de confirmação).
    ///
    /// O endpoint é `POST` com o id na query string (o frontend legado usa
    /// `postMethodWithParameters` sem corpo); um `GET` retorna 404/405.
    pub fn confirm_appointment_whatsapp(&self, appointment_id: i64) -> Result<(), MedxError> {
        self.post_empty(&format!(
            "agenda/ConfirmaAgendamentoWhatsapp?Iddoagendamento={appointment_id}"
        ))?;
        Ok(())
    }

    /// Gera relatório de agenda para um período e profissional.
    pub fn agenda_report(&self, dto: &AgendaReportDto) -> Result<ReportResponse, MedxError> {
        self.post("report/ReportAgenda", dto)
    }

    /// Exclui um agendamento pelo ID.
    pub fn delete_appointment(&self, id: i64) -> Result<(), MedxError> {
        self.delete(&format!("agenda/DeleteAgendamento?Id={id}"))
    }

    /// Gera relatório de no-show para um período.
    ///
    /// `dto.user_id == 0` usa `report/ReportAgendaNoShowGeral` (todos os profissionais).
    pub fn no_show_report(&self, dto: &NoShowReportDto) -> Result<ReportResponse, MedxError> {
        let endpoint = if dto.user_id == 0 {
            "report/ReportAgendaNoShowGeral"
        } else {
            "report/ReportAgendaNoShow"
        };
        self.post(endpoint, dto)
    }
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const AGENDA_PARAMS_JSON: &str = r##"{
        "colorandlabels": {
            "Colors": ["#A9A9A9", "#FFFFFF", "#90EE90"],
            "Labels": ["DESMARCADO", "AGENDADO", "COMPARECEU"]
        },
        "agendausuarios": [
            {
                "Id_do_Usuario": 42,
                "Usuario": "DR JOAO",
                "Id_do_Setor": 1,
                "Agenda": true,
                "hora1i": "08:00", "hora1f": "18:00",
                "hora2i": "08:00", "hora2f": "18:00",
                "hora3i": "08:00", "hora3f": "18:00",
                "hora4i": "08:00", "hora4f": "18:00",
                "hora5i": "08:00", "hora5f": "18:00",
                "hora6i": "00:00", "hora6f": "00:00",
                "hora7i": "00:00", "hora7f": "00:00"
            }
        ],
        "agendasetores": [
            {"Id_do_Setor": 1, "Setor": "CONSULTORIO 1"}
        ],
        "parametrosgerais": {
            "duracao": "0030",
            "horariode": "0800",
            "horarioate": "2000"
        }
    }"##;

    pub const APPOINTMENT_JSON: &str = r#"{
        "Id_Do_Agendamento": 999,
        "Id_do_Usuario": 42,
        "Id_da_Assinatura": 19,
        "Inicio": "2026-03-20T08:00:00",
        "Final": "2026-03-20T08:30:00",
        "Descricao": "CONSULTA DR JOAO",
        "Status": 1,
        "Vinculado_a": 12345,
        "Id_do_Diagnostico_QP": 0,
        "Chegada": "0001-01-01T00:00:00",
        "Atendido_As": "0001-01-01T00:00:00",
        "SMS": "62999998888"
    }"#;

    #[test]
    fn deserializa_agenda_parameters() {
        let p: AgendaParameters = serde_json::from_str(AGENDA_PARAMS_JSON).unwrap();
        assert_eq!(p.users.len(), 1);
        assert_eq!(p.users[0].id, 42);
        assert_eq!(p.users[0].username, "DR JOAO");
        assert!(p.users[0].has_agenda);
        assert_eq!(p.users[0].mon_start, "08:00");
        assert_eq!(p.users[0].sat_start, "00:00");
        assert_eq!(p.sectors.len(), 1);
        assert_eq!(p.sectors[0].id, 1);
        assert_eq!(p.sectors[0].name, "CONSULTORIO 1");
        assert_eq!(p.general.slot_duration, "0030");
        assert_eq!(p.general.start_time, "0800");
        assert_eq!(p.color_labels.label_for(0), Some("DESMARCADO"));
        assert_eq!(p.color_labels.label_for(1), Some("AGENDADO"));
        assert_eq!(p.color_labels.color_for(2), Some("#90EE90"));
        assert!(p.color_labels.label_for(99).is_none());
    }

    #[test]
    fn deserializa_appointment() {
        let a: Appointment = serde_json::from_str(APPOINTMENT_JSON).unwrap();
        assert_eq!(a.id, 999);
        assert_eq!(a.user_id, 42);
        assert_eq!(a.status, 1);
        assert_eq!(a.contact_id, Some(12345));
        assert_eq!(a.diagnostic_id, 0);
        assert!(!a.has_arrived());
        assert!(!a.is_attended());
    }

    #[test]
    fn appointment_has_arrived_sentinela() {
        let mut a: Appointment = serde_json::from_str(APPOINTMENT_JSON).unwrap();
        assert!(!a.has_arrived());
        a.arrived_at = "2026-03-20T09:05:00".to_string();
        assert!(a.has_arrived());
    }

    #[test]
    fn appointment_into_dto() {
        let a: Appointment = serde_json::from_str(APPOINTMENT_JSON).unwrap();
        let dto: AppointmentDto = a.into();
        assert_eq!(dto.id, 999);
        assert_eq!(dto.user_id, 42);
        assert_eq!(dto.contact_id, Some(12345));
        // arrived_at sentinela → None no DTO
        assert!(dto.arrived_at.is_none());
        assert_eq!(dto.repeat_weeks, 0);
    }

    #[test]
    fn appointment_dto_new_defaults() {
        let dto = AppointmentDto::new(42, "2026-03-20T08:00:00", "2026-03-20T08:30:00");
        assert_eq!(dto.id, 0);
        assert_eq!(dto.user_id, 42);
        assert_eq!(dto.start, "2026-03-20T08:00:00");
        assert_eq!(dto.status, 0);
        assert!(dto.contact_id.is_none());
        assert!(dto.other_professionals.is_empty());
    }

    #[test]
    fn serializa_appointment_dto_campos_api() {
        let dto = AppointmentDto::new(42, "2026-03-20T08:00:00", "2026-03-20T08:30:00");
        let json = serde_json::to_value(&dto).unwrap();
        assert!(json.get("Id_do_Agendamento").is_some());
        assert!(json.get("Id_do_Usuario").is_some());
        assert!(json.get("Inicio").is_some());
        assert!(json.get("Final").is_some());
        assert!(json.get("RepetirSemanas").is_some());
    }

    #[test]
    fn serializa_status_dto() {
        let dto = AppointmentStatusDto { id: 123, status: 2 };
        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["Id_do_Agendamento"], 123);
        assert_eq!(json["Status"], 2);
    }

    #[test]
    fn parse_insert_response_string_success() {
        assert!(parse_insert_response(&serde_json::json!("Success")));
        assert!(parse_insert_response(&serde_json::json!("success")));
        assert!(!parse_insert_response(&serde_json::json!("erro")));
    }

    #[test]
    fn parse_insert_response_array_direto() {
        let ok = serde_json::json!([{ "StatusAgendado": true, "iddousuario": 1 }]);
        assert!(parse_insert_response(&ok));
        let fail = serde_json::json!([{ "StatusAgendado": false }]);
        assert!(!parse_insert_response(&fail));
        assert!(!parse_insert_response(&serde_json::json!([])));
    }

    #[test]
    fn parse_insert_response_envelope_retorno() {
        let all_ok = serde_json::json!({
            "retorno": [
                { "StatusAgendado": true, "iddousuario": 1 },
                { "StatusAgendado": true, "iddousuario": 2 }
            ]
        });
        assert!(parse_insert_response(&all_ok));

        // Sucesso parcial → false (nem todas as ocorrências agendadas).
        let partial = serde_json::json!({
            "retorno": [
                { "StatusAgendado": true, "iddousuario": 1 },
                { "StatusAgendado": false, "iddousuario": 2 }
            ]
        });
        assert!(!parse_insert_response(&partial));
    }

    #[test]
    fn add_other_professional_acumula_ids_numericos() {
        let mut dto = AppointmentDto::new(10, "2026-03-20T08:00:00", "2026-03-20T08:30:00");
        assert!(dto.other_professionals.is_empty());
        dto.add_other_professional(42);
        dto.add_other_professional(99);
        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["OutrosProfissionais"], serde_json::json!([42, 99]));
    }
}
