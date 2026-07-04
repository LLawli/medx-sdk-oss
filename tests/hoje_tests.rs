//! Testes de integração — Etapa 9: Dashboard (Hoje).
//! Execute com: `cargo test --test hoje_tests -- --ignored`
//!
//! Endpoints cobertos:
//!   9.1  hoje_notificacoes()          → GET  hoje/GetHojeNotificacoes
//!   9.2  ultimos_atendidos()          → GET  hoje/GetUltimosAtendidos
//!   9.3  trial_info()                 → GET  adm/infosTrial
//!   9.4  notas()                      → GET  hoje/GetNotas
//!   9.5  insert_nota(&dto)            → POST hoje/InsertNota
//!   9.6  insert_nota_cliente(&dto)    → POST hoje/InsertNotaCliente
//!   9.7  update_nota(&dto)            → PUT  hoje/UpdateNota
//!   9.8  delete_nota(id)              → DELETE hoje/DeleteNotaById?Id=
//!   9.9  is_otp_or_expired()          → GET  hoje/IsOTPOrExpired
//!   9.10 sync_version()               → GET  SyncVersion/SyncData55To60

mod common;
use common::{shared_client, with_temp_dir};
use medx::MedxClient;

fn client() -> &'static MedxClient { shared_client() }



// ── hoje_notificacoes ─────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_hoje_notificacoes_nao_panica() {
    with_temp_dir(|| {
        let _ = client().hoje_notificacoes();
    });
}

// ── ultimos_atendidos ─────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_ultimos_atendidos_nao_panica() {
    with_temp_dir(|| {
        let _ = client().ultimos_atendidos();
    });
}

// ── trial_info ────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_trial_info_nao_panica() {
    with_temp_dir(|| {
        let _ = client().trial_info();
    });
}

// ── notas ─────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_notas_nao_panica() {
    with_temp_dir(|| {
        let _ = client().notas();
    });
}

// ── insert_nota + update_nota + delete_nota ───────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_nota_ciclo_completo() {
    with_temp_dir(|| {
        let c = client();
        let user = c.current_user().expect("current_user falhou");

        // Cria nota
        let dto = medx::InsertNotaDto::new(user.user_id, "MEDX SDK TESTE — pode ser excluída");
        let nota_id = c.insert_nota(&dto).expect("insert_nota falhou");
        // A API pode retornar 0 se não incluir o ID na resposta;
        // nesse caso encontramos o ID listando as notas criadas
        let real_id = if nota_id != 0 {
            nota_id
        } else {
            // Busca a nota pela lista de notas (a mais recente com texto de teste)
            c.notas()
                .unwrap_or_default()
                .iter()
                .find(|n| n.text.contains("MEDX SDK TESTE"))
                .map(|n| n.id)
                .unwrap_or(0)
        };

        if real_id != 0 {
            // Atualiza nota
            let upd = medx::UpdateNotaDto::new(real_id, user.user_id, "MEDX SDK TESTE — atualizada");
            c.update_nota(&upd).expect("update_nota falhou");

            // Exclui nota (cleanup)
            c.delete_nota(real_id).expect("delete_nota falhou");
        }
    });
}

// ── insert_nota_cliente ───────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_insert_nota_cliente_nao_panica() {
    with_temp_dir(|| {
        let c = client();
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let _user = c.current_user().expect("current_user falhou");
        // InsertNotaCliente é um fluxo de feedback: rating numérico + observação.
        let dto = medx::InsertNotaClienteDto::new(5, "MEDX SDK TESTE — pode ser ignorada");
        // Pode retornar erro dependendo de permissões — não deve panicar
        let _ = c.insert_nota_cliente(&dto);
    });
}

// ── is_otp_or_expired ─────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_is_otp_or_expired_nao_panica() {
    with_temp_dir(|| {
        let _ = client().is_otp_or_expired();
    });
}

// ── sync_version ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_sync_version_nao_panica() {
    with_temp_dir(|| {
        // Pode retornar erro se a clínica já está na versão mais recente
        let _ = client().sync_version();
    });
}

#[test]
#[ignore = "debug raw hoje"]
fn debug_hoje_notificacoes_raw() {
    with_temp_dir(|| {
        let c = client();
        let raw = c.get_text("hoje/GetHojeNotificacoes").unwrap_or_else(|e| format!("ERR: {:?}", e));
        println!("RAW: {}", &raw[..raw.len().min(500)]);
    });
}

#[test]
#[ignore = "debug raw ultimos"]
fn debug_ultimos_atendidos_raw() {
    with_temp_dir(|| {
        let c = client();
        let raw = c.get_text("hoje/GetUltimosAtendidos").unwrap_or_else(|e| format!("ERR: {:?}", e));
        println!("RAW: {}", &raw[..raw.len().min(500)]);
    });
}

#[test]
#[ignore = "debug raw notas"]
fn debug_notas_raw() {
    with_temp_dir(|| {
        let c = client();
        let raw = c.get_text("hoje/GetNotas").unwrap_or_else(|e| format!("ERR: {:?}", e));
        println!("RAW notas: {}", &raw[..raw.len().min(400)]);
    });
}

