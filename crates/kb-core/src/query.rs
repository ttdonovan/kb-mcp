//! Structured frontmatter queries — filter documents by metadata.
//!
//! Extracted from the MCP query tool so both CLI and MCP can share
//! the same filtering logic without duplication.

use crate::format;
use crate::types::Document;

/// Check if a document matches all query filters (AND logic).
///
/// Both the CLI and MCP tool call this function with loose arguments
/// rather than a shared params struct — each transport defines its own
/// params type (clap args vs JsonSchema struct).
pub fn matches_query(
    doc: &Document,
    collection: Option<&str>,
    tag: Option<&str>,
    status: Option<&str>,
    created_after: Option<&str>,
    has_sources: bool,
) -> bool {
    if collection.is_some_and(|c| doc.collection != c) {
        return false;
    }

    if let Some(tag) = tag {
        let tag_lower = tag.to_lowercase();
        if !doc.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
            return false;
        }
    }

    if let Some(status) = status {
        let has_status = doc
            .frontmatter
            .get("status")
            .and_then(|v| match v {
                serde_yaml::Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .is_some_and(|s| s == status);
        if !has_status {
            return false;
        }
    }

    if let Some(after) = created_after
        && let Ok(after_date) = chrono::NaiveDate::parse_from_str(after, "%Y-%m-%d")
    {
        let created = doc
            .frontmatter
            .get("created")
            .and_then(|v| {
                let s = format::yaml_value_to_string(v);
                chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()
            });
        match created {
            Some(d) if d >= after_date => {}
            _ => return false,
        }
    }

    if has_sources && !doc.frontmatter.contains_key("sources") {
        return false;
    }

    true
}
