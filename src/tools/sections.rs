use rmcp::model::CallToolResult;

use crate::format;
use crate::server::KbMcpServer;

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::sections_router()
}

#[rmcp::tool_router(router = sections_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "list_sections",
        description = "List all collections and their sections with document counts and descriptions. Use this to discover what knowledge areas are available."
    )]
    pub(crate) async fn list_sections(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let index = self.index.read().await;
        let json = format::format_sections(&index.sections);
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json,
        )]))
    }
}
