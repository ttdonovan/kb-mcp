# Adding Tools

## Steps

1. Create `crates/kb-mcp-server/src/tools/my_tool.rs`
2. Add `pub(crate) mod my_tool;` to `crates/kb-mcp-server/src/tools/mod.rs`
3. Add `+ my_tool::router()` to the `combined_router()` function
4. Add a CLI subcommand in `crates/kb-cli/src/main.rs`
5. If shared logic is needed, add it to `crates/kb-core/src/`
6. Update server instructions in `crates/kb-mcp-server/src/server.rs`

## Tool Template

```rust
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MyToolParams {
    /// Description shown in tool schema
    pub query: String,
}

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::my_tool_router()
}

#[rmcp::tool_router(router = my_tool_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "my_tool",
        description = "What this tool does."
    )]
    pub(crate) async fn my_tool(
        &self,
        Parameters(params): Parameters<MyToolParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        // Call kb_core functions for shared logic
        Ok(CallToolResult::success(vec![
            rmcp::model::Content::text("result"),
        ]))
    }
}
```

## Key Points

- Params struct must derive `Deserialize` + `JsonSchema`
- Use `#[schemars(description = "...")]` for field descriptions in the tool schema
- Use `#[serde(default)]` for optional fields
- Return `CallToolResult::error(...)` with actionable messages for user-facing errors
- Every tool must have a corresponding CLI subcommand for testing parity
- Shared logic (filtering, formatting, utilities) belongs in `kb-core`, not in tool files
