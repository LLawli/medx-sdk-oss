//! Testes de integração — Chat interno.
//! Execute com: `cargo test --test chat_tests -- --ignored`

mod common;
use common::{shared_client, with_temp_dir};
use medx::MedxClient;

fn client() -> &'static MedxClient { shared_client() }


// ── chat_users ────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_chat_users_nao_panica() {
    with_temp_dir(|| {
        let _ = client().chat_users();
    });
}

// ── chat_unread_count ─────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_chat_unread_count_nao_panica() {
    with_temp_dir(|| {
        let _ = client().chat_unread_count();
    });
}

// ── chat_history ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_chat_history_nao_panica() {
    with_temp_dir(|| {
        let c = client();
        let Ok(users) = c.chat_users() else { return };
        if users.is_empty() {
            return;
        }
        let _ = c.chat_history(users[0].id);
    });
}

// ── chat_incoming ─────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_chat_incoming_nao_panica() {
    with_temp_dir(|| {
        // Pode falhar com InvalidCredentials se testes CLI de auth rodam em paralelo
        let _ = client().chat_incoming();
    });
}

// ── send + mark_read ──────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_send_e_mark_read_nao_panica() {
    with_temp_dir(|| {
        let c = client();
        let Ok(user) = c.current_user() else { return };
        let Ok(users) = c.chat_users() else { return };

        // Busca um usuário diferente do atual para enviar mensagem
        let Some(recipient) = users.iter().find(|u| u.id != user.user_id) else { return };

        let now = ""; // data opcional — a API aceita string vazia
        let dto = medx::SendMessageDto::new(
            user.user_id,
            &user.username,
            recipient.id,
            &recipient.full_name,
            "MEDX SDK TESTE INTEGRACAO — pode ser ignorado",
            now,
        );
        let _ = c.send_chat_message(&dto);

        // Busca mensagens recebidas e marca como lidas
        let Ok(incoming) = c.chat_incoming() else { return };
        if !incoming.is_empty() {
            let ids: Vec<i64> = incoming.iter().map(|m| m.id).collect();
            let _ = c.mark_messages_read(&ids);
        }
    });
}
