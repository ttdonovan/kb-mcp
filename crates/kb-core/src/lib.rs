//! Core library for kb-mcp — types, configuration, indexing, search, and formatting.
//!
//! This crate contains all shared logic used by both the CLI (`kb`) and the
//! MCP server (`kb-mcp`). It has zero dependency on `rmcp`, `schemars`, or
//! `clap` — those belong in the binary crates that provide the transport layer.

pub mod config;
pub mod format;
pub mod index;
pub mod query;
pub mod search;
pub mod store;
pub mod types;
pub mod write;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::{ResolvedCollection, ResolvedConfig};
use crate::index::Index;
use crate::search::SearchEngine;

/// Bundled application state from initialization.
///
/// Returns owned values so each binary can wrap them as needed:
/// the CLI uses them directly, the MCP server wraps in Arc/RwLock.
pub struct AppContext {
    pub index: Index,
    pub search_engine: SearchEngine,
    pub collections: Vec<ResolvedCollection>,
    pub cache_dir: PathBuf,
    #[cfg(feature = "hybrid")]
    pub embedder: std::sync::Arc<memvid_core::LocalTextEmbedder>,
}

/// Initialize the full application: load config, build index, sync stores,
/// create search engine.
pub async fn init(config_path: Option<&Path>) -> Result<AppContext> {
    let config = config::load_config(config_path)?;
    let ResolvedConfig {
        cache_dir,
        collections,
    } = config;

    store::ensure_cache_dir(&cache_dir)?;
    let index = Index::build(&collections);

    #[cfg(feature = "hybrid")]
    let embedder = std::sync::Arc::new(
        memvid_core::LocalTextEmbedder::new(memvid_core::TextEmbedConfig::default())?,
    );

    let stores = sync_stores(
        &cache_dir,
        &index,
        &collections,
        #[cfg(feature = "hybrid")]
        &embedder,
    );

    let search_engine = SearchEngine::new(
        stores,
        #[cfg(feature = "hybrid")]
        embedder.clone(),
    );

    Ok(AppContext {
        index,
        search_engine,
        collections,
        cache_dir,
        #[cfg(feature = "hybrid")]
        embedder,
    })
}

/// Sync .mv2 files for all collections against the filesystem.
/// Returns a map of collection_name -> Memvid handle for the search engine.
pub fn sync_stores(
    cache_dir: &Path,
    index: &Index,
    collections: &[ResolvedCollection],
    #[cfg(feature = "hybrid")] embedder: &memvid_core::LocalTextEmbedder,
) -> HashMap<String, memvid_core::Memvid> {
    let mut stores = HashMap::new();

    for collection in collections {
        let current_hashes = index
            .content_hashes
            .get(&collection.name)
            .cloned()
            .unwrap_or_default();

        let docs: Vec<&types::Document> = index
            .documents
            .iter()
            .filter(|d| d.collection == collection.name)
            .collect();

        match store::sync_collection(
            cache_dir,
            collection,
            &current_hashes,
            &index.documents,
            #[cfg(feature = "hybrid")]
            embedder,
        ) {
            Ok((mem, changes)) => {
                if changes > 0 {
                    tracing::info!(
                        "synced collection '{}': {} changes ({} docs)",
                        collection.name,
                        changes,
                        docs.len()
                    );
                }
                stores.insert(collection.name.clone(), mem);
            }
            Err(e) => {
                eprintln!("failed to sync collection '{}': {:?}", collection.name, e);
            }
        }
    }

    stores
}
