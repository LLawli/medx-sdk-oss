//! Utilitários compartilhados entre os testes de integração.
//!
//! Os testes de integração são marcados com `#[ignore]` e só executam com:
//!
//!   cargo test -- --ignored
//!
//! Ou individualmente:
//!
//!   cargo test integration_ -- --ignored
//!
//! # Sessão compartilhada entre processos
//!
//! A API MedX permite apenas **uma sessão ativa por conta**. Para evitar que
//! múltiplos binários de teste se invalidem mutuamente, usamos:
//!
//! - Um diretório temp **fixo** (não baseado em PID) para a sessão
//! - Um arquivo de lock inter-processos para serializar o login
//! - `SHARED_CLIENT` com `OnceLock` para logar apenas uma vez por processo
//!
//! Todos os processos de teste compartilham o mesmo token salvo em disco.

use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use medx::MedxClient;

/// Serializa testes de integração DENTRO do mesmo processo.
pub static API_LOCK: Mutex<()> = Mutex::new(());

/// Diretório temp compartilhado entre todos os processos de teste.
/// Fixo (não baseado em PID) para que a sessão seja reutilizada entre binários.
static SHARED_TEMP: OnceLock<PathBuf> = OnceLock::new();

/// Cliente compartilhado — faz login uma só vez por processo.
static SHARED_CLIENT: OnceLock<MedxClient> = OnceLock::new();

/// Caminho do diretório compartilhado por TODOS os binários de teste.
fn shared_temp_dir() -> &'static PathBuf {
    SHARED_TEMP.get_or_init(|| {
        let tmp = std::env::temp_dir().join("medx-sdk-integration-tests");
        std::fs::create_dir_all(&tmp).expect("não foi possível criar temp dir");
        tmp
    })
}

/// Caminho do arquivo de lock inter-processos para serializar logins.
fn login_lock_path() -> PathBuf {
    std::env::temp_dir().join("medx-sdk-login.lock")
}

/// Guard que libera o lock inter-processos ao ser dropped.
struct LoginLock;

impl Drop for LoginLock {
    fn drop(&mut self) {
        std::fs::remove_file(login_lock_path()).ok();
    }
}

/// Adquire o lock inter-processos para login (spin-wait, timeout de 30s).
/// Retorna um guard que libera o lock no drop.
fn acquire_login_lock() -> LoginLock {
    let lock_path = login_lock_path();
    let deadline = Instant::now() + Duration::from_secs(30);

    loop {
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(_) => return LoginLock,
            Err(_) => {
                if Instant::now() > deadline {
                    // Se expirou, forçamos remoção de lock stale e tentamos de novo
                    std::fs::remove_file(&lock_path).ok();
                    continue;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

/// Retorna um cliente compartilhado.
///
/// A lógica de inicialização:
/// 1. Adquire o lock inter-processos (serializa logins entre binários)
/// 2. Tenta reutilizar sessão existente salva em disco (de outro binário)
/// 3. Se não houver sessão, faz login e salva o token
/// 4. Armazena o cliente em `OnceLock` (1 login por processo)
pub fn shared_client() -> &'static MedxClient {
    SHARED_CLIENT.get_or_init(|| {
        env::set_var("MEDX_CONFIG_DIR", shared_temp_dir());
        let (email, pass) = test_credentials();

        // Lock inter-processos: apenas UM processo faz login de cada vez
        let _lock = acquire_login_lock();

        // Tenta reutilizar sessão já existente (criada por outro binário)
        if let Some(session) = medx::load_session() {
            return MedxClient::from_session_with_credentials(session, email, pass);
        }

        // Sem sessão existente — faz login
        MedxClient::login(&email, &pass).expect("login inicial falhou")
    })
}

/// Executa `f` com `MEDX_CONFIG_DIR` apontando para o tempdir compartilhado,
/// garantindo que testes não colidam nem sobrescrevam a sessão real do usuário.
/// Também adquire `API_LOCK` para serializar chamadas com a API dentro do processo.
pub fn with_temp_dir<F: FnOnce()>(f: F) {
    let _api_guard = API_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    env::set_var("MEDX_CONFIG_DIR", shared_temp_dir());
    f();
}

/// Carrega as credenciais do arquivo `.env` ou de variáveis de ambiente.
/// Pânico se nenhuma fonte estiver disponível.
pub fn test_credentials() -> (String, String) {
    // 1. Tenta variáveis de ambiente diretas
    if let (Ok(email), Ok(pass)) = (
        env::var("MEDX_LOGIN_CREDENTIAL"),
        env::var("MEDX_PASSWORD_CREDENTIAL"),
    ) {
        if !email.is_empty() && !pass.is_empty() {
            return (email, pass);
        }
    }

    // 2. Tenta ler do .env na raiz do projeto
    let dotenv_path = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");
    if let Ok(content) = std::fs::read_to_string(dotenv_path) {
        let mut email = None;
        let mut pass = None;
        for line in content.lines() {
            if let Some(v) = line.strip_prefix("MEDX_LOGIN_CREDENTIAL=") {
                email = Some(v.trim().to_string());
            }
            if let Some(v) = line.strip_prefix("MEDX_PASSWORD_CREDENTIAL=") {
                pass = Some(v.trim().to_string());
            }
        }
        if let (Some(e), Some(p)) = (email, pass) {
            return (e, p);
        }
    }

    panic!(
        "Credenciais não encontradas. \
        Defina MEDX_LOGIN_CREDENTIAL e MEDX_PASSWORD_CREDENTIAL ou crie um arquivo .env."
    );
}

/// Valor de uma variável dos testes ao vivo: do ambiente ou, se faltar, do
/// arquivo `.env` na raiz do projeto (como as credenciais).
#[allow(dead_code)] // nem todo binário de teste usa
pub fn test_var(name: &str) -> Option<String> {
    if let Ok(v) = env::var(name) {
        if !v.trim().is_empty() {
            return Some(v.trim().to_string());
        }
    }
    let dotenv_path = concat!(env!("CARGO_MANIFEST_DIR"), "/.env");
    let content = std::fs::read_to_string(dotenv_path).ok()?;
    let prefix = format!("{name}=");
    content
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[allow(dead_code)] // nem todo binário de teste usa
fn required_test_var(name: &str, what: &str) -> String {
    test_var(name).unwrap_or_else(|| {
        panic!("Defina {name} ({what}) no ambiente ou no .env para rodar os testes ao vivo.")
    })
}

/// Id de um paciente da conta de teste, usado pelos testes ao vivo que leem
/// e restauram dados de um paciente (`MEDX_TEST_PATIENT_ID`).
#[allow(dead_code)] // nem todo binário de teste usa
pub fn test_patient_id() -> i64 {
    required_test_var("MEDX_TEST_PATIENT_ID", "id do paciente de teste")
        .parse()
        .expect("MEDX_TEST_PATIENT_ID deve ser um número")
}

/// Nome completo do mesmo paciente (`MEDX_TEST_PATIENT_NAME`).
#[allow(dead_code)] // nem todo binário de teste usa
pub fn test_patient_name() -> String {
    required_test_var("MEDX_TEST_PATIENT_NAME", "nome completo do paciente de teste")
}

/// `classe` de um arquivo do prontuário desse paciente
/// (`MEDX_TEST_FILE_CLASSE`, o campo `classe` de um registro com arquivo).
#[allow(dead_code)] // nem todo binário de teste usa
pub fn test_file_classe() -> String {
    required_test_var("MEDX_TEST_FILE_CLASSE", "classe de um arquivo do prontuário")
}
