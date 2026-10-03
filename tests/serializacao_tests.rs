//! Serialização dos tipos de resposta.
//!
//! Os tipos de resposta são lidos com os nomes da API MedX (`#[serde(rename =
//! "...")]`), mas ao serializar devem usar os nomes do SDK, isto é, os nomes
//! dos campos da struct, em snake_case. Assim quem usa o SDK (o medx-mcp, por
//! exemplo) devolve JSON com os mesmos nomes da documentação do crate, e não
//! os nomes crus e inconsistentes da API.
//!
//! As fixtures são sintéticas, no formato da API, sem dado real.

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

/// Desserializa `fixture` (formato da API), serializa de volta e confere que
/// as chaves de primeiro nível são exatamente `campos` e que nenhuma chave,
/// em nenhum nível, tem letra maiúscula.
fn confere<T: DeserializeOwned + Serialize>(fixture: &str, campos: &[&str]) {
    let valor: T =
        serde_json::from_str(fixture).expect("a fixture no formato da API deveria desserializar");
    let json = serde_json::to_value(&valor).expect("a serialização falhou");
    let objeto = json
        .as_object()
        .expect("deveria serializar como objeto JSON");

    let mut chaves: Vec<&str> = objeto.keys().map(String::as_str).collect();
    chaves.sort_unstable();
    let mut esperadas = campos.to_vec();
    esperadas.sort_unstable();
    assert_eq!(
        chaves, esperadas,
        "as chaves serializadas devem ser os nomes dos campos do SDK"
    );

    sem_maiusculas(&json, "$");
}

fn sem_maiusculas(valor: &Value, caminho: &str) {
    match valor {
        Value::Object(mapa) => {
            for (chave, filho) in mapa {
                assert!(
                    !chave.chars().any(char::is_uppercase),
                    "chave com nome da API em {caminho}: {chave}"
                );
                sem_maiusculas(filho, &format!("{caminho}.{chave}"));
            }
        }
        Value::Array(itens) => {
            for (i, item) in itens.iter().enumerate() {
                sem_maiusculas(item, &format!("{caminho}[{i}]"));
            }
        }
        _ => {}
    }
}

#[test]
fn agenda_color_labels_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Colors": [
        "texto"
    ],
    "Labels": [
        "texto"
    ]
}"#;
    confere::<medx::agenda::AgendaColorLabels>(fixture, &["colors", "labels"]);
}

#[test]
fn agenda_parameters_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "colorandlabels": {
        "Colors": [
            "texto"
        ],
        "Labels": [
            "texto"
        ]
    },
    "agendausuarios": [
        {
            "Id_do_Usuario": 7,
            "Usuario": "texto",
            "Id_do_Setor": 7,
            "Agenda": true,
            "hora1i": "texto",
            "hora1f": "texto",
            "hora2i": "texto",
            "hora2f": "texto",
            "hora3i": "texto",
            "hora3f": "texto",
            "hora4i": "texto",
            "hora4f": "texto",
            "hora5i": "texto",
            "hora5f": "texto",
            "hora6i": "texto",
            "hora6f": "texto",
            "hora7i": "texto",
            "hora7f": "texto"
        }
    ],
    "agendasetores": [
        {
            "Id_do_Setor": 7,
            "Setor": "texto"
        }
    ],
    "parametrosgerais": {
        "duracao": "texto",
        "horariode": "texto",
        "horarioate": "texto"
    }
}"#;
    confere::<medx::agenda::AgendaParameters>(
        fixture,
        &["color_labels", "users", "sectors", "general"],
    );
}

#[test]
fn agenda_general_params_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "duracao": "texto",
    "horariode": "texto",
    "horarioate": "texto"
}"#;
    confere::<medx::agenda::AgendaGeneralParams>(
        fixture,
        &["slot_duration", "start_time", "end_time"],
    );
}

#[test]
fn agenda_user_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id_do_Usuario": 7,
    "Usuario": "texto",
    "Id_do_Setor": 7,
    "Agenda": true,
    "hora1i": "texto",
    "hora1f": "texto",
    "hora2i": "texto",
    "hora2f": "texto",
    "hora3i": "texto",
    "hora3f": "texto",
    "hora4i": "texto",
    "hora4f": "texto",
    "hora5i": "texto",
    "hora5f": "texto",
    "hora6i": "texto",
    "hora6f": "texto",
    "hora7i": "texto",
    "hora7f": "texto"
}"#;
    confere::<medx::agenda::AgendaUser>(
        fixture,
        &[
            "id",
            "username",
            "sector_id",
            "has_agenda",
            "mon_start",
            "mon_end",
            "tue_start",
            "tue_end",
            "wed_start",
            "wed_end",
            "thu_start",
            "thu_end",
            "fri_start",
            "fri_end",
            "sat_start",
            "sat_end",
            "sun_start",
            "sun_end",
        ],
    );
}

#[test]
fn agenda_sector_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id_do_Setor": 7,
    "Setor": "texto"
}"#;
    confere::<medx::agenda::AgendaSector>(fixture, &["id", "name"]);
}

#[test]
fn appointment_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id_Do_Agendamento": 7,
    "Id_do_Usuario": 7,
    "Id_da_Assinatura": 7,
    "Inicio": "texto",
    "Final": "texto",
    "Descricao": "texto",
    "Status": 7,
    "Id_do_Procedimento": 7,
    "Vinculado_a": 7,
    "Id_do_Diagnostico_QP": 7,
    "Confirmacao": "texto",
    "Chegada": "texto",
    "Atendido_As": "texto",
    "SMS": "texto",
    "Saiu_as": "texto"
}"#;
    confere::<medx::agenda::Appointment>(
        fixture,
        &[
            "id",
            "user_id",
            "subscription_id",
            "start",
            "end",
            "description",
            "status",
            "procedure_id",
            "contact_id",
            "diagnostic_id",
            "confirmed_at",
            "arrived_at",
            "attended_at",
            "sms",
            "left_at",
        ],
    );
}

#[test]
fn report_response_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "arquivo": "texto",
    "mensagem": "texto"
}"#;
    confere::<medx::agenda::ReportResponse>(fixture, &["file_url", "message"]);
}

#[test]
fn appointment_created_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "StatusAgendado": true,
    "iddousuario": 7
}"#;
    confere::<medx::agenda::AppointmentCreated>(fixture, &["scheduled", "user_id"]);
}

#[test]
fn relatorio_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id": 7,
    "Nome": "texto",
    "Tipo": "texto"
}"#;
    confere::<medx::ajustes::Relatorio>(fixture, &["id", "name", "tipo"]);
}

#[test]
fn doc_folder_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id": 7,
    "Nome": "texto",
    "Filter": "texto"
}"#;
    confere::<medx::ajustes::DocFolder>(fixture, &["id", "name", "filter_key"]);
}

#[test]
fn ics_config_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Url": "texto",
    "Token": "texto",
    "Ativo": 1
}"#;
    confere::<medx::ajustes::IcsConfig>(fixture, &["url", "token", "active"]);
}

/// O estado do localizador vem da API como inteiro (`Ativo`) num campo
/// privado; serializado, sai como o booleano que `IcsConfig::active` devolve.
#[test]
fn ics_config_serializa_ativo_como_booleano() {
    for (ativo, esperado) in [(1, true), (0, false)] {
        let fixture = format!(r#"{{"Url": "u", "Token": "t", "Ativo": {ativo}}}"#);
        let cfg: medx::ajustes::IcsConfig = serde_json::from_str(&fixture).unwrap();
        let json = serde_json::to_value(&cfg).unwrap();
        assert_eq!(json["active"], Value::Bool(esperado), "Ativo = {ativo}");
    }
}

#[test]
fn chat_user_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "UserId": 7,
    "UserFullName": "texto",
    "UserName": "texto",
    "IsOnLine": true,
    "Total": 7
}"#;
    confere::<medx::chat::ChatUser>(
        fixture,
        &["id", "full_name", "username", "online", "unread"],
    );
}

#[test]
fn chat_message_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "MessageId": 7,
    "De": 7,
    "Para": 7,
    "MessageText": "texto",
    "MessageDate": "texto",
    "Lida": 7,
    "Exibida": 7,
    "strDe": "texto"
}"#;
    confere::<medx::chat::ChatMessage>(
        fixture,
        &[
            "id",
            "from_id",
            "to_id",
            "text",
            "date",
            "read",
            "shown",
            "from_name",
        ],
    );
}

#[test]
fn chat_count_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Total": 7
}"#;
    confere::<medx::chat::ChatCount>(fixture, &["total"]);
}

#[test]
fn contact_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id_do_Cliente": 7,
    "Nome": "texto",
    "Nome_Social": "texto",
    "Sexo": "texto",
    "Nascimento": "texto",
    "CPF_CGC": "texto",
    "RG": "texto",
    "Email": "texto",
    "Celular": "texto",
    "Telefone_Residencial": "texto",
    "Telefone_Residencial_1": "texto",
    "Telefone_Comercial": "texto",
    "Endereco_Residencial": "texto",
    "Bairro_Residencial": "texto",
    "Cidade_Residencial": "texto",
    "Estado_Residencial": "texto",
    "Cep_Residencial": "texto",
    "Pais_Residencial": "texto",
    "Endereco_Comercial": "texto",
    "Bairro_Comercial": "texto",
    "Cidade_Comercial": "texto",
    "Estado_Comercial": "texto",
    "Cep_Comercial": "texto",
    "Pais_Comercial": "texto",
    "Profissao": "texto",
    "Empresa": "texto",
    "Estado_Civil": "texto",
    "Tipo": "texto",
    "Observacoes": "texto",
    "Mae": "texto",
    "Pai": "texto",
    "Conjugue": "texto",
    "Acompanhante": "texto",
    "Contato": "texto",
    "Filhos": 7,
    "Id_do_Convenio": 7,
    "Numero_da_Matricula": "texto",
    "Mala_Direta": true,
    "VIP": true,
    "Exclui_Mkt": 7,
    "Tags": "texto",
    "Como_conheceu": "texto",
    "Indicado_por": "texto",
    "Escolaridade": "texto",
    "Religiao": "texto",
    "Regiao": "texto",
    "Co_Morbidade": "texto",
    "Fadiga": "texto",
    "Fumante": "texto",
    "Historico_Familiar_IAM_AVC_antes_50_anos": "texto",
    "Referencias": "texto",
    "PaginadaWeb": "texto",
    "Opcional1": "texto",
    "Opcional2": "texto",
    "LastEditDate": "texto",
    "CreationDate": "texto"
}"#;
    confere::<medx::contacts::Contact>(
        fixture,
        &[
            "id",
            "name",
            "social_name",
            "gender",
            "birth_date",
            "cpf",
            "rg",
            "email",
            "mobile",
            "phone_home",
            "phone_home_2",
            "phone_work",
            "address_home",
            "neighborhood_home",
            "city_home",
            "state_home",
            "zip_home",
            "country_home",
            "address_work",
            "neighborhood_work",
            "city_work",
            "state_work",
            "zip_work",
            "country_work",
            "profession",
            "company",
            "marital_status",
            "contact_type",
            "notes",
            "mother",
            "father",
            "spouse",
            "companion",
            "emergency_contact",
            "children_count",
            "insurance_id",
            "insurance_number",
            "mailing_list",
            "vip",
            "exclude_marketing",
            "tags",
            "how_found",
            "referred_by",
            "education",
            "religion",
            "region",
            "comorbidities",
            "fatigue",
            "smoker",
            "family_history_cardio",
            "references",
            "web_page",
            "optional_1",
            "optional_2",
            "last_edit_date",
            "creation_date",
        ],
    );
}

#[test]
fn insurance_plan_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id_do_Convenio": 7,
    "Convenio": "texto"
}"#;
    confere::<medx::contacts::InsurancePlan>(fixture, &["id", "name"]);
}

#[test]
fn contact_summary_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id_do_Cliente": 7,
    "Nome": "texto",
    "Nome_Social": "texto",
    "Celular": "texto",
    "Telefone_Residencial": "texto",
    "Email": "texto",
    "CPF_CGC": "texto",
    "IddoConvenio": 7,
    "Convenio": "texto",
    "total": 7
}"#;
    confere::<medx::contacts::ContactSummary>(
        fixture,
        &[
            "id",
            "name",
            "social_name",
            "mobile",
            "phone_home",
            "email",
            "cpf",
            "insurance_id",
            "insurance_name",
            "total",
        ],
    );
}

#[test]
fn homonym_contact_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id_do_Cliente": 7,
    "Nome": "texto",
    "Nascimento": "texto",
    "Sexo": "texto"
}"#;
    confere::<medx::contacts::HomonymContact>(fixture, &["id", "name", "birth_date", "gender"]);
}

#[test]
fn attendance_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Iddoatendimento": "texto",
    "Iddocliente": "texto",
    "NomeCliente": "texto",
    "Data": "texto",
    "IddoConvenio": "texto",
    "Valor_Fatura": 1.5,
    "TotPgto": 1.5,
    "Subtotal": 1.5,
    "Desc_real": 1.5,
    "Desc_perc": 1.5,
    "Desconto": 7,
    "Fechado": 7,
    "Orcamento": 7,
    "NF": "texto",
    "NumerodaGuia": "texto",
    "Recibo": "texto",
    "Filial": "texto",
    "Tabela": "texto",
    "Observacoes": "texto",
    "Iddousuario": 7,
    "IddaUnidadedeNegocios": "texto",
    "IddaAssinatura": 7
}"#;
    confere::<medx::financas::Attendance>(
        fixture,
        &[
            "id",
            "patient_id",
            "patient_name",
            "date",
            "convenio_id",
            "invoice_value",
            "total_paid",
            "subtotal",
            "discount_real",
            "discount_pct",
            "discount_status",
            "closed",
            "budget",
            "nf",
            "guide_number",
            "receipt",
            "branch",
            "table",
            "notes",
            "user_id",
            "business_unit_id",
            "subscription_id",
        ],
    );
}

#[test]
fn hoje_notificacao_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "IddoBoleto": 7,
    "Notificacao": "texto",
    "Status": "texto",
    "IddoEvento": 7,
    "URLdoEvento": "texto"
}"#;
    confere::<medx::hoje::HojeNotificacao>(
        fixture,
        &["id", "message", "tipo", "event_id", "event_url"],
    );
}

#[test]
fn ultimo_atendido_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Nome": "texto",
    "Id_do_Cliente": 7,
    "Ultimo": "texto"
}"#;
    confere::<medx::hoje::UltimoAtendido>(fixture, &["patient_name", "patient_id", "date"]);
}

#[test]
fn trial_info_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "isTrial": true,
    "vigencia": "texto",
    "celular": "texto",
    "conheceu": true
}"#;
    confere::<medx::hoje::TrialInfo>(fixture, &["is_trial", "vigencia", "celular", "conheceu"]);
}

#[test]
fn nota_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id": 7,
    "Memo": "texto",
    "Data": "texto",
    "IddoUsuario": 7
}"#;
    confere::<medx::hoje::Nota>(fixture, &["id", "text", "date", "user_id"]);
}

#[test]
fn evento_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "IddoBoleto": 7,
    "IddoEvento": 7,
    "Notificacao": "texto",
    "Status": "texto",
    "URLdoEvento": "texto"
}"#;
    confere::<medx::marketing::Evento>(fixture, &["id", "event_id", "name", "type_name", "url"]);
}

#[test]
fn quest_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id": 7,
    "Arquivo": "texto",
    "XML": "texto"
}"#;
    confere::<medx::marketing::Quest>(fixture, &["id", "name", "xml"]);
}

#[test]
fn diagnostico_qp_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "IddoDiagnosticoQP": 7,
    "StrDiagnosticoQP": "texto",
    "Tempo": "texto"
}"#;
    confere::<medx::marketing::DiagnosticoQP>(fixture, &["id", "name", "tempo"]);
}

#[test]
fn cliente_settings_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Codigo_do_Cliente": 7,
    "SoftwareId": 7,
    "Logotipo": "texto",
    "Instagram": "texto",
    "Facebook": "texto",
    "Website": "texto",
    "Texto_SMS": "texto",
    "Texto_Whatsapp": "texto",
    "Texto_PreCadastro": "texto",
    "Texto_Questionario": "texto"
}"#;
    confere::<medx::notificacoes::ClienteSettings>(
        fixture,
        &[
            "id",
            "software_id",
            "logo",
            "instagram",
            "facebook",
            "website",
            "sms_template",
            "whatsapp_template",
            "pre_registration_template",
            "quest_template",
        ],
    );
}

#[test]
fn medical_record_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id_do_Historico": 7,
    "Id_da_Assinatura": 7,
    "Id_do_Cliente": 7,
    "Historico": "texto",
    "Data": "texto",
    "Id_do_Usuario": 7,
    "Classe": "texto",
    "Palavraschave": "texto",
    "Usuario": "texto",
    "TipoDoc": "texto",
    "LastEditDate": "texto"
}"#;
    confere::<medx::prontuario::MedicalRecord>(
        fixture,
        &[
            "id",
            "subscription_id",
            "patient_id",
            "content",
            "date",
            "user_id",
            "classe",
            "keywords",
            "usuario",
            "tipo_doc",
            "last_edit_date",
        ],
    );
}

#[test]
fn medical_keywords_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Keywords": "texto"
}"#;
    confere::<medx::prontuario::MedicalKeywords>(fixture, &["raw"]);
}

#[test]
fn convenio_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "IddoConvenio": 7,
    "Convenio": "texto",
    "Ativo": "texto"
}"#;
    confere::<medx::prontuario::Convenio>(fixture, &["id", "name", "active"]);
}

#[test]
fn convenio_procedure_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "IddoProcedimento": 7,
    "Procedimento": "texto",
    "Valor": 1.5,
    "Sessoes": 7
}"#;
    confere::<medx::prontuario::ConvenioProcedure>(fixture, &["id", "name", "price", "sessions"]);
}

#[test]
fn procedure_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "IddoProcedimento": 7,
    "Procedimento": "texto",
    "Comissao": 1.5,
    "PrecoBase": 1.5,
    "Sessoes": 7
}"#;
    confere::<medx::prontuario::Procedure>(
        fixture,
        &["id", "name", "commission", "base_price", "sessions"],
    );
}

#[test]
fn form_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id": 7,
    "Arquivo": "texto"
}"#;
    confere::<medx::prontuario::Form>(fixture, &["id", "name"]);
}

#[test]
fn module_record_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id": 7,
    "Modulo": "texto",
    "Dados": "texto",
    "Data": "texto",
    "Id_do_Cliente": 7
}"#;
    confere::<medx::prontuario::ModuleRecord>(
        fixture,
        &["id", "module", "data", "date", "patient_id"],
    );
}

#[test]
fn business_unit_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "IddaUnidadedeNegocios": 7,
    "UnidadedeNegocios": "texto",
    "CPFCNPJ": "texto",
    "Municipio": "texto",
    "UF": "texto"
}"#;
    confere::<medx::prontuario::BusinessUnit>(
        fixture,
        &["id", "name", "cpf_cnpj", "city", "state"],
    );
}

#[test]
fn general_parameters_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "horariode": "texto",
    "horarioate": "texto",
    "horarioduracao": "texto",
    "smsclinica": "texto",
    "smspais": "texto",
    "smsddd": "texto",
    "smstel": "texto",
    "smsemail": "texto",
    "checksdiagnostico": "True",
    "checkscronometro": "True",
    "carboidratos": "texto",
    "proteinas": "texto",
    "gorduras": "texto",
    "fibras": "texto",
    "botaopref1": "texto"
}"#;
    confere::<medx::settings::GeneralParameters>(
        fixture,
        &[
            "business_hours_start",
            "business_hours_end",
            "slot_duration",
            "sms_clinic_name",
            "sms_country",
            "sms_area_code",
            "sms_phone",
            "sms_email",
            "diagnostics_enabled",
            "timer_enabled",
            "carbs_pct",
            "protein_pct",
            "fat_pct",
            "fiber_pct",
            "quick_buttons",
        ],
    );
}

#[test]
fn color_parameters_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Colors": [
        "texto"
    ],
    "Labels": [
        "texto"
    ]
}"#;
    confere::<medx::settings::ColorParameters>(fixture, &["colors", "labels"]);
}

#[test]
fn current_user_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "DbId": 7,
    "Username": "texto",
    "UserFullName": "texto",
    "UserId": 7,
    "LastLogin": "texto",
    "Plano": "texto",
    "Classificacao": "texto",
    "BloqueioSMS": true,
    "BloqueioInsert": true,
    "BloqueioAcesso": true,
    "UserEmail": "texto",
    "RdStationKey": "texto",
    "EstoqueAtualizado": "texto",
    "isTrial": true
}"#;
    confere::<medx::users::CurrentUser>(
        fixture,
        &[
            "db_id",
            "username",
            "full_name",
            "user_id",
            "last_login",
            "plan",
            "classification",
            "sms_blocked",
            "insert_blocked",
            "access_blocked",
            "email",
            "rd_station_key",
            "stock_updated_raw",
            "is_trial",
        ],
    );
}

#[test]
fn user_serializa_com_nomes_do_sdk() {
    let fixture = r#"{
    "Id_do_Usuario": 7,
    "Usuario": "texto",
    "Nome_Completo": "texto",
    "Ativo": true,
    "CPF": "texto",
    "Cidade": "texto",
    "Conselho_Profissional": "texto",
    "Numero_do_Conselho": "texto",
    "UF_Conselho": "texto",
    "Validade": "texto",
    "Id_da_Unidade_de_Negocios": 7,
    "Agenda": true,
    "Pacientes": true,
    "Prontuario": true,
    "Gerencia": true,
    "Estatistica": true,
    "Usuarios": true,
    "Configuracao": true,
    "Financeiro": true,
    "Marketing": true,
    "Dashboard": true,
    "Relatorios_Financeiros": true,
    "Atendimentos": true,
    "Prontuario_escreve": true,
    "hora1i": "texto",
    "hora1f": "texto",
    "hora2i": "texto",
    "hora2f": "texto",
    "hora3i": "texto",
    "hora3f": "texto",
    "hora4i": "texto",
    "hora4f": "texto",
    "hora5i": "texto",
    "hora5f": "texto",
    "hora6i": "texto",
    "hora6f": "texto",
    "hora7i": "texto",
    "hora7f": "texto"
}"#;
    confere::<medx::users::User>(
        fixture,
        &[
            "id",
            "username",
            "full_name",
            "active",
            "cpf",
            "city",
            "professional_council",
            "council_number",
            "council_uf",
            "validity",
            "business_unit_id",
            "perm_agenda",
            "perm_patients",
            "perm_medical_records",
            "perm_management",
            "perm_stats",
            "perm_users",
            "perm_config",
            "perm_financials",
            "perm_marketing",
            "perm_dashboard",
            "perm_financial_reports",
            "perm_appointments",
            "perm_medical_records_write",
            "schedule_mon_start",
            "schedule_mon_end",
            "schedule_tue_start",
            "schedule_tue_end",
            "schedule_wed_start",
            "schedule_wed_end",
            "schedule_thu_start",
            "schedule_thu_end",
            "schedule_fri_start",
            "schedule_fri_end",
            "schedule_sat_start",
            "schedule_sat_end",
            "schedule_sun_start",
            "schedule_sun_end",
        ],
    );
}
