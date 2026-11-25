use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContextParams {
    /// Path to the document or document title
    pub path: String,
    /// Collection to look in; needed when the path or title is in more than one
    pub collection: Option<String>,
}

pub fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::context_router()
}

#[rmcp::tool_router(router = context_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "kb_context",
        description = "Token-efficient document briefing. Returns frontmatter metadata and first paragraph summary without the full body. Use this to survey relevance before calling get_document for full content."
    )]
    pub(crate) async fn kb_context(
        &self,
        Parameters(params): Parameters<ContextParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let index = self.index.read().await;
        let json = index
            .lookup(&params.path, params.collection.as_deref())
            .map(kb_core::format::format_context);
        drop(index);

        match json {
            Ok(json) => Ok(CallToolResult::success(vec![
                rmcp::model::ContentBlock::text(json),
            ])),
            Err(message) => Ok(CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(message),
            ])),
        }
    }
}
