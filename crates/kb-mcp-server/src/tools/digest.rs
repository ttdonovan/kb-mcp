//! Vault summary tool — coverage, gaps, recent additions.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DigestParams {
    /// Filter to a specific collection (default: all)
    #[serde(default)]
    pub collection: Option<String>,
}

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::digest_router()
}

#[rmcp::tool_router(router = digest_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "kb_digest",
        description = "Vault summary — shows collections, sections with topics, recent additions (last 7 days), and thin sections (<2 docs). Use this to understand what the knowledge base covers before searching."
    )]
    pub(crate) async fn kb_digest(
        &self,
        Parameters(params): Parameters<DigestParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.auto_reindex_stale_collections().await;
        let index = self.index.read().await;
        let json = kb_core::format::format_digest(
            &index.documents,
            &index.sections,
            params.collection.as_deref(),
        );
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json,
        )]))
    }
}
