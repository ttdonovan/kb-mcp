use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContextParams {
    /// Path to the document or document title
    pub path: String,
}

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
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
        let doc = index
            .find_by_path(&params.path)
            .or_else(|| index.find_by_title(&params.path));

        match doc {
            Some(doc) => {
                let json = kb_core::format::format_context(doc);
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    json,
                )]))
            }
            None => Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                format!("Document not found: {}", params.path),
            )])),
        }
    }
}
