//! Document writing utilities — shared between CLI and MCP write commands.
//!
//! Extracted from the MCP write tool to unify the duplicated logic that
//! existed in both `tools/write.rs` and `cli.rs`.

use std::io::Write;
use std::path::{Component, Path, PathBuf};

use crate::config::ResolvedCollection;

/// The directory to write into: the collection root, or `directory` under it,
/// created if missing.
///
/// `directory` must be plain relative segments, and the deepest part that
/// already exists must resolve (through symlinks) inside the collection, both
/// checked before anything is created, so a refused write leaves nothing behind.
///
/// # Errors
///
/// `directory` escapes the collection, or it cannot be created.
pub fn target_dir(
    collection: &ResolvedCollection,
    directory: Option<&str>,
) -> Result<PathBuf, String> {
    let Some(dir) = directory else {
        return Ok(collection.path.clone());
    };
    if !collection.path.is_dir() {
        return Err(format!(
            "Collection directory '{}' does not exist",
            collection.path.display()
        ));
    }
    let escapes = || {
        format!(
            "Directory '{dir}' escapes the collection root. Use a relative path within the collection."
        )
    };
    if !Path::new(dir)
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(escapes());
    }

    let base = collection
        .path
        .canonicalize()
        .unwrap_or_else(|_| collection.path.clone());
    let resolved = collection.path.join(dir);
    // symlink_metadata finds a dangling link too, which then fails to canonicalize.
    let existing = resolved
        .ancestors()
        .find(|p| p.symlink_metadata().is_ok())
        .and_then(|p| p.canonicalize().ok());
    if !existing.is_some_and(|p| p.starts_with(&base)) {
        return Err(escapes());
    }

    std::fs::create_dir_all(&resolved)
        .map_err(|e| format!("Failed to create directory '{dir}': {e}"))?;
    let canonical = resolved.canonicalize().unwrap_or(resolved);
    if !canonical.starts_with(&base) {
        return Err(escapes());
    }
    Ok(canonical)
}

/// The file name to write: `filename` with `.md` added if missing, or a
/// date-prefixed slug of the title.
///
/// # Errors
///
/// `filename` is not a single plain file name (it has a path separator, is
/// `.` or `..`, or is absolute), which would let it leave the target directory.
pub fn file_name(filename: Option<&str>, title: &str, today: &str) -> Result<String, String> {
    let Some(name) = filename else {
        return Ok(format!("{today}-{}.md", slugify_title(title)));
    };
    let mut components = Path::new(name).components();
    if !matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(n)), None) if n == name
    ) {
        return Err(format!(
            "Filename '{name}' must be a plain file name. Use directory for subdirectories."
        ));
    }
    if Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        Ok(name.to_string())
    } else {
        Ok(format!("{name}.md"))
    }
}

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
///
/// Any entry counts as taken, including a dangling symlink.
///
/// # Errors
///
/// The name and its `-2` to `-99` variants are all taken; never a taken path.
pub fn find_available_path(dir: &Path, base_name: &str) -> Result<PathBuf, String> {
    let free = |p: &Path| p.symlink_metadata().is_err();
    let candidate = dir.join(base_name);
    if free(&candidate) {
        return Ok(candidate);
    }

    let stem = base_name.trim_end_matches(".md");
    for i in 2..100 {
        let candidate = dir.join(format!("{stem}-{i}.md"));
        if free(&candidate) {
            return Ok(candidate);
        }
    }

    Err(format!(
        "No free file name for '{base_name}': it and its -2 to -99 variants are taken"
    ))
}

/// Create `path` with `content`, failing if any entry is already there.
///
/// `create_new` (`O_EXCL`) refuses an existing file or symlink, dangling or
/// not, so a link placed after [`find_available_path`] chose the name is not
/// followed and an existing note is never overwritten.
///
/// # Errors
///
/// The message to show when the file exists or cannot be written.
pub fn write_new(path: &Path, content: &str) -> Result<(), String> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| file.write_all(content.as_bytes()))
        .map_err(|e| format!("Failed to write file: {e}"))
}
