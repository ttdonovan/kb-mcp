//! Structured frontmatter queries — filter documents by tags, status, dates.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryParams {
    /// Filter by tag (e.g. "vector-search")
    #[serde(default)]
    pub tag: Option<String>,
    /// Filter by frontmatter status field
    #[serde(default)]
    pub status: Option<String>,
    /// Filter by created date (YYYY-MM-DD, returns docs created on or after)
    #[serde(default)]
    pub created_after: Option<String>,
    /// Filter by collection name
    #[serde(default)]
    pub collection: Option<String>,
    /// Only return documents that have a sources field
    #[serde(default)]
    pub has_sources: bool,
}

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::query_router()
}

#[rmcp::tool_router(router = query_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "kb_query",
        description = "Filter documents by frontmatter fields. Supports tag, status, created_after (YYYY-MM-DD), collection, and has_sources. Multiple filters combine with AND logic. Returns document metadata without body."
    )]
    pub(crate) async fn kb_query(
        &self,
        Parameters(params): Parameters<QueryParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        if let Some(ref date_str) = params.created_after
            && chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err()
        {
            return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                format!(
                    "Invalid created_after date '{}', expected YYYY-MM-DD format",
                    date_str
                ),
            )]));
        }

        self.auto_reindex_stale_collections().await;
        let index = self.index.read().await;
        let results: Vec<&kb_core::types::Document> = index
            .documents
            .iter()
            .filter(|doc| {
                kb_core::query::matches_query(
                    doc,
                    params.collection.as_deref(),
                    params.tag.as_deref(),
                    params.status.as_deref(),
                    params.created_after.as_deref(),
                    params.has_sources,
                )
            })
            .collect();
        let json = kb_core::format::format_query(&results);
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json,
        )]))
    }
}
