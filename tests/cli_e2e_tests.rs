//! Testes end-to-end para o binário `medx-cli`.
//!
//! Testes sem rede executam sempre (sem `#[ignore]`).
//! Testes com rede requerem credenciais e são marcados com `#[ignore]`.
//!
//! Para executar apenas os testes sem rede:
//!   cargo test --test cli_e2e_tests
//!
//! Para executar todos (incluindo rede):
//!   cargo test --test cli_e2e_tests -- --include-ignored

mod common;

// ── Helpers gerais ─────────────────────────────────────────────────────────────

fn cli() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_medx-cli"))
}

fn assert_success(output: &std::process::Output) {
    if !output.status.success() {
        panic!(
            "comando falhou ({})\nstdout: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

// ── Helpers para testes de autenticação (sessão isolada) ───────────────────────

fn rand_u64() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    std::thread::current().id().hash(&mut h);
    h.finish()
}

/// Testes de auth (login/logout/session) usam uma sessão temporária isolada
/// para não interferir com a sessão compartilhada.
fn with_temp_session<F: FnOnce(&std::path::Path)>(f: F) {
    let _lock = common::API_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = std::env::temp_dir().join(format!("medx-e2e-{}", rand_u64()));
    std::fs::create_dir_all(&tmp).unwrap();
    f(&tmp);
    std::fs::remove_dir_all(&tmp).ok();
}

/// Executa login com `MEDX_CONFIG_DIR` apontando para `tmp` e retorna o output.
fn do_login(tmp: &std::path::Path, email: &str, pass: &str) -> std::process::Output {
    cli()
        .env("MEDX_CONFIG_DIR", tmp)
        .args(["auth", "login", email, pass])
        .output()
        .expect("falha ao executar medx-cli auth login")
}

/// Testes que não testam auth usam a sessão compartilhada.
/// Garante que a sessão existe e retorna o diretório compartilhado.
fn with_shared_session<F: FnOnce(&std::path::Path)>(f: F) {
    let _lock = common::API_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // Dispara o login compartilhado (no-op se já logado por outro binário)
    let _ = common::shared_client();
    let shared_dir = std::env::temp_dir().join("medx-sdk-integration-tests");
    // Garante credenciais em env vars para retry automático no subprocess CLI
    let (email, pass) = common::test_credentials();
    std::env::set_var("MEDX_LOGIN_CREDENTIAL", &email);
    std::env::set_var("MEDX_PASSWORD_CREDENTIAL", &pass);
    f(&shared_dir);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES SEM REDE — executam sempre, sem #[ignore]
// ═══════════════════════════════════════════════════════════════════════════════

/// Sem argumentos o CLI deve imprimir a ajuda global e sair com código 0.
#[test]
fn help_sem_args_imprime_recursos() {
    let output = cli().output().expect("falha ao executar medx-cli");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("auth") || out.contains("agenda"), "esperava subcomandos na ajuda\n{out}");
}

/// `medx-cli help` deve imprimir ajuda e sair com código 0.
#[test]
fn help_subcomando_imprime_ajuda() {
    let output = cli()
        .arg("help")
        .output()
        .expect("falha ao executar medx-cli help");
    assert_success(&output);
}

/// `medx-cli version` deve imprimir a versão e sair com código 0.
#[test]
fn version_imprime_versao() {
    let output = cli()
        .arg("version")
        .output()
        .expect("falha ao executar medx-cli version");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("medx-cli"), "esperava 'medx-cli' na saída de version\n{out}");
}

/// `medx-cli -v` deve funcionar igual a `medx-cli version`.
#[test]
fn version_flag_curta_imprime_versao() {
    let output = cli()
        .arg("-v")
        .output()
        .expect("falha ao executar medx-cli -v");
    assert_success(&output);
    let out = stdout(&output);
    assert!(out.contains("medx-cli"), "esperava 'medx-cli' em -v\n{out}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTES COM REDE — todos com #[ignore = "requer rede e credenciais válidas"]
// ═══════════════════════════════════════════════════════════════════════════════

/// Login com credenciais reais deve sair com código 0 e imprimir "Sessão salva".
#[test]
#[ignore = "requer rede e credenciais válidas"]
fn e2e_auth_login_sucesso() {
    let (email, pass) = common::test_credentials();
    with_temp_session(|tmp| {
        let output = do_login(tmp, &email, &pass);
        assert_success(&output);
        let out = stdout(&output);
        assert!(
            out.contains("Sessão salva"),
            "esperava 'Sessão salva' no stdout após login\n{out}"
        );
    });
}

/// Após login, `auth session` deve exibir o email e o db_id da sessão.
#[test]
#[ignore = "requer rede e credenciais válidas"]
fn e2e_auth_session_mostra_sessao() {
    let (email, pass) = common::test_credentials();
    with_temp_session(|tmp| {
        let login_output = do_login(tmp, &email, &pass);
        assert_success(&login_output);

        let output = cli()
            .env("MEDX_CONFIG_DIR", tmp)
            .args(["auth", "session"])
            .output()
            .expect("falha ao executar medx-cli auth session");
        assert_success(&output);
        let out = stdout(&output);
        assert!(
            out.contains(&email),
            "esperava o email '{email}' na saída de auth session\n{out}"
        );
        assert!(
            out.contains("db_id") || out.contains("db"),
            "esperava campo db_id na saída de auth session\n{out}"
        );
    });
}

/// login → logout → session deve mostrar "Nenhuma sessão".
#[test]
#[ignore = "requer rede e credenciais válidas"]
fn e2e_auth_logout_remove_sessao() {
    let (email, pass) = common::test_credentials();
    with_temp_session(|tmp| {
        let login_output = do_login(tmp, &email, &pass);
        assert_success(&login_output);

        let logout_output = cli()
            .env("MEDX_CONFIG_DIR", tmp)
            .args(["auth", "logout"])
            .output()
            .expect("falha ao executar medx-cli auth logout");
        assert_success(&logout_output);
        let logout_out = stdout(&logout_output);
        assert!(
            logout_out.contains("removida") || logout_out.contains("Sessão"),
            "esperava confirmação de logout\n{logout_out}"
        );

        let session_output = cli()
            .env("MEDX_CONFIG_DIR", tmp)
            .args(["auth", "session"])
            .output()
            .expect("falha ao executar medx-cli auth session após logout");
        assert_success(&session_output);
        let session_out = stdout(&session_output);
        assert!(
            session_out.to_lowercase().contains("nenhuma") || session_out.to_lowercase().contains("sessão"),
            "esperava mensagem de ausência de sessão após logout\n{session_out}"
        );
    });
}

/// contacts search A → código 0 (usa sessão compartilhada, sem login extra).
#[test]
#[ignore = "requer rede e credenciais válidas"]
fn e2e_contacts_search_retorna_resultados() {
    with_shared_session(|tmp| {
        let output = cli()
            .env("MEDX_CONFIG_DIR", tmp)
            .args(["contacts", "search", "A"])
            .output()
            .expect("falha ao executar medx-cli contacts search");
        assert_success(&output);
    });
}

/// agenda users → código 0 (usa sessão compartilhada, sem login extra).
#[test]
#[ignore = "requer rede e credenciais válidas"]
fn e2e_agenda_users_lista_profissionais() {
    with_shared_session(|tmp| {
        let output = cli()
            .env("MEDX_CONFIG_DIR", tmp)
            .args(["agenda", "users"])
            .output()
            .expect("falha ao executar medx-cli agenda users");
        assert_success(&output);
    });
}

/// hoje trial → código 0, sem panic (usa sessão compartilhada, sem login extra).
#[test]
#[ignore = "requer rede e credenciais válidas"]
fn e2e_hoje_trial_nao_panica() {
    with_shared_session(|tmp| {
        let output = cli()
            .env("MEDX_CONFIG_DIR", tmp)
            .args(["hoje", "trial"])
            .output()
            .expect("falha ao executar medx-cli hoje trial");
        assert_success(&output);
    });
}

/// chat users → código 0 (usa sessão compartilhada, sem login extra).
#[test]
#[ignore = "requer rede e credenciais válidas"]
fn e2e_chat_users_lista_usuarios() {
    with_shared_session(|tmp| {
        let output = cli()
            .env("MEDX_CONFIG_DIR", tmp)
            .args(["chat", "users"])
            .output()
            .expect("falha ao executar medx-cli chat users");
        assert_success(&output);
    });
}

/// Sem fazer login, contacts search deve sair com código != 0 e stderr com "sessão".
#[test]
#[ignore = "requer rede e credenciais válidas"]
fn e2e_comando_sem_sessao_imprime_erro() {
    with_temp_session(|tmp| {
        let output = cli()
            .env("MEDX_CONFIG_DIR", tmp)
            .args(["contacts", "search", "A"])
            .output()
            .expect("falha ao executar medx-cli contacts search sem sessão");
        assert!(
            !output.status.success(),
            "esperava código de saída != 0 sem sessão ativa\nstdout: {}\nstderr: {}",
            stdout(&output),
            stderr(&output)
        );
        let err = stderr(&output);
        assert!(
            err.to_lowercase().contains("sess") || err.to_lowercase().contains("login"),
            "esperava menção a 'sessão' ou 'login' em stderr\n{err}"
        );
    });
}

/// agenda params → código 0 (usa sessão compartilhada, sem login extra).
#[test]
#[ignore = "requer rede e credenciais válidas"]
fn e2e_agenda_params_retorna_parametros() {
    with_shared_session(|tmp| {
        let output = cli()
            .env("MEDX_CONFIG_DIR", tmp)
            .args(["agenda", "params"])
            .output()
            .expect("falha ao executar medx-cli agenda params");
        assert_success(&output);
    });
}
