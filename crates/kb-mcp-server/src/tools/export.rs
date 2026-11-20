//! Vault export — concatenate all documents into a single markdown file.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use std::fmt::Write as _;

use crate::server::KbMcpServer;

const fn default_max_documents() -> usize {
    200
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportParams {
    /// Collection to export (default: all)
    #[serde(default)]
    pub collection: Option<String>,
    /// Maximum number of documents to include (default: 200).
    /// Prevents unbounded output for large vaults.
    #[serde(default = "default_max_documents")]
    pub max_documents: usize,
}

pub fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::export_router()
}

#[rmcp::tool_router(router = export_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "kb_export",
        description = "Export vault as a single markdown document. Concatenates documents with frontmatter headers. Limited to max_documents (default 200) to prevent unbounded output. Use to create a portable snapshot of knowledge base content."
    )]
    pub(crate) async fn kb_export(
        &self,
        Parameters(params): Parameters<ExportParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.auto_reindex_stale_collections().await;
        let index = self.index.read().await;

        let matching_count = index
            .documents
            .iter()
            .filter(|doc| {
                params
                    .collection
                    .as_ref()
                    .is_none_or(|f| doc.collection == *f)
            })
            .count();

        let docs_with_bodies: Vec<(&kb_core::types::Document, String)> = index
            .documents
            .iter()
            .filter(|doc| {
                params
                    .collection
                    .as_ref()
                    .is_none_or(|f| doc.collection == *f)
            })
            .take(params.max_documents)
            .filter_map(|doc| {
                let body = self.read_fresh(doc)?;
                Some((doc, body))
            })
            .collect();

        if docs_with_bodies.is_empty() {
            let msg = params.collection.as_ref().map_or_else(
                || "No documents found".to_string(),
                |c| format!("No documents found in collection '{c}'"),
            );
            return Ok(CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(msg),
            ]));
        }

        let mut output =
            kb_core::format::format_export(&docs_with_bodies, params.collection.as_deref());
        let shown = docs_with_bodies.len();
        drop(docs_with_bodies);
        drop(index);

        if matching_count > params.max_documents {
            // Writing to a String cannot fail.
            let _ = writeln!(
                output,
                "... truncated: showing {shown} of {matching_count} documents"
            );
        }
        Ok(CallToolResult::success(vec![
            rmcp::model::ContentBlock::text(output),
        ]))
    }
}
