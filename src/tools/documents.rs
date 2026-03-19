use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::format;
use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDocumentParams {
    /// Path to the document (e.g. "concepts/mcp-server-pattern.md") or document title
    pub path: String,
}

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::documents_router()
}

#[rmcp::tool_router(router = documents_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "get_document",
        description = "Retrieve a document by path or title. Returns the full document content read fresh from disk. Use list_sections or search to discover documents first."
    )]
    pub(crate) async fn get_document(
        &self,
        Parameters(params): Parameters<GetDocumentParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let index = self.index.read().await;
        let doc = index
            .find_by_path(&params.path)
            .or_else(|| index.find_by_title(&params.path));

        match doc {
            Some(doc) => {
                // Read fresh from disk if possible
                let content = self.read_fresh(doc);
                let json = if let Some(fresh) = content {
                    let mut fresh_doc = doc.clone();
                    fresh_doc.body = fresh;
                    format::format_document(&fresh_doc, true)
                } else {
                    format::format_document(doc, true)
                };
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
