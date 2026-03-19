//! Testes de integração — Etapa 2: Usuários.
//! Execute com: `cargo test --test users_tests -- --ignored`

mod common;
use common::{shared_client, with_temp_dir};
use medx::MedxClient;

fn client() -> &'static MedxClient { shared_client() }


// ── current_user ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_current_user_retorna_email_correto() {
    with_temp_dir(|| {
        let (email, _) = common::test_credentials();
        let user = client().current_user().expect("current_user falhou");
        assert_eq!(user.email, email);
        assert!(!user.username.is_empty());
        assert!(!user.full_name.is_empty());
        assert!(user.db_id > 0);
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_current_user_plano_nao_vazio() {
    with_temp_dir(|| {
        let user = client().current_user().unwrap();
        assert!(!user.plan.is_empty(), "plano não deve ser vazio");
        assert!(!user.classification.is_empty(), "classificação não deve ser vazia");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_current_user_sem_bloqueios() {
    with_temp_dir(|| {
        let user = client().current_user().unwrap();
        // Conta de testes não deve estar bloqueada
        assert!(!user.access_blocked, "conta não deve estar com acesso bloqueado");
        assert!(!user.insert_blocked, "conta não deve estar com inserção bloqueada");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_current_user_last_login_e_iso8601() {
    with_temp_dir(|| {
        let user = client().current_user().unwrap();
        // Verifica que last_login tem formato de data ISO 8601 (começa com "20")
        assert!(
            user.last_login.starts_with("20"),
            "last_login deve ser ISO 8601, obtido: {}",
            user.last_login
        );
    });
}

// ── users ─────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_users_retorna_lista_nao_vazia() {
    with_temp_dir(|| {
        let users = client().users().expect("users falhou");
        assert!(!users.is_empty(), "deve haver ao menos um usuário cadastrado");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_users_tem_ao_menos_um_ativo() {
    with_temp_dir(|| {
        let users = client().users().unwrap();
        let ativos = users.iter().filter(|u| u.active).count();
        assert!(ativos > 0, "deve haver ao menos um usuário ativo");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_users_horarios_no_formato_correto() {
    with_temp_dir(|| {
        let users = client().users().unwrap();
        for user in users.iter().filter(|u| u.active) {
            // Horários devem estar no formato "HH:MM"
            for hora in [&user.schedule_mon_start, &user.schedule_mon_end] {
                assert_eq!(hora.len(), 5, "horário deve ter 5 chars (HH:MM), obtido: {hora}");
                assert!(hora.contains(':'), "horário deve conter ':', obtido: {hora}");
            }
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_users_ids_unicos() {
    with_temp_dir(|| {
        let users = client().users().unwrap();
        let mut ids: Vec<i64> = users.iter().map(|u| u.id).collect();
        let original_len = ids.len();
        ids.dedup();
        // Após dedup de lista ordenada — usamos set para verificar unicidade
        let unique: std::collections::HashSet<i64> = users.iter().map(|u| u.id).collect();
        assert_eq!(unique.len(), original_len, "todos os UserId devem ser únicos");
    });
}

// ── change_password (somente estrutura, não altera senha real) ─────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_change_password_senha_muito_longa_falha_antes_da_api() {
    with_temp_dir(|| {
        // RSA 512-bit + OAEP SHA-1: máximo = 64 - 2*20 - 2 = 22 bytes
        let senha_longa = "a".repeat(23);
        let c = client();
        let err = c
            .change_password(&senha_longa)
            .expect_err("senha > 22 chars deve falhar no RSA");
        assert!(
            matches!(err, medx::MedxError::Rsa(_)),
            "esperado MedxError::Rsa, obtido: {err}"
        );
    });
}
