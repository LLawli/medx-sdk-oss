//! Informações que o servidor entrega no handshake.

mod common;

use medx_mcp::config::Config;

#[tokio::test]
async fn handshake_entrega_nome_versao_e_instrucoes() {
    let client = common::connect(Config::default(), None).await;
    let info = client.peer_info().expect("informações do servidor");

    let server_info = info.server_info.as_ref().expect("server_info");
    assert_eq!(server_info.name, "medx-mcp");
    assert_eq!(server_info.version, env!("CARGO_PKG_VERSION"));

    let instructions = info.instructions.as_deref().expect("instruções");
    assert_eq!(instructions, include_str!("../src/instructions.md"));
    assert!(
        info.capabilities.tools.is_some(),
        "o servidor deveria anunciar ferramentas"
    );
}
