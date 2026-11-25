use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDocumentParams {
    /// Path to the document (e.g. "concepts/mcp-server-pattern.md") or document title
    pub path: String,
    /// Collection to look in; needed when the path or title is in more than one
    pub collection: Option<String>,
}

pub fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
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
        let json = index
            .lookup(&params.path, params.collection.as_deref())
            .map(|doc| {
                self.read_fresh(doc).map_or_else(
                    || kb_core::format::format_document(doc, true),
                    |fresh| {
                        let mut fresh_doc = doc.clone();
                        fresh_doc.body = fresh;
                        kb_core::format::format_document(&fresh_doc, true)
                    },
                )
            });
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
