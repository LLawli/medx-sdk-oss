//! O binário `medx-mcp` como processo filho, falando JSON-RPC cru pelo stdio.
//!
//! Toda linha que sai no stdout tem de ser JSON-RPC: o driver falha o teste
//! na primeira que não for. O stdin fica aberto até a resposta esperada
//! chegar, para o servidor não encerrar no meio de uma chamada.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(30);

pub struct StdioServer {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
    stderr: Arc<Mutex<String>>,
}

impl StdioServer {
    /// Sobe o binário com `MEDX_CONFIG_DIR` em `config_dir` e as variáveis de
    /// `env`; nenhuma outra variável `MEDX_*` do ambiente do teste vaza.
    pub fn start(config_dir: &Path, env: &[(&str, &str)]) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_medx-mcp"));
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("RUST_LOG", "debug");
        for (key, _) in std::env::vars() {
            if key.starts_with("MEDX_") {
                command.env_remove(key);
            }
        }
        command.env("MEDX_CONFIG_DIR", config_dir);
        for (key, value) in env {
            command.env(key, value);
        }
        let mut child = command.spawn().expect("o binário inicia");

        let stdout = child.stdout.take().expect("stdout");
        let (tx, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        let stderr = Arc::new(Mutex::new(String::new()));
        let mut stderr_pipe = child.stderr.take().expect("stderr");
        let sink = Arc::clone(&stderr);
        std::thread::spawn(move || {
            let mut buffer = String::new();
            let _ = stderr_pipe.read_to_string(&mut buffer);
            sink.lock().unwrap().push_str(&buffer);
        });

        let stdin = child.stdin.take();
        Self {
            child,
            stdin,
            lines,
            stderr,
        }
    }

    pub fn send(&mut self, message: &Value) {
        let stdin = self.stdin.as_mut().expect("stdin aberto");
        writeln!(stdin, "{message}").expect("escrever no stdin");
        stdin.flush().expect("flush do stdin");
    }

    /// Lê o stdout até a resposta com `id`. Toda linha lida tem de ser
    /// JSON-RPC.
    pub fn wait_response(&mut self, id: u64) -> Value {
        loop {
            let line = self.lines.recv_timeout(TIMEOUT).unwrap_or_else(|_| {
                panic!(
                    "sem resposta para o id {id}; stderr:\n{}",
                    self.stderr.lock().unwrap()
                )
            });
            let message = parse_json_rpc(&line);
            if message["id"] == id {
                return message;
            }
        }
    }

    pub fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        self.wait_response(id)
    }

    /// `initialize` seguido de `notifications/initialized`.
    pub fn initialize(&mut self) -> Value {
        let response = self.request(
            1,
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "teste", "version": "0" }
            }),
        );
        self.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        response
    }

    /// `tools/call`; devolve o `result` (que pode ter `isError`).
    pub fn call_tool(&mut self, id: u64, name: &str, arguments: Value) -> Value {
        let response = self.request(
            id,
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        );
        response
            .get("result")
            .cloned()
            .unwrap_or_else(|| panic!("tools/call {name} sem result: {response}"))
    }

    /// Fecha o stdin, espera o processo sair e confere que o resto do stdout
    /// também é JSON-RPC. Devolve o status e o stderr.
    pub fn finish(mut self) -> (ExitStatus, String) {
        drop(self.stdin.take());
        let (tx, rx) = mpsc::channel();
        let mut child = self.child;
        std::thread::spawn(move || {
            let _ = tx.send(child.wait());
        });
        let status = rx
            .recv_timeout(TIMEOUT)
            .expect("o binário deveria sair depois de o stdin fechar")
            .expect("status do binário");
        while let Ok(line) = self.lines.recv_timeout(Duration::from_millis(200)) {
            parse_json_rpc(&line);
        }
        // A thread do stderr termina quando o processo sai.
        std::thread::sleep(Duration::from_millis(100));
        let stderr = self.stderr.lock().unwrap().clone();
        (status, stderr)
    }
}

fn parse_json_rpc(line: &str) -> Value {
    let message: Value = serde_json::from_str(line)
        .unwrap_or_else(|err| panic!("linha do stdout que não é JSON ({err}): {line}"));
    assert_eq!(
        message["jsonrpc"], "2.0",
        "linha que não é JSON-RPC: {line}"
    );
    message
}

/// Texto de todos os blocos de texto de um `result` de `tools/call`.
pub fn result_text(result: &Value) -> String {
    result["content"]
        .as_array()
        .map(|blocks| {
            blocks
                .iter()
                .filter_map(|block| block["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}
