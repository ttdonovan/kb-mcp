use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::format;
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
}

pub(crate) fn router() -> rmcp::handler::server::router::tool::ToolRouter<KbMcpServer> {
    KbMcpServer::write_router()
}

#[rmcp::tool_router(router = write_router)]
impl KbMcpServer {
    #[rmcp::tool(
        name = "kb_write",
        description = "Create a new document in a writable collection. Generates proper frontmatter with date-prefixed filename. Only works on collections marked writable in the configuration."
    )]
    pub(crate) async fn kb_write(
        &self,
        Parameters(params): Parameters<WriteParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        // Find the collection
        let collection = self
            .collections
            .iter()
            .find(|c| c.name == params.collection);

        let collection = match collection {
            Some(c) => c,
            None => {
                let available: Vec<&str> = self.collections.iter().map(|c| c.name.as_str()).collect();
                return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                    format!(
                        "Collection '{}' not found. Available: {}",
                        params.collection,
                        available.join(", ")
                    ),
                )]));
            }
        };

        // Check writable
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
            return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                format!(
                    "Collection '{}' is read-only. {}",
                    params.collection, hint
                ),
            )]));
        }

        // Generate filename
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let slug = slugify_title(&params.title);
        let base_name = format!("{}-{}.md", today, slug);

        // Handle collisions
        let file_path = find_available_path(&collection.path, &base_name);
        let rel_path = file_path
            .strip_prefix(&collection.path)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        // Generate frontmatter
        let mut frontmatter = String::new();
        frontmatter.push_str("---\n");
        if !params.tags.is_empty() {
            frontmatter.push_str(&format!(
                "tags: [{}]\n",
                params.tags.join(", ")
            ));
        }
        frontmatter.push_str(&format!("created: {}\n", today));
        frontmatter.push_str(&format!("updated: {}\n", today));
        if let Some(status) = &params.status {
            frontmatter.push_str(&format!("status: {}\n", status));
        }
        if let Some(source) = &params.source {
            frontmatter.push_str(&format!("source: {}\n", source));
        }
        frontmatter.push_str("---\n\n");

        let content = format!("{}# {}\n\n{}\n", frontmatter, params.title, params.body);

        // Write file
        if let Err(e) = std::fs::write(&file_path, &content) {
            return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                format!("Failed to write file: {}", e),
            )]));
        }

        // Rebuild index and sync the collection's .mv2 to include new document
        let new_index = crate::index::Index::build(&self.collections);

        let current_hashes = new_index
            .content_hashes
            .get(&params.collection)
            .cloned()
            .unwrap_or_default();

        if let Ok((mem, _)) = crate::store::sync_collection(
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

        let json = format::format_write(&rel_path, &params.collection, &params.title, &params.tags);
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json,
        )]))
    }
}

/// Convert a title to a URL-safe kebab-case slug for filenames.
pub fn slugify_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Find a non-conflicting filename by appending numeric suffixes (-2, -3, ...).
/// Avoids silently overwriting existing notes when an agent retries a write.
fn find_available_path(dir: &std::path::Path, base_name: &str) -> std::path::PathBuf {
    let candidate = dir.join(base_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = base_name.trim_end_matches(".md");
    for i in 2..100 {
        let candidate = dir.join(format!("{}-{}.md", stem, i));
        if !candidate.exists() {
            return candidate;
        }
    }

    dir.join(base_name)
}
