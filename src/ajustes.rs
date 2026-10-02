//! Etapa 11 — Ajustes & Administração: relatórios disponíveis, pastas de
//! documentos e configuração do localizador ICS.
//!
//! Endpoints mapeados:
//! - `GET  report/listarelatorios`              — lista relatórios disponíveis
//! - `GET  autodocs/getfoldersdocs`             — lista pastas de documentos
//! - `GET  localizadorICS/GetLocalizadorICS`    — configuração do localizador ICS
//!
//! A troca de senha fica em `users::change_password` (`POST
//! usuarios/ChangeMePassword`); o endpoint não valida a senha atual, por isso
//! não há variante que a envie.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{client::MedxClient, error::MedxError};

// ── Deserializadores auxiliares ───────────────────────────────────────────────

fn de_null_str<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    use serde::de::Visitor;
    struct AnyToStr;
    impl<'de> Visitor<'de> for AnyToStr {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "string, número ou null")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_string<E: serde::de::Error>(self, v: String) -> Result<String, E> { Ok(v) }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<String, E> { Ok(v.to_string()) }
        fn visit_unit<E: serde::de::Error>(self) -> Result<String, E> { Ok(String::new()) }
        fn visit_none<E: serde::de::Error>(self) -> Result<String, E> { Ok(String::new()) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<String, D2::Error> {
            d.deserialize_any(AnyToStr)
        }
    }
    d.deserialize_any(AnyToStr)
}

fn de_null_i64<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    use serde::de::Visitor;
    struct I64OrStr;
    impl<'de> Visitor<'de> for I64OrStr {
        type Value = i64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "i64, string de número ou null")
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<i64, E> { Ok(v) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<i64, E> { Ok(v as i64) }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<i64, E> { Ok(v as i64) }
        fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<i64, E> { Ok(v as i64) }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<i64, E> {
            v.trim().parse::<i64>()
                .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<i64, E> { Ok(0) }
        fn visit_none<E: serde::de::Error>(self) -> Result<i64, E> { Ok(0) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<i64, D2::Error> {
            d.deserialize_any(I64OrStr)
        }
    }
    d.deserialize_any(I64OrStr)
}

#[allow(dead_code)]
fn de_null_f64<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    use serde::de::Visitor;
    struct F64OrStr;
    impl<'de> Visitor<'de> for F64OrStr {
        type Value = f64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "f64, string ou null")
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<f64, E> { Ok(v as f64) }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<f64, E> { Ok(v as f64) }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<f64, E> { Ok(v) }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<f64, E> {
            v.trim().replace(',', ".").parse::<f64>()
                .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<f64, E> { Ok(0.0) }
        fn visit_none<E: serde::de::Error>(self) -> Result<f64, E> { Ok(0.0) }
        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<f64, D2::Error> {
            d.deserialize_any(F64OrStr)
        }
    }
    d.deserialize_any(F64OrStr)
}

// ── Tipos ─────────────────────────────────────────────────────────────────────

/// Relatório disponível retornado por `GET report/listarelatorios`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relatorio {
    /// Identificador interno do relatório.
    #[serde(rename(deserialize = "Id"), deserialize_with = "de_null_i64")]
    pub id: i64,

    /// Nome descritivo do relatório.
    #[serde(default, rename(deserialize = "Nome"), deserialize_with = "de_null_str")]
    pub name: String,

    /// Tipo ou categoria do relatório.
    #[serde(default, rename(deserialize = "Tipo"), deserialize_with = "de_null_str")]
    pub tipo: String,
}

/// Pasta de documentos retornada por `GET autodocs/getfoldersdocs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocFolder {
    /// Identificador interno da pasta.
    #[serde(rename(deserialize = "Id"), deserialize_with = "de_null_i64")]
    pub id: i64,

    /// Nome descritivo da pasta.
    #[serde(default, rename(deserialize = "Nome"), deserialize_with = "de_null_str")]
    pub name: String,

    /// Chave de filtro usada para consultas relacionadas à pasta.
    #[serde(default, rename(deserialize = "Filter"), deserialize_with = "de_null_str")]
    pub filter_key: String,
}

/// Configuração do calendário ICS da agenda, montada por
/// [`MedxClient::ics_config`] a partir do localizador do feed.
///
/// `token` é o localizador e `url` o link do feed. O link abre sem
/// autenticação: quem o tem lê a agenda da clínica. Trate os dois como
/// credencial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcsConfig {
    /// URL do feed ICS.
    #[serde(default, rename(deserialize = "Url"), deserialize_with = "de_null_str")]
    pub url: String,

    /// Token de acesso ao feed ICS.
    #[serde(default, rename(deserialize = "Token"), deserialize_with = "de_null_str")]
    pub token: String,

    /// `true` quando `Ativo != 0`. Na serialização sai como `active`, booleano.
    #[serde(
        default,
        rename(serialize = "active", deserialize = "Ativo"),
        serialize_with = "ser_ativo_bool",
        deserialize_with = "de_null_i64"
    )]
    ativo_raw: i64,
}

/// Serializa o `Ativo` bruto da API como booleano (`!= 0`), igual a `active()`.
fn ser_ativo_bool<S: Serializer>(valor: &i64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_bool(*valor != 0)
}

impl IcsConfig {
    /// Retorna `true` se o localizador ICS está ativo (`Ativo != 0`).
    pub fn active(&self) -> bool {
        self.ativo_raw != 0
    }
}

// ── Helpers internos ──────────────────────────────────────────────────────────

/// Converte um `serde_json::Value` em `Vec<T>`.
///
/// - Se o valor for um array JSON, desserializa cada elemento em `T`,
///   ignorando silenciosamente os que falham.
/// - Para qualquer outro valor (objeto, null, string…) retorna `Vec::new()`.
fn parse_vec<T>(value: serde_json::Value) -> Vec<T>
where
    T: for<'de> Deserialize<'de>,
{
    match value {
        serde_json::Value::Array(arr) => arr
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect(),
        _ => Vec::new(),
    }
}

// ── Métodos do cliente ────────────────────────────────────────────────────────

impl MedxClient {
    /// Lista os relatórios disponíveis na clínica autenticada.
    ///
    /// `GET report/listarelatorios`
    pub fn list_reports(&self) -> Result<Vec<Relatorio>, MedxError> {
        let raw: serde_json::Value = self.get("report/listarelatorios")?;
        Ok(parse_vec(raw))
    }

    /// Lista as pastas de documentos, com filtro opcional.
    ///
    /// Passe `""` para retornar todas as pastas sem filtro.
    ///
    /// `GET autodocs/getfoldersdocs?filter={filter}`
    pub fn doc_folders(&self, filter: &str) -> Result<Vec<DocFolder>, MedxError> {
        let endpoint = format!("autodocs/getfoldersdocs?filter={filter}");
        let raw: serde_json::Value = self.get(&endpoint)?;
        Ok(parse_vec(raw))
    }

    /// Localizador do feed ICS da clínica (`GET ICS/GetLocalizador`, o
    /// endpoint do webapp). A API devolve uma string JSON; vazia (ou `null`)
    /// quando não há feed.
    pub fn ics_locator(&self) -> Result<String, MedxError> {
        let body = self.get_text("ICS/GetLocalizador")?;
        let body = body.trim();
        // A API devolve uma string JSON; `null` e corpo vazio querem dizer
        // que não há feed. Qualquer outro corpo vale como veio.
        let locator = if body.is_empty() || body == "null" {
            String::new()
        } else {
            serde_json::from_str::<String>(body).unwrap_or_else(|_| body.to_string())
        };
        Ok(locator.trim().to_string())
    }

    /// Configuração do calendário ICS: o localizador e o link do feed,
    /// `<base_url>/ics/getics?id=<localizador>`. Sem localizador, tudo vazio
    /// e `active()` falso.
    pub fn ics_config(&self) -> Result<IcsConfig, MedxError> {
        let locator = self.ics_locator()?;
        if locator.is_empty() {
            return Ok(IcsConfig {
                url: String::new(),
                token: String::new(),
                ativo_raw: 0,
            });
        }
        Ok(IcsConfig {
            url: format!("{}/ics/getics?id={locator}", self.base_url.trim_end_matches('/')),
            token: locator,
            ativo_raw: 1,
        })
    }
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── fixtures ──────────────────────────────────────────────────────────────

    const RELATORIOS_JSON: &str = r#"[
        {"Id": 1, "Nome": "Relatório de Agenda", "Tipo": "agenda"},
        {"Id": 2, "Nome": "Relatório Financeiro", "Tipo": "financeiro"}
    ]"#;

    const DOC_FOLDERS_JSON: &str = r#"[
        {"Id": 3, "Nome": "Documentos do Paciente", "Filter": "paciente"},
        {"Id": 7, "Nome": "Exames", "Filter": "exames"}
    ]"#;

    const ICS_CONFIG_JSON: &str = r#"{
        "Url": "https://example.com/ics/feed",
        "Token": "tok_ics_abc123",
        "Ativo": 1
    }"#;

    // ── testes ────────────────────────────────────────────────────────────────

    #[test]
    fn deserializa_vec_relatorio() {
        let raw: serde_json::Value = serde_json::from_str(RELATORIOS_JSON).unwrap();
        let lista: Vec<Relatorio> = parse_vec(raw);

        assert_eq!(lista.len(), 2);

        assert_eq!(lista[0].id, 1);
        assert_eq!(lista[0].name, "Relatório de Agenda");
        assert_eq!(lista[0].tipo, "agenda");

        assert_eq!(lista[1].id, 2);
        assert_eq!(lista[1].name, "Relatório Financeiro");
        assert_eq!(lista[1].tipo, "financeiro");
    }

    #[test]
    fn deserializa_vec_doc_folder() {
        let raw: serde_json::Value = serde_json::from_str(DOC_FOLDERS_JSON).unwrap();
        let lista: Vec<DocFolder> = parse_vec(raw);

        assert_eq!(lista.len(), 2);

        assert_eq!(lista[0].id, 3);
        assert_eq!(lista[0].name, "Documentos do Paciente");
        assert_eq!(lista[0].filter_key, "paciente");

        assert_eq!(lista[1].id, 7);
        assert_eq!(lista[1].name, "Exames");
        assert_eq!(lista[1].filter_key, "exames");
    }

    #[test]
    fn deserializa_ics_config_completo_e_verifica_active() {
        let cfg: IcsConfig = serde_json::from_str(ICS_CONFIG_JSON).unwrap();

        assert_eq!(cfg.url, "https://example.com/ics/feed");
        assert_eq!(cfg.token, "tok_ics_abc123");
        assert!(cfg.active(), "Ativo=1 deve resultar em active()=true");

        // Garante que Ativo=0 torna active() falso
        let json_inativo = r#"{"Url": "https://x.com", "Token": "t", "Ativo": 0}"#;
        let inativo: IcsConfig = serde_json::from_str(json_inativo).unwrap();
        assert!(!inativo.active(), "Ativo=0 deve resultar em active()=false");
    }

    #[test]
    fn deserializa_ics_config_com_campos_nulos() {
        let json = r#"{"Url": null, "Token": null, "Ativo": null}"#;
        let cfg: IcsConfig = serde_json::from_str(json).unwrap();

        assert_eq!(cfg.url, "");
        assert_eq!(cfg.token, "");
        assert!(!cfg.active(), "Ativo=null (→0) deve resultar em active()=false");
    }

    #[test]
    fn parse_vec_retorna_vazio_para_resposta_nao_array() {
        // Objeto JSON
        let obj: serde_json::Value = serde_json::json!({"Id": 1, "Nome": "x"});
        let lista: Vec<Relatorio> = parse_vec(obj);
        assert!(lista.is_empty(), "objeto não deve produzir elementos");

        // null JSON
        let null_val: serde_json::Value = serde_json::Value::Null;
        let lista2: Vec<DocFolder> = parse_vec(null_val);
        assert!(lista2.is_empty(), "null não deve produzir elementos");

        // String JSON
        let str_val: serde_json::Value = serde_json::json!("apenas uma string");
        let lista3: Vec<Relatorio> = parse_vec(str_val);
        assert!(lista3.is_empty(), "string não deve produzir elementos");
    }
}
