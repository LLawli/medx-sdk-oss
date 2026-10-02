//! Utilitários dos testes de integração.

#![allow(dead_code)]

pub mod fake_medx;
pub mod fixtures;
pub mod stdio;

use std::time::Duration;

use medx::Session;
use medx_mcp::config::Config;
use medx_mcp::server::MedxServer;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::RunningService;
use rmcp::{RoleClient, ServiceExt};
use serde_json::Value;

pub type Client = RunningService<RoleClient, ()>;

/// Prazo de uma chamada de ferramenta nos testes.
pub const CALL_TIMEOUT: Duration = Duration::from_secs(30);

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

/// Um cliente ligado a um servidor apontado para o fake, com a sessão válida
/// do fake (nenhum teste em processo faz login: o login grava o session.json
/// no diretório de configuração do usuário).
pub async fn connect_fake(fake: &fake_medx::FakeMedx) -> Client {
    connect(fake.config(), Some(fake.session())).await
}

/// Como [`connect_fake`], com a escrita ligada.
pub async fn connect_fake_with_write(fake: &fake_medx::FakeMedx) -> Client {
    connect(fake.config_with_write(), Some(fake.session())).await
}

pub async fn call(client: &Client, tool: &str, arguments: Value) -> CallToolResult {
    let Value::Object(arguments) = arguments else {
        panic!("os argumentos da ferramenta têm de ser um objeto JSON");
    };
    let request =
        client.call_tool(CallToolRequestParams::new(tool.to_owned()).with_arguments(arguments));
    // Um pânico dentro da ferramenta derruba a resposta em vez de falhar a
    // chamada; sem prazo o teste ficaria pendurado.
    tokio::time::timeout(CALL_TIMEOUT, request)
        .await
        .unwrap_or_else(|_| panic!("{tool} não respondeu em {CALL_TIMEOUT:?}"))
        .unwrap_or_else(|err| panic!("{tool}: erro de protocolo {err:?}"))
}

/// Textos dos blocos de texto do resultado, na ordem.
pub fn texts(result: &CallToolResult) -> Vec<String> {
    result
        .content
        .iter()
        .filter_map(|block| block.as_text().map(|text| text.text.clone()))
        .collect()
}

/// Chama a ferramenta, exige sucesso e devolve o JSON do primeiro bloco.
pub async fn call_json(client: &Client, tool: &str, arguments: Value) -> Value {
    let result = call(client, tool, arguments).await;
    assert_ne!(result.is_error, Some(true), "{tool} falhou: {result:?}");
    let blocks = texts(&result);
    let first = blocks
        .first()
        .unwrap_or_else(|| panic!("{tool} sem bloco de texto: {result:?}"));
    serde_json::from_str(first)
        .unwrap_or_else(|err| panic!("{tool}: o primeiro bloco não é JSON ({err}): {first}"))
}

/// Chama a ferramenta, exige `isError` e devolve o texto.
pub async fn call_error(client: &Client, tool: &str, arguments: Value) -> String {
    let result = call(client, tool, arguments).await;
    assert_eq!(
        result.is_error,
        Some(true),
        "{tool} deveria falhar: {result:?}"
    );
    texts(&result).join("\n")
}

/// Nomes das ferramentas anunciadas, em ordem alfabética.
pub async fn tool_names(client: &Client) -> Vec<String> {
    let mut names: Vec<String> = client
        .list_all_tools()
        .await
        .expect("tools/list")
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect();
    names.sort_unstable();
    names
}
