//! Document writing utilities — shared between CLI and MCP write commands.
//!
//! Extracted from the MCP write tool to unify the duplicated logic that
//! existed in both `tools/write.rs` and `cli.rs`.

use std::path::{Path, PathBuf};

use crate::config::ResolvedCollection;

/// Refuse a target the collection's `exclude` would hide, since the note
/// would be written but never indexed, searchable or fetchable.
///
/// # Errors
///
/// The message to show when `file_path` is excluded.
pub fn check_not_excluded(collection: &ResolvedCollection, file_path: &Path) -> Result<(), String> {
    let Ok(rel) = file_path.strip_prefix(&collection.path) else {
        return Err(format!(
            "'{}' is outside collection '{}'",
            file_path.display(),
            collection.name
        ));
    };
    if collection.exclude.excludes(rel) {
        return Err(format!(
            "'{}' matches an exclude pattern of collection '{}', so it would never be indexed. Choose another directory or filename.",
            rel.display(),
            collection.name
        ));
    }
    Ok(())
}

/// Convert a title to a URL-safe kebab-case slug for filenames.
#[must_use]
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
#[must_use]
pub fn find_available_path(dir: &Path, base_name: &str) -> PathBuf {
    let candidate = dir.join(base_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = base_name.trim_end_matches(".md");
    for i in 2..100 {
        let candidate = dir.join(format!("{stem}-{i}.md"));
        if !candidate.exists() {
            return candidate;
        }
    }

    dir.join(base_name)
}
