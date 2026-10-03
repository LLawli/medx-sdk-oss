//! Testes de integração — Etapa 2: Configurações.
//! Execute com: `cargo test --test settings_tests -- --ignored`

mod common;
use common::{shared_client, with_temp_dir};
use medx::MedxClient;

fn client() -> &'static MedxClient {
    shared_client()
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_general_parameters_tem_horarios() {
    with_temp_dir(|| {
        let p = client()
            .general_parameters()
            .expect("general_parameters falhou");
        // Horários no formato "HHMM" (4 dígitos)
        assert_eq!(
            p.business_hours_start.len(),
            4,
            "horariode deve ter 4 chars"
        );
        assert_eq!(p.business_hours_end.len(), 4, "horarioate deve ter 4 chars");
        assert!(
            !p.sms_clinic_name.is_empty(),
            "nome da clínica SMS não deve ser vazio"
        );
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_general_parameters_macros_somam_100() {
    with_temp_dir(|| {
        let p = client().general_parameters().unwrap();
        let carbs: u32 = p.carbs_pct.parse().unwrap_or(0);
        let protein: u32 = p.protein_pct.parse().unwrap_or(0);
        let fat: u32 = p.fat_pct.parse().unwrap_or(0);
        let fiber: u32 = p.fiber_pct.parse().unwrap_or(0);
        let total = carbs + protein + fat + fiber;
        assert_eq!(
            total, 100,
            "macronutrientes devem somar 100%, obtido: {total}"
        );
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_general_parameters_quick_buttons_nao_vazio() {
    with_temp_dir(|| {
        let p = client().general_parameters().unwrap();
        let buttons = p.quick_buttons_list();
        assert!(
            !buttons.is_empty(),
            "deve haver ao menos um botão de atalho"
        );
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_color_parameters_cores_e_labels_mesmo_tamanho() {
    with_temp_dir(|| {
        let cp = client()
            .color_parameters()
            .expect("color_parameters falhou");
        assert_eq!(
            cp.colors.len(),
            cp.labels.len(),
            "colors e labels devem ter o mesmo tamanho"
        );
        assert!(!cp.colors.is_empty(), "deve haver ao menos uma cor");
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_color_parameters_cores_sao_hex() {
    with_temp_dir(|| {
        let cp = client().color_parameters().unwrap();
        for color in &cp.colors {
            assert!(
                color.starts_with('#') && color.len() == 7,
                "cor deve ser #RRGGBB, obtido: {color}"
            );
        }
    });
}

#[test]
#[ignore = "requer rede e credenciais válidas"]
fn integration_color_pairs_emparelha_corretamente() {
    with_temp_dir(|| {
        let cp = client().color_parameters().unwrap();
        let pairs = cp.pairs();
        assert_eq!(pairs.len(), cp.colors.len());
        // Primeiro par: cor corresponde à primeira cor
        assert_eq!(pairs[0].0, cp.colors[0]);
        assert_eq!(pairs[0].1, cp.labels[0]);
    });
}
