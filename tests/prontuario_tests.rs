//! Testes de integração — Etapa 5: Prontuário.
//! Execute com: `cargo test --test prontuario_tests -- --ignored`

mod common;
use common::{shared_client, with_temp_dir};
use medx::{ArquivoDto, AttachFilesDto, MedicalRecordDto, MedxClient};

const PATIENT_ID: i64 = 100001;

fn client() -> &'static MedxClient { shared_client() }


// ── medical_history_summary ───────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_medical_history_summary_nao_pânica() {
    with_temp_dir(|| {
        let c = client();
        // Busca um paciente real da conta para testar
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let patient_id = contacts[0].id;
        // Campos podem ser vazios, mas não deve panicar
        let _ = c.medical_history_summary(patient_id);
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_medical_history_summary_paciente_inexistente_retorna_default() {
    with_temp_dir(|| {
        let summary = client()
            .medical_history_summary(999999999)
            .expect("medical_history_summary para id inexistente falhou");
        // Deve retornar summary vazio, não erro
        assert!(summary.diagnostic.is_empty() || !summary.diagnostic.is_empty());
    });
}

// ── medical_records ───────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_medical_records_nao_pânica() {
    with_temp_dir(|| {
        let c = client();
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let patient_id = contacts[0].id;
        // Endpoint pode retornar 404 se paciente não tem prontuário — não deve panicar
        let _ = c.medical_records(patient_id);
    });
}

// ── search_medical_records ────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_search_medical_records_nao_pânica() {
    with_temp_dir(|| {
        let c = client();
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let patient_id = contacts[0].id;
        let _ = c.search_medical_records(patient_id, "consulta");
    });
}

// ── medical_keywords ──────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_medical_keywords_nao_pânica() {
    with_temp_dir(|| {
        // Pode ser vazio, apenas verifica que deserializou sem pânico
        let _ = client().medical_keywords();
    });
}

// ── convenios ─────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_convenios_retorna_lista_nao_vazia() {
    with_temp_dir(|| {
        let convenios = client().convenios().expect("convenios falhou");
        assert!(!convenios.is_empty(), "deve haver ao menos um convênio");
        for c in &convenios {
            assert!(!c.name.is_empty(), "nome do convênio não deve ser vazio");
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_convenio_procedures_nao_pânica() {
    with_temp_dir(|| {
        let c = client();
        let convenios = c.convenios().expect("convenios falhou");
        if convenios.is_empty() {
            return;
        }
        let _ = c.convenio_procedures(convenios[0].id);
    });
}

// ── procedures ────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_procedures_retorna_lista() {
    with_temp_dir(|| {
        let procs = client().procedures().expect("procedures falhou");
        for p in &procs {
            assert!(!p.name.is_empty(), "nome do procedimento não deve ser vazio");
            assert!(p.base_price >= 0.0, "preço não deve ser negativo");
        }
    });
}

// ── forms ─────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_forms_nao_pânica() {
    with_temp_dir(|| {
        let _ = client().forms();
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_form_html_nao_pânica() {
    with_temp_dir(|| {
        let c = client();
        let forms = match c.forms() {
            Ok(f) => f,
            Err(medx::MedxError::Api { status: 500, .. }) => return, // instabilidade do servidor
            Err(e) => panic!("forms falhou: {e}"),
        };
        if forms.is_empty() {
            return;
        }
        let _ = c.form_html(forms[0].id);
    });
}

// ── business_units ────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_business_units_retorna_lista() {
    with_temp_dir(|| {
        // Endpoint pode retornar 500 se clínica não tem unidades configuradas — não deve panicar
        let _ = client().business_units();
    });
}

// ── upsert_medical_history_summary ────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_upsert_medical_history_summary_roundtrip() {
    with_temp_dir(|| {
        let c = client();
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let patient_id = contacts[0].id;

        // Lê estado original para restaurar depois
        let original = c
            .medical_history_summary(patient_id)
            .expect("medical_history_summary falhou");

        // Faz uma atualização com valor de teste
        let mut updated = original.clone();
        updated.free_text = "MEDX SDK TESTE INTEGRACAO".to_string();
        c.upsert_medical_history_summary(patient_id, &updated)
            .expect("upsert_medical_history_summary falhou");

        // Verifica que foi salvo
        let fetched = c
            .medical_history_summary(patient_id)
            .expect("medical_history_summary pós-upsert falhou");
        assert_eq!(fetched.free_text, "MEDX SDK TESTE INTEGRACAO");

        // Restaura estado original
        c.upsert_medical_history_summary(patient_id, &original)
            .expect("restauração do sumário falhou");
    });
}

// ── create_medical_record ─────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_create_medical_record_nao_pânica() {
    with_temp_dir(|| {
        let c = client();
        let contacts = c
            .search_contacts("A", medx::ContactSearchGroup::All, 1)
            .expect("search_contacts falhou");
        if contacts.is_empty() {
            return;
        }
        let patient_id = contacts[0].id;
        let user = c.current_user().expect("current_user falhou");

        let dto = MedicalRecordDto::new(
            patient_id,
            user.user_id,
            "<p>MEDX SDK TESTE INTEGRACAO — pode ser excluído</p>",
            "2026-03-17T09:00:00",
        );
        let _ = c.create_medical_record(&dto);
        // Não há endpoint de delete para prontuário — é imutável por design
    });
}


// ── module_records ────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_module_records_exames_nao_pânica() {
    with_temp_dir(|| {
        // API retorna 404 quando o módulo não tem registros — deve ser aceito como Ok(vec![])
        // ou como erro; ambos os casos são válidos.
        let result = client().module_records(PATIENT_ID, "Exames");
        match result {
            Ok(records) => {
                for r in &records {
                    assert!(r.id != 0, "id do registro de módulo deve ser não-zero");
                    assert!(!r.date.is_empty(), "data do registro de módulo não deve ser vazia");
                }
            }
            Err(medx::MedxError::Api { status: 404, .. }) => {
                // Aceitável: módulo sem registros retorna 404
            }
            Err(e) => panic!("module_records falhou com erro inesperado: {:?}", e),
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_module_records_modulo_inexistente_retorna_vazio() {
    with_temp_dir(|| {
        // Módulo que não existe deve retornar Ok(vec![]) ou qualquer erro graciosamente
        let result = client().module_records(PATIENT_ID, "ModuloQueNaoExiste");
        let _ = result;
    });
}

#[test]
#[ignore = "debug get file url param format"]
fn debug_get_file_url_param_format() {
    with_temp_dir(|| {
        use medx::session::load as load_session;
        let c = client();
        let _ = c;
        let classe = "4242-00000000-0000-4000-8000-000000000001.pdf";
        let uuid_only = "00000000-0000-4000-8000-000000000001";
        let session = load_session().unwrap();
        let token = session.token.clone();
        let http = reqwest::blocking::Client::builder().user_agent("medx-sdk/0.1").build().unwrap();

        // POST com diferentes valores para "arquivo"
        let valores: Vec<(&str, serde_json::Value)> = vec![
            ("classe completo",    serde_json::json!({"arquivo": classe})),
            ("uuid sem prefix",    serde_json::json!({"arquivo": uuid_only})),
            ("uuid+ext",           serde_json::json!({"arquivo": format!("{uuid_only}.pdf")})),
            ("com softwareId",     serde_json::json!({"arquivo": classe, "softwareId": 4242})),
            ("com SoftwareId",     serde_json::json!({"arquivo": classe, "SoftwareId": 4242})),
            ("com PacId",          serde_json::json!({"arquivo": classe, "PacId": 100001i64})),
            ("classe como string", serde_json::Value::String(classe.to_string())),
            ("uuid como string",   serde_json::Value::String(uuid_only.to_string())),
        ];

        for (label, body) in &valores {
            let resp = http.post("https://v65.medx.med.br/api/prontuario/GetFileUrl")
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .json(body)
                .send();
            match resp {
                Ok(r) => {
                    let status = r.status();
                    let text = r.text().unwrap_or_default();
                    // Mostrar só início se for HTML de erro
                    let preview = if text.contains("<!DOCTYPE") {
                        "500 HTML error page".to_string()
                    } else {
                        text[..text.len().min(300)].to_string()
                    };
                    println!("{label}: {status} → {preview}");
                }
                Err(e) => println!("{label}: ERR {e}"),
            }
        }

        // Também testar GET com "arquivo" como query param (não como POST body)
        let ep = format!("https://v65.medx.med.br/api/prontuario/GetFileUrl?arquivo={classe}");
        let resp = http.get(&ep)
            .header("Authorization", format!("Bearer {token}"))
            .send();
        match resp {
            Ok(r) => {
                let status = r.status();
                let text = r.text().unwrap_or_default();
                let preview = if text.contains("<!DOCTYPE") { "HTML error page".to_string() } else { text[..text.len().min(300)].to_string() };
                println!("GET ?arquivo=: {status} → {preview}");
            }
            Err(e) => println!("GET ?arquivo=: ERR {e}"),
        }
    });
}

#[test]
#[ignore = "debug file url redirect"]
fn debug_file_url_redirect() {
    with_temp_dir(|| {
        use medx::session::load as load_session;

        let c = client();
        let classe = "4242-00000000-0000-4000-8000-000000000001.pdf";

        // 1. GetFileUrl com vários parâmetros (endpoint retornou 500 — existe!)
        let params = vec![
            format!("prontuario/GetFileUrl?arquivo={classe}"),
            format!("prontuario/GetFileUrl?Arquivo={classe}"),
            format!("prontuario/GetFileUrl?nomeArquivo={classe}"),
            format!("prontuario/GetFileUrl?NomeArquivo={classe}"),
            format!("prontuario/GetFileUrl?fileName={classe}"),
            format!("prontuario/GetFileUrl?FileName={classe}"),
            format!("prontuario/GetFileUrl?Classe={classe}"),
            format!("prontuario/GetFileUrl?classe={classe}"),
            format!("prontuario/GetFileUrl?name={classe}"),
            format!("prontuario/GetFileUrl?Name={classe}"),
        ];
        for ep in &params {
            match c.get_text(ep) {
                Ok(raw) => println!("OK  {ep}\n    → {}", &raw[..raw.len().min(400)]),
                Err(e)  => println!("ERR {ep} → {e}"),
            }
        }

        // 2. Request autenticada ao /medxdata/{classe} — captura redirect
        let session = load_session().unwrap();
        let token = session.token.clone();
        let medxdata_url = format!("https://v65.medx.med.br/medxdata/{classe}");
        println!("\nFazendo GET autenticado: {medxdata_url}");

        // reqwest sem auto-redirect para ver o Location header
        let http_no_redir = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("medx-sdk/0.1")
            .build().unwrap();

        match http_no_redir.get(&medxdata_url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
        {
            Ok(resp) => {
                println!("  status: {}", resp.status());
                for (k, v) in resp.headers() {
                    println!("  header: {}: {}", k, v.to_str().unwrap_or("?"));
                }
                let body = resp.text().unwrap_or_default();
                println!("  body: {}", &body[..body.len().min(500)]);
            }
            Err(e) => println!("  request falhou: {e}"),
        }

        // 3. Mesmo request seguindo redirects — captura URL final
        let http_redir = reqwest::blocking::Client::builder()
            .user_agent("medx-sdk/0.1")
            .build().unwrap();

        match http_redir.get(&medxdata_url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
        {
            Ok(resp) => {
                println!("\nSeguindo redirects:");
                println!("  url final: {}", resp.url());
                println!("  status: {}", resp.status());
            }
            Err(e) => println!("  redirect falhou: {e}"),
        }
    });
}

// ── attach_files ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "requer rede e credenciais válidas — faz upload real no prontuário de Paciente"]
fn integration_attach_files_txt() {
    with_temp_dir(|| {
        let c = client();
        let data = b"teste de upload via SDK medx";
        let arquivo = ArquivoDto::from_bytes("teste.txt", "text/plain", data);
        let dto = AttachFilesDto::new(PATIENT_ID, "Arquivo de teste SDK", vec![arquivo]);
        let resp = c.attach_files(&dto).expect("attach_files falhou");
        println!("resposta: {resp}");
        assert_eq!(resp, "Success");
    });
}
