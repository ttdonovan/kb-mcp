use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use std::fmt::Write as _;

use crate::server::KbMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteParams {
    /// Target collection name (must be writable)
    pub collection: String,
    /// Document title
    pub title: String,
    /// Tags for the document
    #[serde(default)]
    pub tags: Vec<String>,
    /// Document body content (markdown)
    pub body: String,
    /// Optional status field for frontmatter
    #[serde(default)]
    pub status: Option<String>,
    /// Optional source field for frontmatter
    #[serde(default)]
    pub source: Option<String>,
    /// Optional subdirectory within the collection (e.g. "concepts/memory" or
    /// "research/papers"), relative and without "..". Created automatically if it doesn't exist. Files
    /// written here appear under the corresponding section in `list_sections`.
    #[serde(default)]
    pub directory: Option<String>,
    /// Optional filename (e.g. "cognitive-memory-model.md"), a plain file name, without path separators. When omitted, a
    /// date-prefixed slug is generated from the title. Use this for structured
    /// hierarchies where date prefixes don't fit.
    #[serde(default)]
    pub filename: Option<String>,
}

pub fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::write_router()
}

#[rmcp::tool_router(router = write_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "kb_write",
        description = "Create a new document in a writable collection. Generates frontmatter with date-prefixed filename by default. Use 'directory' to write into subdirectories (created automatically) and 'filename' to specify an exact filename without date prefix. Only works on collections marked writable in the configuration."
    )]
    pub(crate) async fn kb_write(
        &self,
        Parameters(params): Parameters<WriteParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let collection = self
            .collections
            .iter()
            .find(|c| c.name == params.collection);

        let Some(collection) = collection else {
            let available: Vec<&str> = self.collections.iter().map(|c| c.name.as_str()).collect();
            return Ok(CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(format!(
                    "Collection '{}' not found. Available: {}",
                    params.collection,
                    available.join(", ")
                )),
            ]));
        };

        if !collection.writable {
            let writable: Vec<&str> = self
                .collections
                .iter()
                .filter(|c| c.writable)
                .map(|c| c.name.as_str())
                .collect();
            let hint = if writable.is_empty() {
                "No writable collections configured.".to_string()
            } else {
                format!("Writable collections: {}", writable.join(", "))
            };
            return Ok(CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(format!(
                    "Collection '{}' is read-only. {}",
                    params.collection, hint
                )),
            ]));
        }

        let target_dir = match kb_core::write::target_dir(collection, params.directory.as_deref()) {
            Ok(dir) => dir,
            Err(message) => {
                return Ok(CallToolResult::error(vec![
                    rmcp::model::ContentBlock::text(message),
                ]));
            }
        };

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let base_name =
            match kb_core::write::file_name(params.filename.as_deref(), &params.title, &today) {
                Ok(name) => name,
                Err(message) => {
                    return Ok(CallToolResult::error(vec![
                        rmcp::model::ContentBlock::text(message),
                    ]));
                }
            };

        let file_path = match kb_core::write::find_available_path(&target_dir, &base_name) {
            Ok(path) => path,
            Err(message) => {
                return Ok(CallToolResult::error(vec![
                    rmcp::model::ContentBlock::text(message),
                ]));
            }
        };
        if let Err(message) = kb_core::write::check_not_excluded(collection, &file_path) {
            return Ok(CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(message),
            ]));
        }
        let rel_path = file_path
            .strip_prefix(&collection.path)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        // Writing to a String cannot fail.
        let mut frontmatter = String::from("---\n");
        if !params.tags.is_empty() {
            let _ = writeln!(frontmatter, "tags: [{}]", params.tags.join(", "));
        }
        let _ = writeln!(frontmatter, "created: {today}");
        let _ = writeln!(frontmatter, "updated: {today}");
        if let Some(status) = &params.status {
            let _ = writeln!(frontmatter, "status: {status}");
        }
        if let Some(source) = &params.source {
            let _ = writeln!(frontmatter, "source: {source}");
        }
        frontmatter.push_str("---\n\n");

        let content = format!("{}# {}\n\n{}\n", frontmatter, params.title, params.body);

        if let Err(message) = kb_core::write::write_new(&file_path, &content) {
            return Ok(CallToolResult::error(vec![
                rmcp::model::ContentBlock::text(message),
            ]));
        }

        let new_index = kb_core::index::Index::build(&self.collections);

        let current_hashes = new_index
            .content_hashes
            .get(&params.collection)
            .cloned()
            .unwrap_or_default();

        if let Ok((mem, _)) = kb_core::store::sync_collection(
            &self.cache_dir,
            collection,
            &current_hashes,
            &new_index.documents,
            #[cfg(feature = "hybrid")]
            &self.embedder,
        ) {
            self.search_engine.replace_store(&params.collection, mem);
        }

        {
            let mut index = self.index.write().await;
            *index = new_index;
        }

        let json = kb_core::format::format_write(
            &rel_path,
            &params.collection,
            &params.title,
            &params.tags,
        );
        Ok(CallToolResult::success(vec![
            rmcp::model::ContentBlock::text(json),
        ]))
    }
}
