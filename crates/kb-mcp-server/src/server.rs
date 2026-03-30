//! MCP server implementation.
//!
//! Holds shared state (index, search engine, collections) behind Arc wrappers
//! and wires tool routers into the rmcp `ServerHandler` trait. The `Clone` derive
//! is required by rmcp — it clones the server for each request, but all data
//! lives behind Arcs so cloning is cheap.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use rmcp::ServerHandler;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use tokio::sync::RwLock;

use kb_core::config::ResolvedCollection;
use kb_core::index::Index;
use kb_core::search::SearchEngine;

use crate::tools;

/// Minimum seconds between auto-reindex checks. Prevents redundant filesystem
/// walks when agents fire multiple tool calls in rapid succession.
const AUTO_REINDEX_COOLDOWN_SECS: u64 = 5;

#[derive(Clone)]
pub struct KbMcpServer {
    pub(crate) index: Arc<RwLock<Index>>,
    pub(crate) search_engine: Arc<SearchEngine>,
    pub(crate) collections: Arc<Vec<ResolvedCollection>>,
    pub(crate) cache_dir: std::path::PathBuf,
    #[cfg(feature = "hybrid")]
    pub(crate) embedder: Arc<memvid_core::LocalTextEmbedder>,
    /// Epoch seconds of the last auto-reindex. Used for debounce.
    last_auto_reindex: Arc<AtomicU64>,
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
            last_auto_reindex: Arc::new(AtomicU64::new(0)),
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
        // Debounce: skip if less than COOLDOWN seconds since last check
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let last = self.last_auto_reindex.load(Ordering::Relaxed);
        if now.saturating_sub(last) < AUTO_REINDEX_COOLDOWN_SECS {
            return;
        }
        self.last_auto_reindex.store(now, Ordering::Relaxed);

        let mut stale_collections = Vec::new();

        for collection in self.collections.iter() {
            let hashes_file = kb_core::store::hashes_path(&self.cache_dir, collection);

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

        let mut index = self.index.write().await;

        for collection in self.collections.iter() {
            if !stale_collections.contains(&collection.name) {
                continue;
            }

            index.rebuild_collection(collection, &self.collections);

            let current_hashes = index
                .content_hashes
                .get(&collection.name)
                .cloned()
                .unwrap_or_default();

            match kb_core::store::sync_collection(
                &self.cache_dir,
                collection,
                &current_hashes,
                &index.documents,
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
    }

    /// Read a document fresh from disk rather than returning indexed content.
    pub(crate) fn read_fresh(&self, doc: &kb_core::types::Document) -> Option<String> {
        let coll = self.collections.iter().find(|c| c.name == doc.collection)?;
        let file_path = coll.path.join(&doc.path);
        kb_core::format::read_document_body(&file_path)
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
