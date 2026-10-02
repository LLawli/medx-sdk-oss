//! Utilitários dos testes de integração.

#![allow(dead_code)]

use medx::Session;
use medx_mcp::config::Config;
use medx_mcp::server::MedxServer;
use rmcp::service::RunningService;
use rmcp::{RoleClient, ServiceExt};

pub type Client = RunningService<RoleClient, ()>;

/// Um cliente MCP ligado a um servidor em processo por um canal duplex em
/// memória.
pub async fn connect(config: Config, session: Option<Session>) -> Client {
    let (server_io, client_io) = tokio::io::duplex(1 << 20);
    tokio::spawn(async move {
        let server = MedxServer::new(config, session)
            .serve(server_io)
            .await
            .expect("o servidor sobe");
        let _ = server.waiting().await;
    });
    ().serve(client_io).await.expect("o cliente conecta")
}
