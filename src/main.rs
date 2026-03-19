//! kb-mcp — MCP server + CLI for markdown knowledge bases.
//!
//! Dual-mode binary: runs as an MCP stdio server when invoked with no arguments
//! (as MCP clients like Claude Code expect), or as a CLI when given subcommands
//! (for testing, scripting, and debugging without an MCP client).

mod cli;
mod config;
mod format;
mod index;
mod store;
mod search;
mod server;
mod tools;
mod types;

use std::collections::HashMap;

use clap::Parser;
use rmcp::ServiceExt;
use rmcp::transport::io::stdio;

use crate::config::load_config;
use crate::index::Index;
use crate::search::SearchEngine;
use crate::server::KbMcpServer;

/// Sync .mv2 files for all collections against the filesystem.
/// Returns a map of collection_name → Memvid handle for the search engine.
fn sync_stores(
    cache_dir: &std::path::Path,
    index: &Index,
    collections: &[config::ResolvedCollection],
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

        // Collect owned documents for sync_collection (it needs &[Document])
        match store::sync_collection(cache_dir, collection, &current_hashes, &index.documents) {
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
                tracing::error!("failed to sync collection '{}': {}", collection.name, e);
            }
        }
    }

    stores
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Detect mode by arg count — MCP clients invoke with zero args.
    if std::env::args().len() > 1 {
        let parsed = cli::Cli::parse();
        let config = load_config(parsed.config.as_deref())?;

        let collections = config.collections.clone();
        let cache_dir = config.cache_dir.clone();
        let index = Index::build(&config.collections);
        let stores = sync_stores(&cache_dir, &index, &collections);
        let search_engine = SearchEngine::new(stores);

        cli::run(parsed, &index, &search_engine, &collections);
        return Ok(());
    }

    // MCP mode — stdout is the JSON-RPC transport, so all logs must go to stderr.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let config = load_config(None)?;
    let collections = config.collections.clone();
    let cache_dir = config.cache_dir.clone();
    let index = Index::build(&config.collections);
    let stores = sync_stores(&cache_dir, &index, &collections);
    let search_engine = SearchEngine::new(stores);

    tracing::info!(
        "kb-mcp ready: {} documents across {} sections",
        index.documents.len(),
        index.sections.len()
    );

    let server = KbMcpServer::new(index, search_engine, collections, cache_dir);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
