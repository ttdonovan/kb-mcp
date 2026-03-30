//! kb-mcp — MCP stdio server for markdown knowledge bases.
//!
//! Runs as an MCP stdio server. MCP clients like Claude Code invoke this
//! binary with zero arguments. For CLI usage, use the `kb` binary instead.

mod server;
mod tools;

use rmcp::ServiceExt;
use rmcp::transport::io::stdio;

use crate::server::KbMcpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // MCP mode — stdout is the JSON-RPC transport, so all logs must go to stderr.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let ctx = kb_core::init(None).await?;

    tracing::info!(
        "kb-mcp ready: {} documents across {} sections",
        ctx.index.documents.len(),
        ctx.index.sections.len()
    );

    let server = KbMcpServer::new(
        ctx.index,
        ctx.search_engine,
        ctx.collections,
        ctx.cache_dir,
        #[cfg(feature = "hybrid")]
        ctx.embedder,
    );
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
