//! Fixtures sintéticas no formato da API da MedX (nomes de campo da API).
//! Nenhum dado real: nomes, e-mails e documentos são inventados.

use serde_json::{Value, json};

pub fn current_user() -> Value {
    json!({
        "DbId": 4242,
        "Username": "DRA TESTE",
        "UserFullName": "DRA ANA TESTE",
        "UserId": 7,
        "LastLogin": "2026-10-01T12:00:00+00:00",
        "Plano": "FULL",
        "Classificacao": "GOLD",
        "BloqueioSMS": false,
        "BloqueioInsert": false,
        "BloqueioAcesso": false,
        "UserEmail": "medico@example.invalid",
        "RdStationKey": "chave-secreta-rd",
        "EstoqueAtualizado": "True",
        "isTrial": false
    })
}

/// Usuário da clínica (`Users/GetUsers`).
pub fn user(id: i64, nome: &str) -> Value {
    let mut user = json!({
        "CPF": "00000000000",
        "Id_da_Assinatura": 0,
        "Id_do_Usuario": id,
        "Usuario": nome,
        "Id_do_Setor": 1,
        "Ativo": true,
        "Agenda": true,
        "Senha": "",
        "Carimbo": "",
        "Cidade": "CIDADE TESTE",
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
        "Marketing": false,
        "Dashboard": true,
        "Qualidade": false,
        "Nome_Completo": format!("{nome} COMPLETO"),
        "Conselho_Profissional": "CRM",
        "Numero_do_Conselho": "00000",
        "UF_Conselho": "GO",
        "Relatorios_Financeiros": false,
        "Cripto": true,
        "Validade": "2026-12-31T00:00:00",
        "Ultima_senha": "",
        "Tentativas": 0,
        "Grupos": "",
        "E_Grupo": false,
        "Biometria": false,
        "Portal": false,
        "Faturamento_editar": false
    });
    for day in 1..=7 {
        user[format!("hora{day}i")] = json!("08:00");
        user[format!("hora{day}f")] = json!("18:00");
    }
    user
}

pub fn agenda_parameters() -> Value {
    json!({
        "colorandlabels": {
            "Colors": ["#A9A9A9", "#FFFFFF", "#90EE90"],
            "Labels": ["DESMARCADO", "AGENDADO", "COMPARECEU"]
        },
        "agendausuarios": [
            agenda_user(42, "DRA TESTE", true),
            agenda_user(43, "RECEPCAO", false)
        ],
        "agendasetores": [{ "Id_do_Setor": 1, "Setor": "CONSULTORIO 1" }],
        "parametrosgerais": { "duracao": "0030", "horariode": "0800", "horarioate": "2000" }
    })
}

fn agenda_user(id: i64, nome: &str, agenda: bool) -> Value {
    let mut user = json!({
        "Id_do_Usuario": id,
        "Usuario": nome,
        "Id_do_Setor": 1,
        "Agenda": agenda
    });
    for day in 1..=7 {
        user[format!("hora{day}i")] = json!("08:00");
        user[format!("hora{day}f")] = json!("18:00");
    }
    user
}

/// Agendamento; `paciente` 0 é um bloqueio de agenda.
pub fn appointment(id: i64, paciente: i64, descricao: &str) -> Value {
    json!({
        "Id_Do_Agendamento": id,
        "Id_do_Usuario": 42,
        "Id_da_Assinatura": 1,
        "Inicio": "2026-10-02T08:00:00",
        "Final": "2026-10-02T08:30:00",
        "Descricao": descricao,
        "Status": 1,
        "Vinculado_a": paciente,
        "Id_do_Diagnostico_QP": 0,
        "Chegada": "0001-01-01T00:00:00",
        "Atendido_As": "0001-01-01T00:00:00",
        "SMS": "00000000000"
    })
}

pub fn report(arquivo: &str) -> Value {
    json!({ "arquivo": arquivo, "mensagem": "" })
}

pub fn hoje_notificacao(id: i64, texto: &str) -> Value {
    json!({
        "IddoBoleto": id,
        "Notificacao": texto,
        "Status": "Aniversario",
        "IddoEvento": 0,
        "URLdoEvento": ""
    })
}

pub fn ultimo_atendido(id: i64, nome: &str) -> Value {
    json!({ "Nome": nome, "Id_do_Cliente": id, "Ultimo": "2026-10-01" })
}

pub fn nota(id: i64, texto: &str) -> Value {
    json!({ "Id": id, "Memo": texto, "Data": "2026-10-01T10:00:00", "IddoUsuario": 7 })
}
