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
