/// Persiste e lê a sessão ativa em ~/.config/medx-sdk/session.json
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::client::DEFAULT_HOST;
use crate::error::MedxError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub email: String,
    pub db_id: String,
    /// Host (origem, sem `/api`) em que o token foi emitido. Cada host do MedX
    /// tem sessão própria, então o cliente reabre a sessão nesse mesmo host.
    /// Ausente no `session.json` gravado antes deste campo: cai no host padrão.
    #[serde(default = "default_host")]
    pub host: String,
}

fn default_host() -> String {
    DEFAULT_HOST.to_string()
}

fn session_path() -> PathBuf {
    let base = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("medx-sdk");
    base.join("session.json")
}

pub fn save(session: &Session) -> Result<(), MedxError> {
    let path = session_path();
    fs::create_dir_all(path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(session)?;
    fs::write(&path, json)?;
    Ok(())
}

pub fn load() -> Option<Session> {
    let content = fs::read_to_string(session_path()).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn clear() -> Result<(), MedxError> {
    let path = session_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Serializa testes que alteram variáveis de ambiente (evita race condition).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_dir<F: FnOnce()>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!(
            "medx-sdk-unit-{}-{}",
            std::process::id(),
            // Sufixo único por thread para evitar colisão mesmo com lock
            std::thread::current().name().unwrap_or("t").replace("::", "_")
        ));
        fs::create_dir_all(&tmp).unwrap();
        env::set_var("XDG_CONFIG_HOME", &tmp);
        f();
        fs::remove_dir_all(&tmp).ok();
        env::remove_var("XDG_CONFIG_HOME");
    }

    fn sample() -> Session {
        Session {
            token: "tok_abc".to_string(),
            email: "user@test.com".to_string(),
            db_id: "db42".to_string(),
            host: "https://host.example.com".to_string(),
        }
    }

    #[test]
    fn save_e_load_roundtrip() {
        with_temp_dir(|| {
            let s = sample();
            save(&s).unwrap();
            let loaded = load().expect("load deve retornar Some após save");
            assert_eq!(loaded.token, s.token);
            assert_eq!(loaded.email, s.email);
            assert_eq!(loaded.db_id, s.db_id);
            assert_eq!(loaded.host, s.host);
        });
    }

    #[test]
    fn session_json_sem_host_carrega_com_host_padrao() {
        with_temp_dir(|| {
            let path = session_path();
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, r#"{"token":"tok","email":"a@b.com","db_id":"7"}"#).unwrap();
            let loaded = load().expect("session.json antigo deve continuar abrindo");
            assert_eq!(loaded.token, "tok");
            assert_eq!(loaded.host, DEFAULT_HOST);
        });
    }

    #[test]
    fn load_sem_arquivo_retorna_none() {
        with_temp_dir(|| {
            // Garante que não existe nada
            let _ = fs::remove_file(session_path());
            assert!(load().is_none());
        });
    }

    #[test]
    fn clear_remove_o_arquivo() {
        with_temp_dir(|| {
            save(&sample()).unwrap();
            assert!(session_path().exists());
            clear().unwrap();
            assert!(!session_path().exists());
        });
    }

    #[test]
    fn clear_sem_arquivo_nao_retorna_erro() {
        with_temp_dir(|| {
            assert!(clear().is_ok());
        });
    }

    #[test]
    fn save_cria_diretorios_intermediarios() {
        with_temp_dir(|| {
            // Remove tudo abaixo do tmpdir e verifica que save() recria
            let dir = session_path().parent().unwrap().to_path_buf();
            let _ = fs::remove_dir_all(&dir);
            assert!(save(&sample()).is_ok());
            assert!(session_path().exists());
        });
    }
}
