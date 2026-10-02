//! O servidor MCP: roteamento das ferramentas e informações do servidor.

use medx::Session;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, tool_handler};

use std::sync::Arc;

use crate::config::Config;
use crate::worker::MedxWorker;

#[derive(Clone)]
pub struct MedxServer {
    config: Config,
    session: Option<Session>,
    pub(crate) medx: Arc<MedxWorker>,
    tool_router: ToolRouter<Self>,
}

impl MedxServer {
    /// `session` é a sessão salva (`medx::load_session()` no binário).
    pub fn new(config: Config, session: Option<Session>) -> Self {
        Self {
            medx: Arc::new(MedxWorker::spawn(config.clone(), session.clone())),
            tool_router: Self::tool_router(config.allow_write),
            config,
            session,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// Todos os grupos de ferramentas, somados aqui com `+`. As escritas só
    /// entram com `allow_write`: sem ele, nem aparecem no `tools/list`.
    fn tool_router(allow_write: bool) -> ToolRouter<Self> {
        let reads = Self::usuarios_router()
            + Self::agenda_router()
            + Self::hoje_router()
            + Self::pacientes_router()
            + Self::prontuario_router()
            + Self::financeiro_router()
            + Self::configuracoes_router()
            + Self::chat_router();
        if allow_write {
            reads + Self::pacientes_escrita_router() + Self::agenda_escrita_router()
        } else {
            reads
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for MedxServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("medx-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(include_str!("instructions.md"))
    }
}
