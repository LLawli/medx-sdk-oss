//! Smoke test contra a MedX real. Ignorado por padrão: só o usuário roda,
//! de propósito, com a própria conta.
//!
//! ```bash
//! MEDX_LOGIN_CREDENTIAL=... MEDX_PASSWORD_CREDENTIAL=... \
//!   cargo test -p medx-mcp --test smoke_real -- --ignored --nocapture
//! ```
//!
//! Usa o `session.json` de sempre (o mesmo do medx-cli) e, se precisar, faz
//! login com as credenciais, o que derruba a sessão aberta no navegador com a
//! mesma conta. Chama só ferramentas de leitura, com `limite` 1, e não imprime
//! nenhum dado devolvido: confere só que as respostas são JSON sem erro e que
//! o stdout do servidor só levou JSON-RPC.

mod common;

use common::stdio::{StdioServer, result_text};
use serde_json::{Value, json};

#[test]
#[ignore = "fala com a MedX real; rode só de propósito"]
fn leituras_basicas_na_medx_real() {
    let config_dir = medx::session::config_dir();
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

    let calls = [
        ("usuario_atual", json!({})),
        ("listar_usuarios", json!({ "limite": 1 })),
        ("parametros_agenda", json!({})),
        ("listar_procedimentos", json!({ "limite": 1 })),
    ];
    for (id, (tool, arguments)) in (2u64..).zip(calls) {
        let result = server.call_tool(id, tool, arguments);
        assert_ne!(
            result["isError"],
            true,
            "{tool} falhou: {}",
            result_text(&result)
        );
        let text = result_text(&result);
        let first = text.lines().next().unwrap_or_default();
        serde_json::from_str::<Value>(first)
            .unwrap_or_else(|err| panic!("{tool} não devolveu JSON ({err})"));
        eprintln!("{tool}: ok ({} bytes)", text.len());
    }

    let (status, _) = server.finish();
    assert!(status.success());
}
