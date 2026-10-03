//! Etapa 2 — Parâmetros gerais da clínica e configurações de cores.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{client::MedxClient, error::MedxError};

// ── Deserializador auxiliar ───────────────────────────────────────────────────

/// Converte strings `"True"` / `"False"` da API em `bool`.
fn de_str_bool<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    let s = String::deserialize(d)?;
    Ok(s.eq_ignore_ascii_case("true"))
}

// ── Tipos ─────────────────────────────────────────────────────────────────────

/// Parâmetros gerais da conta/clínica (`GET settings/GetGeneralParameters`).
///
/// A API retorna um array com um único elemento; este tipo representa esse elemento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralParameters {
    /// Horário de início do expediente, ex: `"0800"`.
    #[serde(rename(deserialize = "horariode"))]
    pub business_hours_start: String,
    /// Horário de fim do expediente, ex: `"2000"`.
    #[serde(rename(deserialize = "horarioate"))]
    pub business_hours_end: String,
    /// Duração padrão do slot de agenda, ex: `"0630"` (6h30).
    #[serde(rename(deserialize = "horarioduracao"))]
    pub slot_duration: String,
    /// Nome da clínica usado nos SMS.
    #[serde(rename(deserialize = "smsclinica"))]
    pub sms_clinic_name: String,
    #[serde(rename(deserialize = "smspais"))]
    pub sms_country: String,
    #[serde(rename(deserialize = "smsddd"))]
    pub sms_area_code: String,
    #[serde(rename(deserialize = "smstel"))]
    pub sms_phone: String,
    #[serde(rename(deserialize = "smsemail"))]
    pub sms_email: String,
    /// Habilita diagnóstico QP na agenda.
    #[serde(
        rename(deserialize = "checksdiagnostico"),
        deserialize_with = "de_str_bool"
    )]
    pub diagnostics_enabled: bool,
    /// Habilita cronômetro de atendimento.
    #[serde(
        rename(deserialize = "checkscronometro"),
        deserialize_with = "de_str_bool"
    )]
    pub timer_enabled: bool,
    /// % de carboidratos no plano nutricional padrão.
    #[serde(rename(deserialize = "carboidratos"))]
    pub carbs_pct: String,
    /// % de proteínas no plano nutricional padrão.
    #[serde(rename(deserialize = "proteinas"))]
    pub protein_pct: String,
    /// % de gorduras no plano nutricional padrão.
    #[serde(rename(deserialize = "gorduras"))]
    pub fat_pct: String,
    /// % de fibras no plano nutricional padrão.
    #[serde(rename(deserialize = "fibras"))]
    pub fiber_pct: String,
    /// Botões de atalho do prontuário separados por `|`.
    #[serde(rename(deserialize = "botaopref1"))]
    pub quick_buttons: String,
}

impl GeneralParameters {
    /// Retorna a lista de botões de atalho do prontuário.
    pub fn quick_buttons_list(&self) -> Vec<&str> {
        self.quick_buttons.split('|').collect()
    }
}

/// Paleta de cores e rótulos usados na agenda (`GET parametrosCores/getallparametroscores`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorParameters {
    #[serde(rename(deserialize = "Colors"))]
    pub colors: Vec<String>,
    #[serde(rename(deserialize = "Labels"))]
    pub labels: Vec<String>,
}

impl ColorParameters {
    /// Retorna os pares `(cor_hex, rótulo)` da paleta da agenda.
    pub fn pairs(&self) -> Vec<(&str, &str)> {
        self.colors
            .iter()
            .zip(self.labels.iter())
            .map(|(c, l)| (c.as_str(), l.as_str()))
            .collect()
    }
}

// ── Métodos do MedxClient ─────────────────────────────────────────────────────

impl MedxClient {
    /// Retorna os parâmetros gerais da clínica.
    pub fn general_parameters(&self) -> Result<GeneralParameters, MedxError> {
        let list: Vec<GeneralParameters> = self.get("settings/GetGeneralParameters")?;
        list.into_iter().next().ok_or_else(|| {
            MedxError::UnexpectedResponse("GetGeneralParameters retornou lista vazia".into())
        })
    }

    /// Retorna as cores e rótulos de status da agenda.
    pub fn color_parameters(&self) -> Result<ColorParameters, MedxError> {
        self.get("parametrosCores/getallparametroscores")
    }
}

// ── Testes unitários ──────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const GENERAL_PARAMS_JSON: &str = r#"[{
        "horariode": "0800",
        "horarioate": "2000",
        "horarioduracao": "0630",
        "smsclinica": "CLINICA TESTE",
        "smspais": "BRASIL",
        "smsddd": "62",
        "smstel": "00000000",
        "smsemail": "",
        "checksdiagnostico": "False",
        "checkscronometro": "False",
        "carboidratos": "40",
        "proteinas": "30",
        "gorduras": "25",
        "fibras": "5",
        "botaopref1": "Receituário|Resultados de Exames|Galeria de Imagens|Percentis"
    }]"#;

    pub const COLOR_PARAMS_JSON: &str = r##"{
        "Colors": ["#A9A9A9", "#FFFFFF", "#90EE90"],
        "Labels": ["DESMARCADO", "AGENDADO", "COMPARECEU"]
    }"##;

    #[test]
    fn deserializa_general_parameters() {
        let list: Vec<GeneralParameters> = serde_json::from_str(GENERAL_PARAMS_JSON).unwrap();
        assert_eq!(list.len(), 1);
        let p = &list[0];
        assert_eq!(p.business_hours_start, "0800");
        assert_eq!(p.business_hours_end, "2000");
        assert_eq!(p.slot_duration, "0630");
        assert_eq!(p.sms_clinic_name, "CLINICA TESTE");
        assert_eq!(p.sms_country, "BRASIL");
        assert_eq!(p.sms_area_code, "62");
        assert!(!p.diagnostics_enabled);
        assert!(!p.timer_enabled);
        assert_eq!(p.carbs_pct, "40");
    }

    #[test]
    fn de_str_bool_aceita_true_case_insensitive() {
        let json = GENERAL_PARAMS_JSON.replace("\"False\"", "\"True\"");
        let list: Vec<GeneralParameters> = serde_json::from_str(&json).unwrap();
        assert!(list[0].diagnostics_enabled);
        assert!(list[0].timer_enabled);
    }

    #[test]
    fn quick_buttons_list_split_correto() {
        let list: Vec<GeneralParameters> = serde_json::from_str(GENERAL_PARAMS_JSON).unwrap();
        let buttons = list[0].quick_buttons_list();
        assert_eq!(
            buttons,
            vec![
                "Receituário",
                "Resultados de Exames",
                "Galeria de Imagens",
                "Percentis",
            ]
        );
    }

    #[test]
    fn deserializa_color_parameters() {
        let cp: ColorParameters = serde_json::from_str(COLOR_PARAMS_JSON).unwrap();
        assert_eq!(cp.colors.len(), 3);
        assert_eq!(cp.labels.len(), 3);
        assert_eq!(cp.colors[0], "#A9A9A9");
        assert_eq!(cp.labels[0], "DESMARCADO");
    }

    #[test]
    fn color_pairs_emparelha_corretamente() {
        let cp: ColorParameters = serde_json::from_str(COLOR_PARAMS_JSON).unwrap();
        let pairs = cp.pairs();
        assert_eq!(pairs[0], ("#A9A9A9", "DESMARCADO"));
        assert_eq!(pairs[1], ("#FFFFFF", "AGENDADO"));
        assert_eq!(pairs[2], ("#90EE90", "COMPARECEU"));
    }

    #[test]
    fn general_params_lista_vazia_seria_erro() {
        // Valida que nosso parse de "primeiro item" falharia em lista vazia
        let empty: Vec<GeneralParameters> = serde_json::from_str("[]").unwrap();
        assert!(empty.into_iter().next().is_none());
    }
}
