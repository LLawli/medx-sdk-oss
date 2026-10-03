//! Testes de integração — Etapa 0: Autenticação.
//!
//! Requerem rede e credenciais válidas.
//! Execute com: `cargo test --test auth_tests -- --ignored`

mod common;

use common::with_temp_dir;
use medx::auth;
use medx::session;

// ── Login ──────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_login_retorna_token_nao_vazio() {
    with_temp_dir(|| {
        let (email, pass) = common::test_credentials();
        let session = auth::login(&email, &pass).expect("login falhou");
        assert!(!session.token.is_empty(), "token não deve ser vazio");
        assert_eq!(session.email, email);
        assert!(!session.db_id.is_empty(), "db_id não deve ser vazio");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_login_persiste_sessao_em_disco() {
    with_temp_dir(|| {
        let (email, pass) = common::test_credentials();
        auth::login(&email, &pass).expect("login falhou");

        let loaded = session::load().expect("sessão deve existir após login");
        assert_eq!(loaded.email, email);
        assert!(!loaded.token.is_empty());
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_login_senha_errada_retorna_invalid_credentials() {
    with_temp_dir(|| {
        let (email, _) = common::test_credentials();
        let err = auth::login(&email, "SenhaErrada1!")
            .expect_err("deveria falhar com credencial inválida");
        assert!(
            matches!(err, medx::MedxError::InvalidCredentials)
                || matches!(err, medx::MedxError::Api { .. }),
            "erro inesperado: {err}"
        );
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_login_invalida_sessao_ativa_e_reloga() {
    with_temp_dir(|| {
        let (email, pass) = common::test_credentials();

        // Primeiro login
        let s1 = auth::login(&email, &pass).expect("primeiro login falhou");

        // Segundo login — deve invalidar o primeiro automaticamente
        let s2 = auth::login(&email, &pass).expect("segundo login falhou");

        // Os tokens devem ser diferentes (sessão foi renovada)
        assert_ne!(s1.token, s2.token, "sessão deve ter sido renovada");

        // A sessão salva deve ser a nova
        let saved = session::load().unwrap();
        assert_eq!(saved.token, s2.token);
    });
}

// ── Session ────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_session_clear_e_reload() {
    with_temp_dir(|| {
        let (email, pass) = common::test_credentials();
        auth::login(&email, &pass).unwrap();

        assert!(session::load().is_some());
        session::clear().unwrap();
        assert!(session::load().is_none());
    });
}
