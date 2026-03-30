//! Vault health diagnostics — document quality and hygiene checks.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::KbMcpServer;

fn default_stale_days() -> u32 {
    90
}

fn default_min_words() -> u32 {
    50
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HealthParams {
    /// Filter to a specific collection
    #[serde(default)]
    pub collection: Option<String>,
    /// Days threshold for staleness (default: 90)
    #[serde(default = "default_stale_days")]
    pub stale_days: u32,
    /// Minimum word count — documents below this are flagged as stubs (default: 50)
    #[serde(default = "default_min_words")]
    pub min_words: u32,
}

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::health_router()
}

#[rmcp::tool_router(router = health_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "kb_health",
        description = "Vault health diagnostics — checks document quality across collections. Flags missing frontmatter dates, untagged docs, stale content, stub documents, orphaned notes (no inbound wiki-links), and broken wiki-links. Use kb_digest for coverage overview, kb_health for quality issues."
    )]
    pub(crate) async fn kb_health(
        &self,
        Parameters(params): Parameters<HealthParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if let Some(ref coll) = params.collection
            && !self.collections.iter().any(|c| c.name == *coll)
        {
            let available: Vec<&str> =
                self.collections.iter().map(|c| c.name.as_str()).collect();
            return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                format!(
                    "Collection '{}' not found. Available: {}",
                    coll,
                    available.join(", ")
                ),
            )]));
        }

        self.auto_reindex_stale_collections().await;
        let index = self.index.read().await;
        let json = kb_core::format::format_health(
            &index.documents,
            params.collection.as_deref(),
            params.stale_days,
            params.min_words,
        );
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json,
        )]))
    }
}
