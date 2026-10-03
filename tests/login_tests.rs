//! Login sem escrever no terminal.
//!
//! A biblioteca não pode escrever no stdout: um programa que fala outro
//! protocolo por ele (um servidor MCP sobre stdio, por exemplo) teria o canal
//! corrompido no primeiro login e em cada re-login depois de um 401. As
//! mensagens de progresso viram etapas (`LoginStep`), e quem mostra é o
//! medx-cli.
//!
//! Sem rede externa: um servidor HTTP local imita as rotas de login da MedX,
//! com uma chave RSA sintética.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex, MutexGuard};

use medx::auth::{login_at_with_progress, LoginStep};

/// Chave pública RSA de 1024 bits gerada só para estes testes.
const PUBLIC_KEY_XML: &str = "<RSAKeyValue><Modulus>nQmVeUUOCy8WOa2rTuRl27GEYcBFLroA6W2BVHBmnlDrRemIQMCoN3pOHUiZZmo+3AZUFjjQj6yms4907FsJjVVcpX3i3Y1vpwVee68xzQk+pfQGSnbOBQksovif32EAMADRRkblFSnvN1pmCLxJUekjxrQX1vVFf1/+PzhHThM=</Modulus><Exponent>AQAB</Exponent></RSAKeyValue>";

const SESSAO_ATIVA_COM_TOKEN: &str = "usuário já logado, dispositivo: web, token  TOKEN_ANTIGO";
const SESSAO_ATIVA_SEM_TOKEN: &str = "usuário já logado";

/// Servidor de login fake. Com `sessao_ativa`, o primeiro `loginV3` responde
/// 400 com essa mensagem; os seguintes respondem 200.
struct FakeLogin {
    url: String,
    paths: Arc<Mutex<Vec<String>>>,
}

impl FakeLogin {
    fn start(sessao_ativa: Option<&'static str>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let paths = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&paths);
        std::thread::spawn(move || {
            let mut logins = 0;
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    continue;
                }
                let mut content_length = 0usize;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) <= 2 {
                        break;
                    }
                    let lower = line.to_ascii_lowercase();
                    if let Some(v) = lower.strip_prefix("content-length:") {
                        content_length = v.trim().parse().unwrap_or(0);
                    }
                }
                let mut body = vec![0u8; content_length];
                let _ = reader.read_exact(&mut body);

                let path = request_line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_string();
                seen.lock().unwrap().push(path.clone());

                let (status, body) = if path.starts_with("/api/LoginUnificado/VerificaEmailCripto")
                {
                    ("200 OK", "\"Ok:Success:SoftwareId:4242\"".to_string())
                } else if path == "/api/security/getkeys" {
                    let json = serde_json::json!({ "KeyId": "k1", "PublicKey": PUBLIC_KEY_XML });
                    ("200 OK", json.to_string())
                } else if path == "/api/LoginUnificado/loginV3" {
                    logins += 1;
                    match sessao_ativa {
                        Some(message) if logins == 1 => (
                            "400 Bad Request",
                            serde_json::json!({ "Message": message }).to_string(),
                        ),
                        _ => ("200 OK", "\"tok_novo\"".to_string()),
                    }
                } else if path.starts_with("/api/security/removetokeninuse") {
                    ("200 OK", String::new())
                } else {
                    ("404 Not Found", String::new())
                };
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                );
            }
        });
        FakeLogin { url, paths }
    }

    fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }
}

/// `MEDX_CONFIG_DIR` é do processo inteiro: os testes que fazem login o
/// apontam para um diretório próprio, um de cada vez.
static CONFIG_DIR_LOCK: Mutex<()> = Mutex::new(());

struct ConfigDir {
    path: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl ConfigDir {
    fn new(nome: &str) -> Self {
        let guard = CONFIG_DIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let path = std::env::temp_dir().join(format!("medx-login-{nome}-{}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        std::env::set_var("MEDX_CONFIG_DIR", &path);
        ConfigDir {
            path,
            _guard: guard,
        }
    }
}

impl Drop for ConfigDir {
    fn drop(&mut self) {
        std::env::remove_var("MEDX_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn etapas_do_login(fake: &FakeLogin) -> (medx::Session, Vec<LoginStep>) {
    let mut etapas = Vec::new();
    let session = login_at_with_progress(&fake.url, "a@example.invalid", "senha", &mut |etapa| {
        etapas.push(etapa)
    })
    .expect("o login no servidor fake deveria passar");
    (session, etapas)
}

// ── Etapas ────────────────────────────────────────────────────────────────────

#[test]
fn login_relata_as_etapas_em_ordem() {
    let dir = ConfigDir::new("etapas");
    let fake = FakeLogin::start(None);

    let (session, etapas) = etapas_do_login(&fake);

    assert_eq!(
        etapas,
        vec![
            LoginStep::VerifyingEmail,
            LoginStep::DbIdResolved("4242".to_string()),
            LoginStep::FetchingKey,
            LoginStep::Authenticating,
            LoginStep::Done,
        ]
    );
    assert_eq!(session.token, "tok_novo");
    assert_eq!(session.db_id, "4242");
    assert!(
        dir.path.join("session.json").exists(),
        "a sessão deveria ser salva"
    );
}

#[test]
fn login_com_sessao_ativa_relata_a_invalidacao() {
    let _dir = ConfigDir::new("sessao-ativa");
    let fake = FakeLogin::start(Some(SESSAO_ATIVA_COM_TOKEN));

    let (session, etapas) = etapas_do_login(&fake);

    assert_eq!(
        etapas,
        vec![
            LoginStep::VerifyingEmail,
            LoginStep::DbIdResolved("4242".to_string()),
            LoginStep::FetchingKey,
            LoginStep::Authenticating,
            LoginStep::ActiveSessionDetected,
            LoginStep::PreviousSessionRemoved,
            LoginStep::Done,
        ]
    );
    assert_eq!(session.token, "tok_novo");
    assert!(
        fake.paths()
            .iter()
            .any(|p| p == "/api/security/removetokeninuse?token=TOKEN_ANTIGO"),
        "deveria invalidar a sessão anterior; requests: {:?}",
        fake.paths()
    );
}

#[test]
fn login_com_sessao_ativa_sem_token_relata_e_tenta_de_novo() {
    let _dir = ConfigDir::new("sem-token");
    let fake = FakeLogin::start(Some(SESSAO_ATIVA_SEM_TOKEN));

    let (_, etapas) = etapas_do_login(&fake);

    assert_eq!(
        etapas[4..],
        [
            LoginStep::ActiveSessionDetected,
            LoginStep::OldTokenNotFound,
            LoginStep::Done,
        ]
    );
    assert!(!fake.paths().iter().any(|p| p.contains("removetokeninuse")));
}

// ── Nada no terminal ──────────────────────────────────────────────────────────

/// Variável que liga [`auxiliar_login_at_em_subprocesso`]: o host do fake.
const HOST_DO_AUXILIAR: &str = "MEDX_TESTE_LOGIN_HOST";

/// Roda `login_at` no fluxo com mais mensagens (sessão ativa invalidada).
/// Só faz algo quando chamado por [`login_at_nao_escreve_no_terminal`], que
/// o executa num subprocesso com `--nocapture` para ver o stdout de verdade.
#[test]
#[ignore = "auxiliar: executado em subprocesso por login_at_nao_escreve_no_terminal"]
fn auxiliar_login_at_em_subprocesso() {
    let Ok(host) = std::env::var(HOST_DO_AUXILIAR) else {
        return;
    };
    medx::auth::login_at(&host, "a@example.invalid", "senha").expect("login no fake");
}

#[test]
fn login_at_nao_escreve_no_terminal() {
    let fake = FakeLogin::start(Some(SESSAO_ATIVA_COM_TOKEN));
    let dir = std::env::temp_dir().join(format!("medx-login-silencio-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "auxiliar_login_at_em_subprocesso",
            "--exact",
            "--include-ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(HOST_DO_AUXILIAR, &fake.url)
        .env("MEDX_CONFIG_DIR", &dir)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "o login no subprocesso falhou:\n{stdout}\n{stderr}"
    );
    assert!(
        stdout.contains("1 passed"),
        "o auxiliar deveria ter rodado:\n{stdout}"
    );
    for trecho in [
        "Verificando",
        "dbId",
        "Buscando",
        "Autenticando",
        "Login realizado",
        "Sessão",
    ] {
        assert!(
            !stdout.contains(trecho),
            "login_at escreveu no stdout ({trecho}):\n{stdout}"
        );
        assert!(
            !stderr.contains(trecho),
            "login_at escreveu no stderr ({trecho}):\n{stderr}"
        );
    }
}

// ── medx-cli ──────────────────────────────────────────────────────────────────

/// O progresso que a biblioteca deixou de imprimir continua aparecendo no
/// `medx-cli auth login`.
#[test]
fn cli_auth_login_mostra_o_progresso() {
    let fake = FakeLogin::start(Some(SESSAO_ATIVA_COM_TOKEN));
    let dir = std::env::temp_dir().join(format!("medx-login-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_medx-cli"))
        .args(["auth", "login", "a@example.invalid", "senha"])
        .env("MEDX_BASE_URL", &fake.url)
        .env("MEDX_CONFIG_DIR", &dir)
        .env_remove("MEDX_LOGIN_CREDENTIAL")
        .env_remove("MEDX_PASSWORD_CREDENTIAL")
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "medx-cli auth login falhou:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for trecho in [
        "Verificando e-mail",
        "dbId: 4242",
        "Buscando chave pública RSA",
        "Autenticando",
        "Sessão ativa detectada",
        "Sessão anterior removida",
        "Login realizado",
    ] {
        assert!(
            stdout.contains(trecho),
            "faltou \"{trecho}\" no stdout:\n{stdout}"
        );
    }
}
