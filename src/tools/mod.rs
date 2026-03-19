//! MCP tool implementations.
//!
//! One file per tool, each exporting a `router()` function. Routers are
//! composed here with the `+` operator — adding a tool is one new file
//! plus one line in this module.

pub(crate) mod context;
pub(crate) mod documents;
pub(crate) mod reindex;
pub(crate) mod search;
pub(crate) mod sections;
pub(crate) mod write;

use crate::server::KbMcpServer;
use rmcp::handler::server::router::tool::ToolRouter;

pub(crate) fn combined_router() -> ToolRouter<KbMcpServer> {
    sections::router()
        + documents::router()
        + search::router()
        + context::router()
        + write::router()
        + reindex::router()
}
