//! Structured frontmatter queries — filter documents by tags, status, dates.
//!
//! Linear scan of `Index.documents` checking each document's frontmatter
//! HashMap against the query params. Multiple filters combine with AND logic.
//! Returns document metadata without body content.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::format;
use crate::server::KbMcpServer;
use crate::types::Document;

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
        // Validate date format upfront so agents get a clear error
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
        let results: Vec<&Document> = index
            .documents
            .iter()
            .filter(|doc| matches_query(doc, &params))
            .collect();
        let json = format::format_query(&results);
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json,
        )]))
    }
}

/// Check if a document matches all query filters (AND logic).
pub fn matches_query(doc: &Document, params: &QueryParams) -> bool {
    if params.collection.as_ref().is_some_and(|c| doc.collection != *c) {
        return false;
    }

    if let Some(ref tag) = params.tag {
        let tag_lower = tag.to_lowercase();
        if !doc.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
            return false;
        }
    }

    if let Some(ref status) = params.status {
        let has_status = doc
            .frontmatter
            .get("status")
            .and_then(|v| match v {
                serde_yaml::Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .is_some_and(|s| s == status);
        if !has_status {
            return false;
        }
    }

    if let Some(ref after) = params.created_after
        && let Ok(after_date) = chrono::NaiveDate::parse_from_str(after, "%Y-%m-%d")
    {
        let created = doc
            .frontmatter
            .get("created")
            .and_then(|v| {
                let s = crate::format::yaml_value_to_string(v);
                chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()
            });
        match created {
            Some(d) if d >= after_date => {}
            _ => return false,
        }
    }

    if params.has_sources && !doc.frontmatter.contains_key("sources") {
        return false;
    }

    true
}
