//! Testes do host da API: o MedX alterna entre `v65` e `care-app65`, então o
//! SDK precisa levar o host escolhido do client até o login.
//!
//! Sem rede externa: um servidor HTTP local responde 401 em toda chamada da API
//! e 500 nas rotas de login, e registra o caminho de cada request recebida.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use medx::{MedxClient, Session};

// ── Servidor local ────────────────────────────────────────────────────────────

struct MockHost {
    url: String,
    paths: Arc<Mutex<Vec<String>>>,
}

impl MockHost {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let paths = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&paths);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    continue;
                }
                // Consome os headers; nenhuma request do teste leva body.
                let mut line = String::new();
                while reader.read_line(&mut line).map(|n| n > 2).unwrap_or(false) {
                    line.clear();
                }
                let path = request_line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_string();
                let status = if path.starts_with("/api/LoginUnificado/") {
                    "500 Internal Server Error"
                } else {
                    "401 Unauthorized"
                };
                seen.lock().unwrap().push(path);
                let _ = stream.write_all(
                    format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                        .as_bytes(),
                );
            }
        });
        MockHost { url, paths }
    }

    fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }
}

fn session_at(host: &str) -> Session {
    Session {
        token: "tok_expirado".to_string(),
        email: "host-test@example.invalid".to_string(),
        db_id: "1".to_string(),
        host: host.to_string(),
    }
}

// ── Client ────────────────────────────────────────────────────────────────────

#[test]
fn relogin_vai_para_o_host_do_client_e_nao_para_o_padrao() {
    let mock = MockHost::start();
    let client = MedxClient::from_session_with_credentials(
        session_at(&mock.url),
        "host-test@example.invalid".to_string(),
        "senha".to_string(),
    );

    // 401 → try_relogin → VerificaEmailCripto responde 500 e o login para ali.
    let err = client
        .get_text("security/getcurrentuser")
        .expect_err("o mock nunca autoriza");
    assert!(
        matches!(err, medx::MedxError::Api { status: 500, .. }),
        "esperado o 500 do login no mock, obtido: {err}"
    );

    let paths = mock.paths();
    assert_eq!(
        paths.first().map(String::as_str),
        Some("/api/security/getcurrentuser")
    );
    assert!(
        paths
            .iter()
            .any(|p| p.starts_with("/api/LoginUnificado/VerificaEmailCripto")),
        "o re-login deveria bater no mesmo host do client; requests no mock: {paths:?}"
    );
}

#[test]
fn login_at_usa_o_host_informado_mesmo_com_api_no_fim() {
    let mock = MockHost::start();
    let err = MedxClient::login_at(&format!("{}/api/", mock.url), "a@example.invalid", "x")
        .err()
        .expect("o mock recusa o login");
    assert!(
        matches!(err, medx::MedxError::Api { status: 500, .. }),
        "obtido: {err}"
    );
    assert_eq!(mock.paths().len(), 1);
    assert!(mock.paths()[0].starts_with("/api/LoginUnificado/VerificaEmailCripto?"));
}

// ── CLI ───────────────────────────────────────────────────────────────────────

/// Roda `medx-cli contacts search A` com uma sessão gravada em `host_da_sessao`.
fn cli_com_sessao(host_da_sessao: &str, medx_base_url: Option<&str>) -> std::process::Output {
    let tmp = std::env::temp_dir().join(format!(
        "medx-host-test-{}-{}",
        std::process::id(),
        host_da_sessao.rsplit(':').next().unwrap_or("x")
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(
        tmp.join("session.json"),
        serde_json::to_string(&session_at(host_da_sessao)).unwrap(),
    )
    .unwrap();

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_medx-cli"));
    cmd.args(["contacts", "search", "A"])
        .env("MEDX_CONFIG_DIR", &tmp)
        .env_remove("MEDX_LOGIN_CREDENTIAL")
        .env_remove("MEDX_PASSWORD_CREDENTIAL")
        .env_remove("MEDX_BASE_URL");
    if let Some(url) = medx_base_url {
        cmd.env("MEDX_BASE_URL", url);
    }
    let out = cmd.output().unwrap();
    std::fs::remove_dir_all(&tmp).ok();
    out
}

#[test]
fn cli_reabre_a_sessao_no_host_em_que_o_token_foi_emitido() {
    let mock = MockHost::start();
    let out = cli_com_sessao(&mock.url, None);
    assert!(!out.status.success(), "o mock responde 401");
    let paths = mock.paths();
    assert!(
        paths.iter().any(|p| p.starts_with("/api/")),
        "a CLI deveria chamar o host da sessão; requests: {paths:?}; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cli_medx_base_url_vence_o_host_da_sessao() {
    let mock = MockHost::start();
    // Sessão num host onde nada escuta: só passa se a CLI usar MEDX_BASE_URL.
    let out = cli_com_sessao("http://127.0.0.1:9", Some(&format!("{}/api", mock.url)));
    assert!(!out.status.success(), "o mock responde 401");
    let paths = mock.paths();
    assert!(
        paths.iter().any(|p| p.starts_with("/api/")),
        "MEDX_BASE_URL deveria mudar o host da CLI; requests: {paths:?}; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cli_recusa_medx_base_url_sem_esquema() {
    let mock = MockHost::start();
    let out = cli_com_sessao(&mock.url, Some("care-app65"));
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("MEDX_BASE_URL"));
    assert!(mock.paths().is_empty(), "não deveria chamar API nenhuma");
}

// ── Guarda: o host do MedX fica numa constante só ────────────────────────────

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn host_do_medx_aparece_so_na_constante_do_client() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    // Mínimo absoluto: se a varredura não achar os módulos, o guarda não vale nada.
    assert!(
        files.len() >= 15,
        "varredura achou só {} arquivos em src/",
        files.len()
    );

    let mut hits = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        for (i, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            if code.contains(".medx.med.br") {
                hits.push(format!(
                    "{}:{}",
                    file.strip_prefix(&src).unwrap().display(),
                    i + 1
                ));
            }
        }
    }
    assert_eq!(
        hits.len(),
        1,
        "o host do MedX deve existir só em client::default_host!; achado em: {hits:?}"
    );
    assert!(hits[0].starts_with("client.rs:"), "achado em: {hits:?}");
}
