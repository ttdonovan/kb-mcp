use rmcp::model::CallToolResult;

use crate::server::KbMcpServer;

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::reindex_router()
}

#[rmcp::tool_router(router = reindex_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "reindex",
        description = "Rebuild the search index from all collections on disk. Use this after adding or editing documents mid-session."
    )]
    pub(crate) async fn reindex(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let new_index = crate::index::Index::build(&self.collections);
        let doc_count = new_index.documents.len();
        let section_count = new_index.sections.len();

        self.search_engine.rebuild(&new_index.documents);

        {
            let mut index = self.index.write().await;
            *index = new_index;
        }

        let msg = format!(
            "Reindexed {} documents across {} sections",
            doc_count, section_count
        );
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            msg,
        )]))
    }
}
