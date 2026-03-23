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
    pub(crate) cache_dir: std::path::PathBuf,
    #[cfg(feature = "hybrid")]
    pub(crate) embedder: Arc<memvid_core::LocalTextEmbedder>,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

impl KbMcpServer {
    pub fn new(
        index: Index,
        search_engine: SearchEngine,
        collections: Vec<ResolvedCollection>,
        cache_dir: std::path::PathBuf,
        #[cfg(feature = "hybrid")] embedder: Arc<memvid_core::LocalTextEmbedder>,
    ) -> Self {
        Self {
            index: Arc::new(RwLock::new(index)),
            search_engine: Arc::new(search_engine),
            collections: Arc::new(collections),
            cache_dir,
            #[cfg(feature = "hybrid")]
            embedder,
            tool_router: tools::combined_router(),
        }
    }

    /// Check each collection for staleness and re-sync any that have changed.
    ///
    /// Compares the collection directory's mtime against the `.hashes` sidecar
    /// file's mtime. If the directory is newer, files were added or removed
    /// since the last sync. This is one `stat()` per collection — microseconds
    /// for typical vaults. Deep content edits still need explicit `reindex`.
    pub(crate) async fn auto_reindex_stale_collections(&self) {
        let mut stale_collections = Vec::new();

        for collection in self.collections.iter() {
            let hashes_file = crate::store::hashes_path(&self.cache_dir, collection);

            let dir_mtime = collection
                .path
                .metadata()
                .and_then(|m| m.modified())
                .ok();
            let hashes_mtime = hashes_file.metadata().and_then(|m| m.modified()).ok();

            let is_stale = match (dir_mtime, hashes_mtime) {
                (Some(dir_t), Some(hash_t)) => dir_t > hash_t,
                (Some(_), None) => true, // no sidecar yet
                _ => false,
            };

            if is_stale {
                stale_collections.push(collection.name.clone());
            }
        }

        if stale_collections.is_empty() {
            return;
        }

        tracing::info!("auto-reindex: stale collections: {:?}", stale_collections);

        // Rebuild the full index to pick up new/removed documents
        let new_index = crate::index::Index::build(&self.collections);

        for collection in self.collections.iter() {
            if !stale_collections.contains(&collection.name) {
                continue;
            }

            let current_hashes = new_index
                .content_hashes
                .get(&collection.name)
                .cloned()
                .unwrap_or_default();

            match crate::store::sync_collection(
                &self.cache_dir,
                collection,
                &current_hashes,
                &new_index.documents,
                #[cfg(feature = "hybrid")]
                &self.embedder,
            ) {
                Ok((mem, changes)) => {
                    tracing::info!(
                        "auto-reindex: synced '{}' ({} changes)",
                        collection.name,
                        changes
                    );
                    self.search_engine
                        .replace_store(&collection.name, mem);
                }
                Err(e) => {
                    tracing::warn!(
                        "auto-reindex: failed to sync '{}': {}",
                        collection.name,
                        e
                    );
                }
            }
        }

        // Update the in-memory index
        let mut index = self.index.write().await;
        *index = new_index;
    }

    /// Read a document fresh from disk rather than returning indexed content.
    /// This ensures `get_document` never serves stale content — edits are
    /// visible immediately without calling `reindex` first.
    pub(crate) fn read_fresh(&self, doc: &crate::types::Document) -> Option<String> {
        let coll = self.collections.iter().find(|c| c.name == doc.collection)?;
        let file_path = coll.path.join(&doc.path);
        crate::format::read_document_body(&file_path)
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
                 reindex to refresh after adding new files, kb_digest for vault summary \
                 and coverage overview, kb_query to filter by tags/status/dates, \
                 kb_export to export vault as a single markdown document, \
                 and kb_health for document quality diagnostics (missing frontmatter, \
                 stale content, stubs, orphans, broken wiki-links)."
                    .to_string(),
            )
    }
}
