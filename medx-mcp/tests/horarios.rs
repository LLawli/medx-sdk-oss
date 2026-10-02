//! Validação de data e hora (`AAAA-MM-DDTHH:MM`), antes de qualquer escrita.

use medx_mcp::tools::{ToolError, check_datetime, check_time_range};

#[test]
fn data_e_hora_sao_normalizadas_com_segundos() {
    assert_eq!(
        check_datetime("inicio", "2026-10-05T09:00").unwrap(),
        "2026-10-05T09:00:00"
    );
    assert_eq!(
        check_datetime("inicio", "2026-10-05T23:59:30").unwrap(),
        "2026-10-05T23:59:30"
    );
}

#[test]
fn data_e_hora_invalidas_sao_erro_que_cita_o_campo() {
    for value in [
        "2026-10-05",
        "2026-10-05 09:00",
        "2026-10-05T9:00",
        "2026-10-05T24:00",
        "2026-10-05T09:60",
        "2026-10-05T09:00:60",
        "2026-02-30T09:00",
        "2026-10-05T09:00Z",
        "2026-10-05T09:00:00-03:00",
    ] {
        let Err(ToolError::InvalidParams(message)) = check_datetime("fim", value) else {
            panic!("{value} deveria ser inválido");
        };
        assert!(message.contains("fim"), "{value}: {message}");
        assert!(message.contains("AAAA-MM-DDTHH:MM"), "{value}: {message}");
    }
}

#[test]
fn intervalo_exige_fim_depois_do_inicio() {
    assert_eq!(
        check_time_range("2026-10-05T09:00", "2026-10-05T09:30").unwrap(),
        (
            "2026-10-05T09:00:00".to_owned(),
            "2026-10-05T09:30:00".to_owned()
        )
    );
    for (inicio, fim) in [
        ("2026-10-05T09:30", "2026-10-05T09:00"),
        ("2026-10-05T09:00", "2026-10-05T09:00:00"),
    ] {
        let Err(ToolError::InvalidParams(message)) = check_time_range(inicio, fim) else {
            panic!("{inicio} -> {fim} deveria ser inválido");
        };
        assert!(message.contains("fim"), "{message}");
    }
}
