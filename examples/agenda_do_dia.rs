//! Agenda de hoje do usuário logado.
//!
//! ```bash
//! MEDX_LOGIN_CREDENTIAL=medico@clinica.com.br MEDX_PASSWORD_CREDENTIAL='senha' \
//!     cargo run --example agenda_do_dia
//! ```
//!
//! O login derruba a sessão aberta no navegador com a mesma conta.

use medx::{MedxClient, MedxError};

fn main() -> Result<(), MedxError> {
    let email = std::env::var("MEDX_LOGIN_CREDENTIAL").unwrap_or_default();
    let senha = std::env::var("MEDX_PASSWORD_CREDENTIAL").unwrap_or_default();

    let client = MedxClient::login(&email, &senha)?;
    let eu = client.current_user()?;
    let hoje = chrono::Local::now().format("%Y-%m-%d").to_string();

    println!("{}, {hoje}", eu.full_name);
    for consulta in client.daily_agenda(eu.user_id, &hoje)? {
        println!("{}  {}", consulta.start, consulta.description);
    }
    Ok(())
}
