//! O servidor MCP: roteamento das ferramentas e informações do servidor.

use medx::Session;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, tool_handler};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct MedxServer {
    config: Config,
    session: Option<Session>,
    tool_router: ToolRouter<Self>,
}

impl MedxServer {
    /// `session` é a sessão salva (`medx::load_session()` no binário).
    pub fn new(config: Config, session: Option<Session>) -> Self {
        Self {
            config,
            session,
            tool_router: Self::tool_router(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// Todos os grupos de ferramentas, somados aqui com `+`. Ainda vazio.
    fn tool_router() -> ToolRouter<Self> {
        ToolRouter::new()
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
