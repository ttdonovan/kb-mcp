use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::format;
use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Search query — supports terms, phrases ("exact match"), and boolean operators (AND, OR).
    pub query: String,
    /// Optional collection filter (e.g. "vault", "skills")
    #[serde(default)]
    pub collection: Option<String>,
    /// Optional section scope filter (e.g. "runtimes", "concepts")
    #[serde(default)]
    pub scope: Option<String>,
    /// Maximum number of results to return (default: 10)
    #[serde(default = "default_max_results")]
    pub max_results: usize,
}

fn default_max_results() -> usize {
    10
}

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::search_router()
}

#[rmcp::tool_router(router = search_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "search",
        description = "Full-text search across the knowledge base. Uses BM25 ranking with stemming. Supports phrases (\"exact match\") and boolean operators (AND, OR). Returns ranked results with excerpts. Filter by collection name or section scope. Automatically detects and re-syncs stale collections."
    )]
    pub(crate) async fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        // Auto-reindex: check if any collection has changed since last sync
        self.auto_reindex_stale_collections().await;

        let index = self.index.read().await;
        let results = self.search_engine.search(
            &index.documents,
            &params.query,
            params.collection.as_deref(),
            params.scope.as_deref(),
            params.max_results,
        );

        let json = format::format_search(&params.query, &results, &index.documents);
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json,
        )]))
    }
}
