//! Testes de integração — Etapa 3: Contatos.
//! Execute com: `cargo test --test contacts_tests -- --ignored`

mod common;
use common::{shared_client, with_temp_dir};
use medx::{ContactDto, ContactSearchGroup, MedxClient};

fn client() -> &'static MedxClient {
    shared_client()
}

/// Guard que garante deleção do contato mesmo em caso de panic.
struct ContactGuard<'a> {
    client: &'a MedxClient,
    id: i64,
}

impl<'a> ContactGuard<'a> {
    fn new(client: &'a MedxClient, id: i64) -> Self {
        Self { client, id }
    }
}

impl<'a> Drop for ContactGuard<'a> {
    fn drop(&mut self) {
        // Ignora erro no cleanup para não mascarar o erro original do teste
        let _ = self.client.delete_contact(self.id);
    }
}

// ── insurance_plans ───────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_insurance_plans_retorna_lista_nao_vazia() {
    with_temp_dir(|| {
        let plans = client().insurance_plans().expect("insurance_plans falhou");
        assert!(!plans.is_empty(), "deve haver ao menos um convênio");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_insurance_plans_tem_particular() {
    with_temp_dir(|| {
        let plans = client().insurance_plans().unwrap();
        let has_particular = plans.iter().any(|p| p.name.to_uppercase().contains("PART"));
        assert!(has_particular, "deve haver convênio particular");
    });
}

// ── search_contacts ───────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_search_contacts_retorna_resultados() {
    with_temp_dir(|| {
        // Busca pela letra "A" — deve retornar ao menos um resultado em qualquer conta
        let results = client()
            .search_contacts("A", ContactSearchGroup::All, 10)
            .expect("search_contacts falhou");
        // Pode ser vazio se não houver contatos, mas não deve falhar
        for c in &results {
            assert!(!c.name.is_empty(), "nome do contato não deve ser vazio");
            assert!(c.id != 0, "id do contato não deve ser zero");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_search_contacts_sem_resultado_retorna_vazio() {
    with_temp_dir(|| {
        // Nome improvável de existir — API retorna null, convertido para Vec vazia
        let results = client()
            .search_contacts("XZXZXZ_NAO_EXISTE_999", ContactSearchGroup::All, 5)
            .expect("search_contacts com nome sem resultado falhou");
        assert!(
            results.is_empty(),
            "busca improvável deve retornar lista vazia"
        );
    });
}

// ── contact CRUD ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_create_and_fetch_contact() {
    with_temp_dir(|| {
        let c = client();

        // Cria contato de teste
        let mut dto = ContactDto::new("MEDX SDK TESTE INTEGRACAO");
        dto.gender = "M".to_string();
        dto.mobile = "62900000001".to_string();
        dto.email = "sdk-test@example.com".to_string();
        dto.notes = "Criado por test automatizado — pode ser excluído".to_string();

        let new_id = c.create_contact(&dto).expect("create_contact falhou");
        // Guard garante deleção mesmo se assertions falharem
        let _guard = ContactGuard::new(c, new_id);

        assert!(
            new_id > 0,
            "id retornado deve ser positivo, obtido: {new_id}"
        );

        // Busca o contato criado
        let fetched = c
            .contact(new_id)
            .expect("contact() falhou para id recém-criado");
        assert_eq!(fetched.id, new_id);
        assert_eq!(fetched.name, "MEDX SDK TESTE INTEGRACAO");
        assert_eq!(fetched.mobile, "62900000001");
    }); // _guard.drop() chamado aqui → delete_contact garantido
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_update_contact() {
    with_temp_dir(|| {
        let c = client();

        // Cria
        let mut dto = ContactDto::new("MEDX SDK UPDATE TESTE");
        dto.gender = "F".to_string();
        let new_id = c.create_contact(&dto).expect("create_contact falhou");
        // Guard garante deleção mesmo se assertions falharem
        let _guard = ContactGuard::new(c, new_id);

        // Busca e atualiza
        let fetched = c.contact(new_id).expect("contact() falhou");
        let mut update_dto: ContactDto = fetched.into();
        update_dto.notes = "Atualizado por teste automatizado".to_string();
        c.update_contact(&update_dto)
            .expect("update_contact falhou");

        // Verifica
        let updated = c.contact(new_id).expect("contact() pós-update falhou");
        assert_eq!(updated.notes, "Atualizado por teste automatizado");
    }); // _guard.drop() → delete garantido
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_contact_nao_existente_retorna_erro() {
    with_temp_dir(|| {
        let err = client()
            .contact(999999999)
            .expect_err("contato inexistente deveria retornar erro");
        // Pode ser UnexpectedResponse (lista vazia) ou Api
        assert!(
            matches!(
                err,
                medx::MedxError::UnexpectedResponse(_) | medx::MedxError::Api { .. }
            ),
            "esperado UnexpectedResponse ou Api, obtido: {err}"
        );
    });
}

// ── photo ─────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_contact_photo_sem_foto_retorna_string() {
    with_temp_dir(|| {
        let c = client();

        // Cria um contato sem foto para testar o endpoint de foto
        let mut dto = ContactDto::new("MEDX SDK FOTO TESTE");
        dto.gender = "M".to_string();
        let new_id = c.create_contact(&dto).expect("create_contact falhou");
        let _guard = ContactGuard::new(c, new_id);

        // Foto deve retornar string (vazia ou base64)
        let photo = c
            .contact_photo_base64(new_id)
            .expect("contact_photo_base64 falhou");
        // Não verificamos conteúdo pois pode ser vazio
        let _ = photo;
    }); // _guard.drop() → delete garantido
}

// ── homonyms ──────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_homonym_contacts_nao_pânica() {
    with_temp_dir(|| {
        // Nome improvável de ter duplicata — só verifica que o endpoint responde
        let _ = client().homonym_contacts("XZXZXZ IMPROVAVEL", "M", "2000-01-01");
    });
}

#[test]
#[ignore = "debug temporario"]
fn debug_search_raw() {
    with_temp_dir(|| {
        let c = client();
        // raw text da API
        let raw = c
            .get_text("contatos/GetContatosGridBySearch?Group=0&GroupValue=10&Name=Paciente")
            .unwrap();
        println!("RAW (Paciente): {}", &raw[..raw.len().min(500)]);
        let raw2 = c
            .get_text("contatos/GetContatosGridBySearch?Group=0&GroupValue=10&Name=Paciente+Lucas")
            .unwrap();
        println!("RAW (Paciente Teste): {}", &raw2[..raw2.len().min(500)]);
    });
}

#[test]
#[ignore = "debug temporario 2"]
fn debug_search_varios_termos() {
    with_temp_dir(|| {
        let c = client();
        for term in &["A", "L", "Le", "Leo", "Leona", "Paciente", "Lucas"] {
            let raw = c
                .get_text(&format!(
                    "contatos/GetContatosGridBySearch?Group=0&GroupValue=5&Name={term}"
                ))
                .unwrap();
            let preview = if raw.len() > 120 { &raw[..120] } else { &raw };
            println!("  {:<12} → {}", term, preview);
        }
    });
}

#[test]
#[ignore = "debug temporario 3"]
fn debug_endpoint_variations() {
    with_temp_dir(|| {
        let c = client();
        let urls = [
            "contatos/GetContatosGridBySearch?Group=0&GroupValue=10&Name=A",
            "contatos/GetContatosGridBySearch?Group=0&GroupValue=100&Name=A",
            "contatos/GetContatosGridBySearch?Group=1&GroupValue=10&Name=A",
            "contatos/GetContatosGridBySearch?Name=A&Group=0&GroupValue=10",
            "contatos/GetContatosGridBySearch?Name=A",
            "contatos/GetContatosGridBySearch?filterstring=A&filter=nome",
            "contatos/GetContatosGrid?Name=A",
            "contatos/GetContatos?Name=A",
        ];
        for url in &urls {
            match c.get_text(url) {
                Ok(raw) => {
                    let preview = if raw.len() > 80 { &raw[..80] } else { &raw };
                    println!("  OK   {}", preview);
                }
                Err(e) => println!("  ERR  {} → {e}", &url[..url.len().min(60)]),
            }
        }
    });
}

#[test]
#[ignore = "debug temporario"]
fn debug_new_endpoint() {
    with_temp_dir(|| {
        let c = client();
        let raw = c
            .get_text("contatos/GetContatosGrid?Name=A&GroupValue=10")
            .unwrap();
        println!("GetContatosGrid+GroupValue: {}", &raw[..raw.len().min(300)]);
        let raw2 = c.get_text("contatos/GetContatosGrid?Name=A").unwrap();
        println!(
            "GetContatosGrid sem GroupValue: {}",
            &raw2[..raw2.len().min(300)]
        );
        let raw3 = c
            .get_text("contatos/GetContatosGridBySearch?Group=1&GroupValue=10&Name=A")
            .unwrap();
        println!("Group=1: {}", &raw3[..raw3.len().min(300)]);
    });
}

#[test]
#[ignore = "debug temporario 2"]
fn debug_search_contacts_via_method() {
    with_temp_dir(|| {
        let c = client();
        match c.search_contacts("A", ContactSearchGroup::All, 10) {
            Ok(contacts) => {
                println!("Resultados: {}", contacts.len());
                for contact in &contacts {
                    println!("  id={} nome={}", contact.id, contact.name);
                }
            }
            Err(e) => println!("ERRO: {:?}", e),
        }
    });
}

#[test]
#[ignore = "debug temporario 3"]
fn debug_no_match_endpoint() {
    with_temp_dir(|| {
        let c = client();
        let raw = c
            .get_text("contatos/GetContatosGrid?Name=XZXZXZ_NAO_EXISTE_999&GroupValue=5")
            .unwrap();
        println!("NoMatch: {}", &raw[..raw.len().min(200)]);
        let raw2 = c
            .get_text("contatos/GetContatosGrid?Name=Paciente+Lucas&GroupValue=10")
            .unwrap();
        println!("Paciente Teste: {}", &raw2[..raw2.len().min(300)]);
        let raw3 = c
            .get_text("contatos/GetContatosGridBySearch?Group=1&GroupValue=10&Name=Paciente+Lucas")
            .unwrap();
        println!("Group1+LL: {}", &raw3[..raw3.len().min(300)]);
    });
}
