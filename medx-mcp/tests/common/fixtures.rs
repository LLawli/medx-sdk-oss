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

/// Ficha completa de paciente (`contatos/GetContatosFichaById`, que devolve
/// uma lista com um item).
pub fn contact(id: i64, nome: &str) -> Value {
    // Montada a partir de texto: um `json!` deste tamanho passa do limite de
    // recursão de macro.
    let mut contact: Value = serde_json::from_str(CONTACT_TEMPLATE).expect("fixture válida");
    contact["Id_do_Cliente"] = json!(id);
    contact["Nome"] = json!(nome);
    contact
}

const CONTACT_TEMPLATE: &str = r#"{
        "Id_do_Cliente": 0,
        "Nome": "",
        "Nome_Social": "",
        "Sexo": "F",
        "Nascimento": "1990-05-15T00:00:00",
        "CPF_CGC": "00000000000",
        "RG": "0000000",
        "Email": "paciente@example.invalid",
        "Celular": "00000000000",
        "Telefone_Residencial": "",
        "Telefone_Residencial_1": "",
        "Telefone_Comercial": "",
        "Endereco_Residencial": "RUA TESTE, 1",
        "Bairro_Residencial": "CENTRO",
        "Cidade_Residencial": "CIDADE TESTE",
        "Estado_Residencial": "GO",
        "Cep_Residencial": "00000000",
        "Pais_Residencial": "BRASIL",
        "Endereco_Comercial": "",
        "Bairro_Comercial": "",
        "Cidade_Comercial": "",
        "Estado_Comercial": "",
        "Cep_Comercial": "",
        "Pais_Comercial": "",
        "Profissao": "",
        "Empresa": "",
        "Estado_Civil": "SOLTEIRO",
        "Tipo": "Paciente",
        "Observacoes": "",
        "Mae": "",
        "Pai": "",
        "Conjugue": "",
        "Acompanhante": "",
        "Contato": "",
        "Filhos": 0,
        "Id_do_Convenio": 1,
        "Numero_da_Matricula": "",
        "Mala_Direta": true,
        "VIP": false,
        "Exclui_Mkt": 0,
        "Tags": "",
        "Como_conheceu": null,
        "Indicado_por": "",
        "Escolaridade": "",
        "Religiao": "",
        "Regiao": "",
        "Co_Morbidade": "",
        "Fadiga": "",
        "Fumante": "",
        "Historico_Familiar_IAM_AVC_antes_50_anos": "",
        "Referencias": "",
        "PaginadaWeb": "",
        "Opcional1": null,
        "Opcional2": null,
        "LastEditDate": "2026-01-10T12:00:00",
        "CreationDate": "2020-03-01T09:30:00"
    }"#;

/// Linha da busca de pacientes (`contatos/GetContatosGridBySearch`).
pub fn contact_summary(id: i64, nome: &str) -> Value {
    json!({
        "Id_do_Cliente": id,
        "Nome": nome,
        "Nome_Social": null,
        "Celular": "00000000000",
        "Telefone_Residencial": null,
        "Email": "paciente@example.invalid",
        "CPF_CGC": null,
        "IddoConvenio": 1,
        "Convenio": "PARTICULAR",
        "total": 2
    })
}

pub fn insurance_plan(id: i64, nome: &str) -> Value {
    json!({ "Id_do_Convenio": id, "Convenio": nome })
}

pub fn homonym(id: i64, nome: &str) -> Value {
    json!({ "Id_do_Cliente": id, "Nome": nome, "Nascimento": "1990-05-15T00:00:00", "Sexo": "F" })
}

pub fn medical_summary() -> Value {
    json!({
        "diagnostico": "Diagnóstico de teste",
        "hpp": "Sem antecedentes",
        "medicamentos": "Nenhum",
        "alergias": "Nenhuma conhecida",
        "livre": ""
    })
}

/// Registro de prontuário; `data` no formato ISO (`2026-10-01T09:00:00`).
pub fn medical_record(id: i64, data: &str, texto: &str) -> Value {
    json!({
        "Id_do_Historico": id,
        "Id_da_Assinatura": 1,
        "Id_do_Cliente": 900,
        "Historico": format!("<p>{texto}</p>"),
        "Data": data,
        "Id_do_Usuario": 7,
        "Classe": "",
        "Palavraschave": "rotina",
        "Usuario": "DRA TESTE",
        "TipoDoc": "",
        "LastEditDate": data
    })
}

pub fn convenio(id: i64, nome: &str) -> Value {
    json!({ "IddoConvenio": id, "Convenio": nome, "Ativo": "true" })
}

pub fn procedure(id: i64, nome: &str) -> Value {
    json!({ "IddoProcedimento": id, "Procedimento": nome, "Comissao": 10.0, "PrecoBase": 150.0, "Sessoes": 1 })
}

pub fn convenio_procedure(id: i64, nome: &str) -> Value {
    json!({ "IddoProcedimento": id, "Procedimento": nome, "Valor": 200.0, "Sessoes": 1 })
}

pub fn form(id: i64, nome: &str) -> Value {
    json!({ "Id": id, "Arquivo": nome })
}

pub fn module_record(id: i64, modulo: &str) -> Value {
    json!({ "Id": id, "Modulo": modulo, "Dados": "{\"campo\":\"valor\"}", "Data": "2026-10-01T09:00:00", "Id_do_Cliente": 900 })
}

pub fn business_unit(id: i64, nome: &str) -> Value {
    json!({ "IddaUnidadedeNegocios": id, "UnidadedeNegocios": nome, "CPFCNPJ": "00000000000000", "Municipio": "CIDADE TESTE", "UF": "GO" })
}
