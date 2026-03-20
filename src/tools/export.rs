//! Vault export — concatenate all documents into a single markdown file.
//!
//! Reads documents fresh from disk (not from the index) to ensure
//! exported content is always current. Documents include their YAML
//! frontmatter as metadata headers. The markdown assembly is delegated
//! to `format::format_export` so MCP and CLI produce identical output.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::format;
use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportParams {
    /// Collection to export (default: all)
    #[serde(default)]
    pub collection: Option<String>,
}

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::export_router()
}

#[rmcp::tool_router(router = export_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "kb_export",
        description = "Export vault as a single markdown document. Concatenates all documents with frontmatter headers. Use to create a portable snapshot of knowledge base content."
    )]
    pub(crate) async fn kb_export(
        &self,
        Parameters(params): Parameters<ExportParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.auto_reindex_stale_collections().await;
        let index = self.index.read().await;

        let docs_with_bodies: Vec<(&crate::types::Document, String)> = index
            .documents
            .iter()
            .filter(|doc| {
                params
                    .collection
                    .as_ref()
                    .is_none_or(|f| doc.collection == *f)
            })
            .filter_map(|doc| {
                let body = self.read_fresh(doc)?;
                Some((doc, body))
            })
            .collect();

        if docs_with_bodies.is_empty() {
            let msg = match &params.collection {
                Some(c) => format!("No documents found in collection '{}'", c),
                None => "No documents found".to_string(),
            };
            return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                msg,
            )]));
        }

        let output = format::format_export(&docs_with_bodies, params.collection.as_deref());
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            output,
        )]))
    }
}
