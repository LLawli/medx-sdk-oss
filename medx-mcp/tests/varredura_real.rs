//! Varredura de leitura contra a MedX real. Ignorada por padrão: só o
//! usuário roda, de propósito, com a própria conta.
//!
//! ```bash
//! MEDX_LOGIN_CREDENTIAL=... MEDX_PASSWORD_CREDENTIAL=... \
//!   cargo test -p medx-mcp --test varredura_real -- --ignored --nocapture
//! ```
//!
//! Chama todas as ferramentas de leitura que não mexem em nada na MedX, com
//! os ids (profissional, paciente, convênio, formulário) tirados das próprias
//! respostas. Imprime só nome, status, tamanho e quantidade de itens: nenhum
//! dado devolvido sai daqui. Ficam de fora os relatórios (geram um PDF no
//! servidor), `mensagens_recebidas_chat` (pode marcar mensagens como
//! exibidas) e `registros_do_modulo` (não há como descobrir os nomes de
//! módulo; com um nome desconhecido a MedX responde 404).
//!
//! Usa o `session.json` de sempre (ou `MEDX_CONFIG_DIR`) e, se precisar, faz
//! login, o que derruba a sessão aberta no navegador com a mesma conta.

mod common;

use common::stdio::StdioServer;
use serde_json::{Value, json};

/// Ferramentas de leitura que a varredura não chama, e por quê.
const SKIPPED: [&str; 5] = [
    "mensagens_recebidas_chat",
    "registros_do_modulo",
    "relatorio_agenda",
    "relatorio_faltas",
    "relatorio_prontuario",
];

struct Sweep {
    server: StdioServer,
    next_id: u64,
    called: Vec<String>,
    failures: Vec<String>,
}

impl Sweep {
    /// Chama a ferramenta e imprime uma linha de resumo. Devolve o JSON do
    /// primeiro bloco de texto, quando há.
    fn call(&mut self, tool: &str, arguments: Value) -> Option<Value> {
        self.next_id += 1;
        let result = self.server.call_tool(self.next_id, tool, arguments);
        self.called.push(tool.to_owned());
        let blocks = result["content"].as_array().cloned().unwrap_or_default();
        let texts: Vec<&str> = blocks.iter().filter_map(|b| b["text"].as_str()).collect();
        let image = blocks.iter().find(|b| b["type"] == "image");
        let size: usize = texts.iter().map(|t| t.len()).sum::<usize>()
            + image.and_then(|i| i["data"].as_str()).map_or(0, str::len);
        let value: Option<Value> = texts.first().and_then(|t| serde_json::from_str(t).ok());

        let status = if result["isError"] == true {
            // O erro é da MedX ou do servidor, não traz dado de paciente.
            let message: String = texts.first().unwrap_or(&"").chars().take(160).collect();
            self.failures.push(format!("{tool}: {message}"));
            format!("ERRO: {message}")
        } else if let Some(image) = image {
            format!("ok, imagem {}", image["mimeType"].as_str().unwrap_or("?"))
        } else if let Some(Value::Array(items)) = &value {
            let cut = if texts.len() > 1 { " (cortado)" } else { "" };
            format!("ok, {} itens{cut}", items.len())
        } else if value.is_some() {
            "ok, objeto".to_owned()
        } else {
            "ok, texto".to_owned()
        };
        eprintln!("{tool:32} {status:40} {size:>7} bytes");
        value
    }
}

fn first_id(value: &Option<Value>, key: &str) -> Option<i64> {
    value.as_ref()?.as_array()?.first()?.get(key)?.as_i64()
}

#[test]
#[ignore = "fala com a MedX real; rode só de propósito"]
fn todas_as_leituras_na_medx_real() {
    let config_dir = std::env::var_os("MEDX_CONFIG_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(medx::session::config_dir);
    let mut env = Vec::new();
    for name in [
        "MEDX_LOGIN_CREDENTIAL",
        "MEDX_PASSWORD_CREDENTIAL",
        "MEDX_BASE_URL",
    ] {
        if let Ok(value) = std::env::var(name) {
            env.push((name, value));
        }
    }
    let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (*k, v.as_str())).collect();

    let mut server = StdioServer::start(&config_dir, &env);
    server.initialize();
    let tools = server.request(2, "tools/list", json!({}));
    let read_tools: Vec<String> = tools["result"]["tools"]
        .as_array()
        .expect("tools/list")
        .iter()
        .filter_map(|t| t["name"].as_str().map(str::to_owned))
        .collect();
    let mut sweep = Sweep {
        server,
        next_id: 2,
        called: Vec::new(),
        failures: Vec::new(),
    };

    let today = medx::util::current_datetime_str()[..10].to_owned();
    sweep.call("usuario_atual", json!({}));
    sweep.call("listar_usuarios", json!({ "limite": 5 }));
    sweep.call("parametros_agenda", json!({}));
    let professionals = sweep.call("listar_profissionais_agenda", json!({ "limite": 5 }));
    sweep.call("listar_setores_agenda", json!({}));
    if let Some(prof) = first_id(&professionals, "id") {
        let day = json!({ "profissional_id": prof, "data": today, "limite": 5 });
        sweep.call("agenda_do_dia", day.clone());
        sweep.call("bloqueios_do_dia", day);
    }
    sweep.call("notificacoes_de_hoje", json!({ "limite": 5 }));
    let recent = sweep.call("ultimos_atendidos", json!({ "limite": 5 }));
    sweep.call("listar_notas", json!({ "limite": 5 }));
    let patient = first_id(&recent, "patient_id").or_else(|| {
        let birthdays = sweep.call(
            "buscar_pacientes",
            json!({ "aniversariantes_do_mes": true, "limite": 5 }),
        );
        first_id(&birthdays, "id")
    });
    if let Some(patient) = patient {
        let record = sweep.call("ver_paciente", json!({ "paciente_id": patient }));
        if let Some(record) = record {
            let name = record["name"].as_str().unwrap_or("").trim().to_owned();
            if let Some(first_name) = name.split_whitespace().next() {
                sweep.call(
                    "buscar_pacientes",
                    json!({ "nome": first_name, "limite": 5 }),
                );
            }
            let gender = record["gender"]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_uppercase();
            let birth: String = record["birth_date"]
                .as_str()
                .unwrap_or("")
                .chars()
                .take(10)
                .collect();
            if !name.is_empty() && (gender == "M" || gender == "F") && birth.len() == 10 {
                sweep.call(
                    "pacientes_homonimos",
                    json!({ "nome": name, "sexo": gender, "nascimento": birth, "limite": 5 }),
                );
            }
        }
        let by_patient = json!({ "paciente_id": patient });
        sweep.call("foto_paciente", by_patient.clone());
        sweep.call("sumario_prontuario", by_patient);
        let records = sweep.call(
            "ver_prontuario",
            json!({ "paciente_id": patient, "limite": 20 }),
        );
        sweep.call(
            "galeria_de_fotos",
            json!({ "paciente_id": patient, "limite": 5 }),
        );
        sweep.call(
            "buscar_no_prontuario",
            json!({ "paciente_id": patient, "texto": "consulta", "limite": 5 }),
        );
        let file = records
            .as_ref()
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().find_map(|r| {
                    let has_file = r["tipo_doc"].as_str().is_some_and(|t| !t.is_empty());
                    r["classe"]
                        .as_str()
                        .filter(|c| has_file && !c.is_empty())
                        .map(str::to_owned)
                })
            });
        if let Some(classe) = file {
            sweep.call("link_arquivo_prontuario", json!({ "classe": classe }));
        }
        sweep.call(
            "atendimentos_do_paciente",
            json!({ "paciente_id": patient, "limite": 5 }),
        );
    }
    sweep.call("listar_planos_de_saude", json!({ "limite": 5 }));
    sweep.call("palavras_chave_prontuario", json!({}));
    let plans = sweep.call("listar_convenios", json!({ "limite": 5 }));
    if let Some(plan) = first_id(&plans, "id") {
        sweep.call(
            "procedimentos_do_convenio",
            json!({ "convenio_id": plan, "limite": 5 }),
        );
    }
    sweep.call("listar_procedimentos", json!({ "limite": 5 }));
    let forms = sweep.call("listar_formularios", json!({ "limite": 5 }));
    if let Some(form) = first_id(&forms, "id") {
        sweep.call("ver_formulario", json!({ "formulario_id": form }));
    }
    sweep.call("listar_unidades", json!({}));
    sweep.call("listar_atendimentos", json!({ "limite": 5 }));
    sweep.call(
        "listar_atendimentos",
        json!({ "filtro": "pendencias", "limite": 5 }),
    );
    sweep.call("listar_eventos", json!({ "limite": 5 }));
    sweep.call("listar_questionarios", json!({ "limite": 5 }));
    sweep.call("listar_diagnosticos_qp", json!({ "limite": 5 }));
    sweep.call("modelos_de_mensagem", json!({}));
    sweep.call("parametros_gerais", json!({}));
    sweep.call("parametros_de_cores", json!({}));
    sweep.call("listar_relatorios", json!({ "limite": 5 }));
    sweep.call("pastas_de_documentos", json!({ "limite": 5 }));
    let chat_users = sweep.call("listar_usuarios_chat", json!({ "limite": 5 }));
    if let Some(user) = first_id(&chat_users, "id") {
        sweep.call("historico_chat", json!({ "usuario_id": user, "limite": 5 }));
    }
    sweep.call("mensagens_nao_lidas_chat", json!({}));

    let not_called: Vec<&String> = read_tools
        .iter()
        .filter(|t| !sweep.called.contains(t) && !SKIPPED.contains(&t.as_str()))
        .collect();
    eprintln!("não chamadas (faltou dado para o id): {not_called:?}");
    eprintln!("puladas de propósito: {SKIPPED:?}");

    let (status, _) = sweep.server.finish();
    assert!(
        sweep.failures.is_empty(),
        "leituras com erro: {:#?}",
        sweep.failures
    );
    assert!(status.success());
}
