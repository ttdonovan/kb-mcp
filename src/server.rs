//! MCP server implementation.
//!
//! Holds shared state (index, search engine, collections) behind Arc wrappers
//! and wires tool routers into the rmcp `ServerHandler` trait. The `Clone` derive
//! is required by rmcp — it clones the server for each request, but all data
//! lives behind Arcs so cloning is cheap.

use std::sync::Arc;

use rmcp::ServerHandler;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use tokio::sync::RwLock;

use crate::config::ResolvedCollection;
use crate::index::Index;
use crate::search::SearchEngine;
use crate::tools;

#[derive(Clone)]
pub struct KbMcpServer {
    pub(crate) index: Arc<RwLock<Index>>,
    pub(crate) search_engine: Arc<SearchEngine>,
    pub(crate) collections: Arc<Vec<ResolvedCollection>>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

impl KbMcpServer {
    pub fn new(
        index: Index,
        search_engine: SearchEngine,
        collections: Vec<ResolvedCollection>,
    ) -> Self {
        Self {
            index: Arc::new(RwLock::new(index)),
            search_engine: Arc::new(search_engine),
            collections: Arc::new(collections),
            tool_router: tools::combined_router(),
        }
    }

    /// Read a document fresh from disk rather than returning indexed content.
    /// This ensures `get_document` never serves stale content — edits are
    /// visible immediately without calling `reindex` first.
    pub(crate) fn read_fresh(&self, doc: &crate::types::Document) -> Option<String> {
        let coll = self.collections.iter().find(|c| c.name == doc.collection)?;
        let file_path = coll.path.join(&doc.path);
        let content = std::fs::read_to_string(&file_path).ok()?;

        // Strip frontmatter
        if let Some(after) = content.strip_prefix("---")
            && let Some(end) = after.find("\n---")
        {
            let body = &after[end + 4..];
            return Some(body.trim_start_matches('\n').to_string());
        }
        Some(content)
    }
}

#[rmcp::tool_handler]
impl ServerHandler for KbMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("kb-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "Knowledge base MCP server. Use list_sections to discover content areas, \
                 search to find documents, get_document to retrieve full content, \
                 kb_context for token-efficient briefings, kb_write to create notes, \
                 and reindex to refresh after adding new files."
                    .to_string(),
            )
    }
}
