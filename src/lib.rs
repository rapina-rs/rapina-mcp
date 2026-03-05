mod tools;

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{ServerCapabilities, ServerInfo},
    tool_handler, tool_router,
};

#[derive(Debug, Clone)]
pub struct RapinaMcp {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl RapinaMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_handler]
impl ServerHandler for RapinaMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                rmcp::model::Implementation::from_build_env(),
            )
            .with_instructions(
                "MCP server for the Rapina web framework. \
                 Provides tools to scaffold projects, inspect routes, \
                 run diagnostics, generate code, and introspect Rapina applications."
                    .to_string(),
            )
    }
}
