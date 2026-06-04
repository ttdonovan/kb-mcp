use std::collections::HashMap;

use rmcp::model::CallToolResult;

use crate::server::KbMcpServer;

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::reindex_router()
}

#[rmcp::tool_router(router = reindex_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "reindex",
        description = "Fully rebuild the search index from all collections on disk, discarding cached state. Use this after adding or editing documents mid-session, or when search results seem stale or incomplete."
    )]
    pub(crate) async fn reindex(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let new_index = kb_core::index::Index::build(&self.collections);
        let doc_count = new_index.documents.len();
        let section_count = new_index.sections.len();

        let mut new_stores = HashMap::new();
        let mut total_changes = 0;

        for collection in self.collections.iter() {
            let current_hashes = new_index
                .content_hashes
                .get(&collection.name)
                .cloned()
                .unwrap_or_default();

            // Explicit reindex forces a full rebuild — clearing the store and
            // sidecar is the only way to repair state where the sidecar
            // wrongly claims documents are already ingested.
            if let Err(e) = kb_core::store::clear_collection_store(&self.cache_dir, collection) {
                return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                    format!("Failed to clear store for '{}': {}", collection.name, e),
                )]));
            }

            match kb_core::store::sync_collection(
                &self.cache_dir,
                collection,
                &current_hashes,
                &new_index.documents,
                #[cfg(feature = "hybrid")]
                &self.embedder,
            ) {
                Ok((mem, changes)) => {
                    total_changes += changes;
                    new_stores.insert(collection.name.clone(), mem);
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                        format!("Failed to sync collection '{}': {}", collection.name, e),
                    )]));
                }
            }
        }

        self.search_engine.replace_all_stores(new_stores);

        {
            let mut index = self.index.write().await;
            *index = new_index;
        }

        let msg = format!(
            "Reindexed {} documents across {} sections ({} changes synced)",
            doc_count, section_count, total_changes
        );
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            msg,
        )]))
    }
}
