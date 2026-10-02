//! O catálogo de ferramentas: as leituras sempre; as escritas só com
//! `MEDX_MCP_ALLOW_WRITE`, e com as anotações certas.

mod common;

use common::fake_medx::FakeMedx;
use common::{connect_fake, connect_fake_with_write, tool_names};

const READ_TOOLS: [&str; 46] = [
    "agenda_do_dia",
    "atendimentos_do_paciente",
    "bloqueios_do_dia",
    "buscar_no_prontuario",
    "buscar_pacientes",
    "calendario_ics",
    "foto_paciente",
    "galeria_de_fotos",
    "historico_chat",
    "link_arquivo_prontuario",
    "listar_atendimentos",
    "listar_convenios",
    "listar_diagnosticos_qp",
    "listar_eventos",
    "listar_formularios",
    "listar_notas",
    "listar_planos_de_saude",
    "listar_procedimentos",
    "listar_profissionais_agenda",
    "listar_questionarios",
    "listar_relatorios",
    "listar_setores_agenda",
    "listar_unidades",
    "listar_usuarios",
    "listar_usuarios_chat",
    "mensagens_nao_lidas_chat",
    "mensagens_recebidas_chat",
    "modelos_de_mensagem",
    "notificacoes_de_hoje",
    "pacientes_homonimos",
    "palavras_chave_prontuario",
    "parametros_agenda",
    "parametros_de_cores",
    "parametros_gerais",
    "pastas_de_documentos",
    "procedimentos_do_convenio",
    "registros_do_modulo",
    "relatorio_agenda",
    "relatorio_faltas",
    "relatorio_prontuario",
    "sumario_prontuario",
    "ultimos_atendidos",
    "usuario_atual",
    "ver_formulario",
    "ver_paciente",
    "ver_prontuario",
];

/// Escritas: (nome, destrutiva, fala com terceiros).
const WRITE_TOOLS: [(&str, bool, bool); 14] = [
    ("atualizar_paciente", true, false),
    ("atualizar_sumario_prontuario", true, false),
    ("bloquear_horario", false, false),
    ("cadastrar_paciente", false, false),
    ("confirmar_agendamento_whatsapp", false, true),
    ("criar_agendamento", false, false),
    ("criar_nota", false, false),
    ("editar_nota", true, false),
    ("editar_registro_prontuario", true, false),
    ("enviar_mensagem_chat", false, false),
    ("marcar_mensagens_lidas", false, false),
    ("mudar_status_agendamento", true, false),
    ("registrar_no_prontuario", false, false),
    ("remarcar_agendamento", true, false),
];

#[tokio::test]
async fn sem_allow_write_so_existem_leituras() {
    let fake = FakeMedx::start();
    let client = connect_fake(&fake).await;

    assert_eq!(tool_names(&client).await, READ_TOOLS);
}

#[tokio::test]
async fn com_allow_write_aparecem_as_escritas() {
    let fake = FakeMedx::start();
    let client = connect_fake_with_write(&fake).await;

    let mut expected: Vec<&str> = READ_TOOLS.to_vec();
    expected.extend(WRITE_TOOLS.iter().map(|(name, _, _)| *name));
    expected.sort_unstable();
    assert_eq!(tool_names(&client).await, expected);
}

#[tokio::test]
async fn toda_ferramenta_tem_descricao_e_anotacoes() {
    let fake = FakeMedx::start();
    let client = connect_fake_with_write(&fake).await;
    let tools = client.list_all_tools().await.expect("tools/list");

    for tool in &tools {
        let name = tool.name.as_ref();
        assert!(
            tool.description.as_deref().is_some_and(|d| d.len() > 20),
            "{name} sem descrição útil"
        );
        let annotations = tool
            .annotations
            .as_ref()
            .unwrap_or_else(|| panic!("{name} sem anotações"));
        match WRITE_TOOLS.iter().find(|(write, _, _)| *write == name) {
            None => {
                assert_eq!(annotations.read_only_hint, Some(true), "{name}");
                assert_eq!(annotations.open_world_hint, Some(false), "{name}");
            }
            Some((_, destructive, open_world)) => {
                assert_eq!(annotations.read_only_hint, Some(false), "{name}");
                assert_eq!(annotations.destructive_hint, Some(*destructive), "{name}");
                assert_eq!(annotations.open_world_hint, Some(*open_world), "{name}");
            }
        }
    }
}
