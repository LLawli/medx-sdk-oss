mod crypto;
mod error;
pub mod session;
pub mod util;

pub mod agenda;
pub mod ajustes;
pub mod auth;
pub mod chat;
pub mod client;
pub mod contacts;
pub mod financas;
pub mod hoje;
pub mod marketing;
pub mod notificacoes;
pub mod prontuario;
pub mod settings;
pub mod users;

pub use agenda::{
    AgendaColorLabels, AgendaGeneralParams, AgendaParameters, AgendaReportDto, AgendaSector,
    AgendaUser, Appointment, AppointmentDto, AppointmentStatusDto, NoShowReportDto, ReportResponse,
};
pub use ajustes::{DocFolder, IcsConfig, Relatorio};
pub use chat::{ChatMessage, ChatUser, SendMessageDto};
pub use client::MedxClient;
pub use contacts::{
    Contact, ContactDto, ContactSearchGroup, ContactSummary, HomonymContact, InsurancePlan,
};
pub use error::MedxError;
pub use financas::{Attendance, AttendanceDto, AttendanceProcedure, PrePaymentDto};
pub use hoje::{
    HojeNotificacao, InsertNotaClienteDto, InsertNotaDto, Nota, TrialInfo, UltimoAtendido,
    UpdateNotaDto,
};
pub use marketing::{DiagnosticoQP, Evento, InsertQuestDto, Quest, UpdateLocalAtendimentoDto};
pub use notificacoes::{ClienteSettings, MailLogDto};
pub use prontuario::{
    ArquivoDto, AttachFilesDto, BusinessUnit, Convenio, ConvenioProcedure, Form,
    MedicalHistorySummary, MedicalKeywords, MedicalRecord, MedicalRecordDto, ModuleRecord,
    Procedure, ProntuarioReportDto,
};
pub use session::{clear as clear_session, load as load_session, save as save_session, Session};
pub use settings::{ColorParameters, GeneralParameters};
pub use users::{CurrentUser, User};
