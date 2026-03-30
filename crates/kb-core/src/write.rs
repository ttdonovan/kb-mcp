//! Document writing utilities — shared between CLI and MCP write commands.
//!
//! Extracted from the MCP write tool to unify the duplicated logic that
//! existed in both `tools/write.rs` and `cli.rs`.

use std::path::{Path, PathBuf};

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
pub fn find_available_path(dir: &Path, base_name: &str) -> PathBuf {
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
