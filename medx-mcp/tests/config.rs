//! Leitura da configuração e escolha da conexão.

use medx::Session;
use medx_mcp::config::{Config, ConfigError, ConnectionPlan, Credentials, plan_connection};

fn config(vars: &[(&str, &str)]) -> Result<Config, ConfigError> {
    let vars: Vec<(String, String)> = vars
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    Config::from_vars(|name| vars.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone()))
}

fn credentials() -> Credentials {
    Credentials {
        email: "medico@example.invalid".to_owned(),
        password: "segredo".to_owned(),
    }
}

fn session(host: &str) -> Session {
    Session {
        token: "tok".to_owned(),
        email: "medico@example.invalid".to_owned(),
        db_id: "1".to_owned(),
        host: host.to_owned(),
    }
}

// ── Config::from_vars ─────────────────────────────────────────────────────────

#[test]
fn sem_variaveis_a_configuracao_e_a_padrao() {
    assert_eq!(config(&[]), Ok(Config::default()));
}

#[test]
fn base_url_e_normalizada_para_a_origem() {
    let cfg = config(&[("MEDX_BASE_URL", " https://clinica.example.invalid/api/ ")]).unwrap();
    assert_eq!(cfg.host.as_deref(), Some("https://clinica.example.invalid"));
}

#[test]
fn base_url_vazia_conta_como_ausente() {
    assert_eq!(config(&[("MEDX_BASE_URL", "   ")]).unwrap().host, None);
}

#[test]
fn base_url_sem_esquema_e_erro() {
    assert_eq!(
        config(&[("MEDX_BASE_URL", "clinica.example.invalid")]),
        Err(ConfigError::BaseUrl("clinica.example.invalid".to_owned()))
    );
}

#[test]
fn credenciais_exigem_as_duas_variaveis() {
    let cfg = config(&[
        ("MEDX_LOGIN_CREDENTIAL", "medico@example.invalid"),
        ("MEDX_PASSWORD_CREDENTIAL", "segredo"),
    ])
    .unwrap();
    assert_eq!(cfg.credentials, Some(credentials()));

    let so_email = config(&[("MEDX_LOGIN_CREDENTIAL", "medico@example.invalid")]).unwrap();
    assert_eq!(so_email.credentials, None);

    let senha_vazia = config(&[
        ("MEDX_LOGIN_CREDENTIAL", "medico@example.invalid"),
        ("MEDX_PASSWORD_CREDENTIAL", "  "),
    ])
    .unwrap();
    assert_eq!(senha_vazia.credentials, None);
}

#[test]
fn allow_write_aceita_so_valores_conhecidos() {
    for ligado in ["1", "true", "TRUE", " True "] {
        let cfg = config(&[("MEDX_MCP_ALLOW_WRITE", ligado)]).unwrap();
        assert!(cfg.allow_write, "{ligado:?} deveria ligar a escrita");
    }
    for desligado in ["0", "false", "FALSE", ""] {
        let cfg = config(&[("MEDX_MCP_ALLOW_WRITE", desligado)]).unwrap();
        assert!(
            !cfg.allow_write,
            "{desligado:?} deveria deixar a escrita desligada"
        );
    }
    assert_eq!(
        config(&[("MEDX_MCP_ALLOW_WRITE", "sim")]),
        Err(ConfigError::AllowWrite("sim".to_owned()))
    );
}

// ── plan_connection ───────────────────────────────────────────────────────────

#[test]
fn sessao_salva_reabre_no_host_em_que_o_token_foi_emitido() {
    let plan = plan_connection(
        &Config::default(),
        Some(session("https://a.example.invalid")),
    );
    let ConnectionPlan::Session {
        session,
        base_url,
        credentials,
    } = plan
    else {
        panic!("esperado Session, obtido {plan:?}");
    };
    assert_eq!(session.token, "tok");
    assert_eq!(base_url, "https://a.example.invalid/api");
    assert_eq!(credentials, None);
}

#[test]
fn medx_base_url_vence_o_host_da_sessao() {
    let cfg = Config {
        host: Some("https://b.example.invalid".to_owned()),
        credentials: Some(credentials()),
        allow_write: false,
    };
    let plan = plan_connection(&cfg, Some(session("https://a.example.invalid")));
    let ConnectionPlan::Session {
        base_url,
        credentials: creds,
        ..
    } = plan
    else {
        panic!("esperado Session, obtido {plan:?}");
    };
    assert_eq!(base_url, "https://b.example.invalid/api");
    assert_eq!(creds, Some(credentials()));
}

#[test]
fn sem_sessao_com_credenciais_faz_login_no_host_padrao() {
    let cfg = Config {
        credentials: Some(credentials()),
        ..Config::default()
    };
    let plan = plan_connection(&cfg, None);
    let ConnectionPlan::Login {
        host,
        credentials: creds,
    } = plan
    else {
        panic!("esperado Login, obtido {plan:?}");
    };
    assert_eq!(host, medx::client::DEFAULT_HOST);
    assert_eq!(creds, credentials());
}

#[test]
fn sem_sessao_com_credenciais_faz_login_no_host_configurado() {
    let cfg = Config {
        host: Some("https://b.example.invalid".to_owned()),
        credentials: Some(credentials()),
        allow_write: false,
    };
    let ConnectionPlan::Login { host, .. } = plan_connection(&cfg, None) else {
        panic!("esperado Login");
    };
    assert_eq!(host, "https://b.example.invalid");
}

#[test]
fn sem_sessao_e_sem_credenciais_nao_ha_conexao() {
    assert!(matches!(
        plan_connection(&Config::default(), None),
        ConnectionPlan::Missing
    ));
}
