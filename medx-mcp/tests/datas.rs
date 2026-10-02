//! Validação de datas e períodos, antes de qualquer chamada à MedX.

use medx_mcp::tools::{ToolError, check_date, check_period};

#[test]
fn data_valida_passa() {
    for data in ["2026-10-02", "2024-02-29", "2026-01-31", "2026-12-01"] {
        assert!(check_date("data", data).is_ok(), "{data}");
    }
}

#[test]
fn data_fora_do_formato_e_erro_que_cita_o_campo() {
    for data in [
        "",
        "02/10/2026",
        "2026-1-02",
        "2026-10-2",
        "2026-13-01",
        "2026-00-10",
        "2026-10-32",
        "2026-10-00",
        "2026-02-30",
        "2026-04-31",
        "2025-02-29",
        "20261002",
        "2026-10-02T08:00",
        "abcd-ef-gh",
    ] {
        let err = check_date("inicio", data).expect_err(data);
        let ToolError::InvalidParams(message) = err else {
            panic!("{data}: esperado InvalidParams");
        };
        assert!(message.contains("inicio"), "{data}: {message}");
        assert!(message.contains("AAAA-MM-DD"), "{data}: {message}");
    }
}

#[test]
fn periodo_aceita_inicio_igual_ou_antes_do_fim() {
    assert!(check_period("2026-10-01", "2026-10-31").is_ok());
    assert!(check_period("2026-10-02", "2026-10-02").is_ok());
}

#[test]
fn periodo_com_fim_antes_do_inicio_e_erro() {
    let err = check_period("2026-10-31", "2026-10-01").expect_err("fim antes do início");
    let ToolError::InvalidParams(message) = err else {
        panic!("esperado InvalidParams");
    };
    assert!(message.contains("fim"), "{message}");
}

#[test]
fn periodo_valida_as_duas_datas() {
    assert!(matches!(
        check_period("2026/10/01", "2026-10-31"),
        Err(ToolError::InvalidParams(m)) if m.contains("inicio")
    ));
    assert!(matches!(
        check_period("2026-10-01", "31/10/2026"),
        Err(ToolError::InvalidParams(m)) if m.contains("fim")
    ));
}
