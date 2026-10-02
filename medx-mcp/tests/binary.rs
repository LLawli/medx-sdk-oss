//! O binário sobre stdio: o stdout leva só JSON-RPC, e configuração inválida
//! encerra antes de servir.

use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use serde_json::Value;

/// Roda o binário com `env` (sem herdar nenhuma variável `MEDX_*`), escreve
/// `stdin` e fecha a entrada; devolve a saída do processo.
fn run(env: &[(&str, &str)], stdin: &str) -> Output {
    let config_dir = tempfile::tempdir().expect("diretório temporário");
    let mut command = Command::new(env!("CARGO_BIN_EXE_medx-mcp"));
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("MEDX_CONFIG_DIR", config_dir.path())
        .env("RUST_LOG", "debug");
    for (key, _) in std::env::vars() {
        if key.starts_with("MEDX_") && key != "MEDX_CONFIG_DIR" {
            command.env_remove(key);
        }
    }
    for (key, value) in env {
        command.env(key, value);
    }
    let mut child = command.spawn().expect("o binário inicia");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("escrever no stdin");

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    rx.recv_timeout(Duration::from_secs(20))
        .expect("o binário deveria terminar depois de o stdin fechar")
        .expect("saída do binário")
}

const HANDSHAKE: &str = concat!(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"teste","version":"0"}}}"#,
    "\n",
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    "\n",
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    "\n",
);

/// Cada linha do stdout, como JSON-RPC. Falha se alguma linha não for.
fn json_rpc_lines(output: &Output) -> Vec<Value> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let value: Value = serde_json::from_str(line)
                .unwrap_or_else(|err| panic!("linha do stdout que não é JSON ({err}): {line}"));
            assert_eq!(value["jsonrpc"], "2.0", "linha que não é JSON-RPC: {line}");
            value
        })
        .collect()
}

fn response(lines: &[Value], id: u64) -> &Value {
    lines
        .iter()
        .find(|line| line["id"] == id)
        .unwrap_or_else(|| panic!("sem resposta para o id {id}: {lines:?}"))
}

#[test]
fn stdout_leva_so_json_rpc() {
    let output = run(&[], HANDSHAKE);
    let lines = json_rpc_lines(&output);

    let init = response(&lines, 1);
    assert_eq!(init["result"]["serverInfo"]["name"], "medx-mcp", "{init}");
    let tools = response(&lines, 2);
    assert!(tools["result"]["tools"].is_array(), "{tools}");
    assert!(
        output.status.success(),
        "o servidor deveria sair com sucesso quando o stdin fecha: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn logs_vao_para_o_stderr() {
    let output = run(&[], HANDSHAKE);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "o servidor falhou: {stderr}");
    let banner = format!("medx-mcp {}", env!("CARGO_PKG_VERSION"));
    assert!(
        stderr.contains(&banner),
        "o log de partida ({banner}) deveria ir para o stderr: {stderr}"
    );
}

#[test]
fn configuracao_invalida_encerra_antes_de_servir() {
    for (var, valor) in [
        ("MEDX_MCP_ALLOW_WRITE", "sim"),
        ("MEDX_BASE_URL", "clinica.example.invalid"),
    ] {
        let output = run(&[(var, valor)], HANDSHAKE);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "{var}={valor} deveria encerrar com erro"
        );
        assert!(
            output.stdout.is_empty(),
            "{var}={valor}: nada deveria ir para o stdout"
        );
        assert!(
            stderr.contains(var),
            "{var}={valor}: o erro deveria citar a variável: {stderr}"
        );
    }
}
