//! MCP tool implementations.
//!
//! One file per tool, each exporting a `router()` function. Routers are
//! composed here with the `+` operator — adding a tool is one new file
//! plus one line in this module.

pub mod context;
pub mod digest;
pub mod documents;
pub mod export;
pub mod health;
pub mod query;
pub mod reindex;
pub mod search;
pub mod sections;
pub mod write;

use crate::server::KbMcpServer;
use rmcp::handler::server::router::tool::ToolRouter;

pub fn combined_router() -> ToolRouter<KbMcpServer> {
    sections::router()
        + documents::router()
        + search::router()
        + context::router()
        + write::router()
        + reindex::router()
        + digest::router()
        + query::router()
        + export::router()
        + health::router()
}
