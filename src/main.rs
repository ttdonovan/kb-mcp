//! kb-mcp — MCP server + CLI for markdown knowledge bases.
//!
//! Dual-mode binary: runs as an MCP stdio server when invoked with no arguments
//! (as MCP clients like Claude Code expect), or as a CLI when given subcommands
//! (for testing, scripting, and debugging without an MCP client).

mod cli;
mod config;
mod format;
mod index;
mod search;
mod server;
mod tools;
mod types;

use clap::Parser;
use rmcp::ServiceExt;
use rmcp::transport::io::stdio;

use crate::config::load_config;
use crate::index::Index;
use crate::search::SearchEngine;
use crate::server::KbMcpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Detect mode by arg count — MCP clients invoke with zero args.
    if std::env::args().len() > 1 {
        let parsed = cli::Cli::parse();
        let config = load_config(parsed.config.as_deref())?;

        let collections = config.collections.clone();
        let index = Index::build(&config.collections);
        let search_engine = SearchEngine::build(&index.documents);

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
    let index = Index::build(&config.collections);
    let search_engine = SearchEngine::build(&index.documents);

    tracing::info!(
        "kb-mcp ready: {} documents across {} sections",
        index.documents.len(),
        index.sections.len()
    );

    let server = KbMcpServer::new(index, search_engine, collections);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
