//! medx-mcp: servidor MCP da MedX sobre stdio. O stdout leva só JSON-RPC; os
//! logs vão para o stderr, filtrados por `RUST_LOG`.

use std::process::ExitCode;

use medx_mcp::config::Config;
use medx_mcp::server::MedxServer;
use rmcp::ServiceExt;
use rmcp::service::ServerInitializeError;
use rmcp::transport::stdio;
use tracing_subscriber::EnvFilter;

#[derive(Debug, thiserror::Error)]
enum FatalError {
    #[error("não foi possível iniciar o runtime assíncrono: {0}")]
    Runtime(#[source] std::io::Error),
    #[error("falha no handshake MCP: {0}")]
    Handshake(String),
    #[error("a tarefa do servidor falhou: {0}")]
    Serve(#[source] tokio::task::JoinError),
}

fn main() -> ExitCode {
    // Antes do tracing e do runtime: configuração inválida sai sem tocar no
    // stdout e sem subir nada.
    let config = match Config::from_vars(|name| std::env::var(name).ok()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("medx-mcp: {err}");
            return ExitCode::from(2);
        }
    };
    init_tracing();
    tracing::info!("medx-mcp {} iniciando", env!("CARGO_PKG_VERSION"));

    // Leitura síncrona do session.json, feita fora do runtime. Não cria
    // MedxClient aqui: o cliente blocking não pode nascer dentro do runtime.
    let session = medx::load_session();

    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(FatalError::Runtime)
        .and_then(|runtime| runtime.block_on(run(config, session)));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("medx-mcp: {err}");
            ExitCode::FAILURE
        }
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // try_init: um segundo subscriber global não vale derrubar o servidor.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .try_init();
}

async fn run(config: Config, session: Option<medx::Session>) -> Result<(), FatalError> {
    let service = match MedxServer::new(config, session).serve(stdio()).await {
        Ok(service) => service,
        // O stdin fechou antes do `initialize`: o cliente foi embora, o que é
        // um encerramento normal e não uma falha de protocolo.
        Err(ServerInitializeError::ConnectionClosed(reason)) => {
            tracing::info!("stdin fechado antes do handshake: {reason}");
            return Ok(());
        }
        Err(err) => return Err(FatalError::Handshake(err.to_string())),
    };
    service.waiting().await.map_err(FatalError::Serve)?;
    Ok(())
}
