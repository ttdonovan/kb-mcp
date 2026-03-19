//! Core domain types shared across modules.
//!
//! These are the in-memory representations built during indexing. They are
//! intentionally decoupled from the RON config types (`CollectionDef`) and
//! the JSON output types (`format.rs`) to keep each concern independent.

use serde::Serialize;

/// A parsed markdown document. Built during filesystem scanning and held
/// in the `Index` for the lifetime of the process.
#[derive(Debug, Clone, Serialize)]
pub struct Document {
    /// Path relative to the collection root (e.g. "concepts/mcp-server-pattern.md").
    pub path: String,
    /// Extracted from the first H1 heading, falling back to the filename.
    pub title: String,
    pub tags: Vec<String>,
    /// Markdown body with frontmatter stripped.
    pub body: String,
    /// First directory component — used for section grouping and scope filtering.
    pub section: String,
    /// Which collection this document belongs to.
    pub collection: String,
    /// All frontmatter fields preserved as-is for `kb_context` output.
    /// Kept separate from `tags` because agents may need arbitrary metadata
    /// (status, source, created date) without reading the full body.
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub frontmatter: std::collections::HashMap<String, serde_yaml::Value>,
}

/// Derived from document counts + RON section definitions.
#[derive(Debug, Clone, Serialize)]
pub struct Section {
    pub name: String,
    pub description: String,
    pub doc_count: usize,
    pub collection: String,
}
