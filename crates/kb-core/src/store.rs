//! Persistent `.mv2` storage via memvid-core.
//!
//! Each collection gets its own `.mv2` file at `<cache_dir>/<hash>-<name>.mv2`.
//! The hash prefix (first 8 chars of SHA-256 of the absolute collection path)
//! prevents collisions when two projects define collections with the same name.
//! With `index_names: Name` the file is `<cache_dir>/<name>.mv2`, stable across
//! moves; see [`IndexNames`].
//!
//! This module manages the lifecycle: open-or-create, bulk ingest, incremental
//! sync, and search. The `Index` (Vec<Document>) continues to handle metadata
//! operations — memvid-core is the search layer only.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::config::{IndexNames, ResolvedCollection};

/// Derive the `.mv2` file path for a collection.
///
/// By default uses SHA-256 of the absolute collection path (first 8 hex chars)
/// as a prefix to avoid collisions across projects with same-named collections.
#[must_use]
pub fn mv2_path(cache_dir: &Path, collection: &ResolvedCollection) -> PathBuf {
    match collection.index_names {
        IndexNames::Path => {
            let mut hasher = Sha256::new();
            hasher.update(collection.path.to_string_lossy().as_bytes());
            let hash = hex_prefix(&hasher.finalize(), 8);
            cache_dir.join(format!("{}-{}.mv2", hash, collection.name))
        }
        IndexNames::Name => cache_dir.join(format!("{}.mv2", collection.name)),
    }
}

/// Derive the sidecar `.hashes` file path (content hashes for incremental sync).
#[must_use]
pub fn hashes_path(cache_dir: &Path, collection: &ResolvedCollection) -> PathBuf {
    let mv2 = mv2_path(cache_dir, collection);
    mv2.with_extension("hashes")
}

/// Format a URI for a document within a collection.
///
/// URI scheme: `<collection>://<relative-path>` — ensures uniqueness across
/// collections and is parseable back to collection name + filesystem path.
#[must_use]
pub fn document_uri(collection_name: &str, relative_path: &str) -> String {
    format!("{collection_name}://{relative_path}")
}

/// Parse a document URI back into (`collection_name`, `relative_path`).
#[must_use]
pub fn parse_uri(uri: &str) -> Option<(&str, &str)> {
    uri.split_once("://")
}

/// Content hash map: `relative_path` → blake3 hex hash.
/// Serialized as a simple newline-delimited format: `<hash> <path>`.
pub type HashIndex = HashMap<String, String>;

/// Load content hashes from the sidecar file.
#[must_use]
pub fn load_hashes(path: &Path) -> HashIndex {
    let Ok(content) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };

    content
        .lines()
        .filter_map(|line| {
            let (hash, path) = line.split_once(' ')?;
            Some((path.to_string(), hash.to_string()))
        })
        .collect()
}

/// Save content hashes to the sidecar file.
///
/// Written via temp file + rename so a concurrent reader in another process
/// never sees a half-written sidecar.
///
/// # Errors
///
/// The temp file cannot be written or renamed into place.
pub fn save_hashes(path: &Path, hashes: &HashIndex) -> Result<()> {
    let mut lines: Vec<String> = hashes
        .iter()
        .map(|(path, hash)| format!("{hash} {path}"))
        .collect();
    lines.sort();

    let tmp = temp_sibling(path);
    let result = std::fs::write(&tmp, lines.join("\n"))
        .and_then(|()| std::fs::rename(&tmp, path))
        .with_context(|| format!("failed to write hashes: {}", path.display()));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Compute blake3 hash of file content.
#[must_use]
pub fn hash_content(content: &[u8]) -> String {
    let hash = blake3::hash(content);
    hash.to_hex().to_string()
}

/// Delete a collection's `.mv2` store and sidecar hashes so the next
/// `sync_collection` rebuilds it from scratch.
///
/// This is the repair path for a store whose sidecar wrongly claims documents
/// are up to date (e.g. after an interrupted ingest or a binary upgrade that
/// changed indexing) — incremental sync trusts the sidecar and can never
/// re-ingest such documents on its own. Explicit reindex calls this; the
/// startup and auto-resync paths stay incremental.
///
/// # Errors
///
/// The store or its sidecar exists but cannot be removed.
pub fn clear_collection_store(cache_dir: &Path, collection: &ResolvedCollection) -> Result<()> {
    let mv2 = mv2_path(cache_dir, collection);
    let hashes = hashes_path(cache_dir, collection);

    for path in [&mv2, &hashes] {
        if path.exists() {
            std::fs::remove_file(path)
                .with_context(|| format!("failed to remove: {}", path.display()))?;
        }
    }
    Ok(())
}

/// Ensure the cache directory exists.
///
/// # Errors
///
/// The directory cannot be created.
pub fn ensure_cache_dir(cache_dir: &Path) -> Result<()> {
    if !cache_dir.exists() {
        std::fs::create_dir_all(cache_dir)
            .with_context(|| format!("failed to create cache dir: {}", cache_dir.display()))?;
    }
    Ok(())
}

/// Diff result from comparing filesystem hashes against stored sidecar hashes.
pub struct SyncDiff {
    /// Files present on disk but not in the sidecar (or with different hash).
    pub to_add: Vec<String>,
    /// Files in the sidecar but missing from disk.
    pub to_remove: Vec<String>,
    /// True if the .mv2 file didn't exist and everything is new.
    pub is_fresh: bool,
}

/// Compare current filesystem hashes against the stored sidecar to determine
/// which documents need to be added/removed from the .mv2 index.
#[must_use]
pub fn compute_diff(current: &HashIndex, stored: &HashIndex) -> SyncDiff {
    let mut to_add = Vec::new();
    let mut to_remove = Vec::new();

    // New or changed files
    for (path, hash) in current {
        match stored.get(path) {
            Some(stored_hash) if stored_hash == hash => {} // unchanged
            _ => to_add.push(path.clone()),                // new or changed
        }
    }

    // Deleted files
    for path in stored.keys() {
        if !current.contains_key(path) {
            to_remove.push(path.clone());
        }
    }

    SyncDiff {
        is_fresh: stored.is_empty(),
        to_add,
        to_remove,
    }
}

/// Sync a collection's .mv2 file against the filesystem.
///
/// Several servers may share one cache dir, so the live `.mv2` is never
/// written, truncated or deleted in place: changes are built in a private temp
/// file and renamed over it, and the returned handle is always read-only
/// (shared lock). Other processes keep searching their own snapshot until
/// they resync.
///
/// # Errors
///
/// The cache dir cannot be created, or building, committing or opening a
/// store, or saving its sidecar, fails.
pub fn sync_collection(
    cache_dir: &Path,
    collection: &crate::config::ResolvedCollection,
    current_hashes: &HashIndex,
    documents: &[crate::types::Document],
    #[cfg(feature = "hybrid")] embedder: &memvid_core::LocalTextEmbedder,
) -> Result<(memvid_core::Memvid, usize)> {
    ensure_cache_dir(cache_dir)?;

    let mv2 = mv2_path(cache_dir, collection);
    let hashes_file = hashes_path(cache_dir, collection);
    let stored_hashes = load_hashes(&hashes_file);
    let diff = compute_diff(current_hashes, &stored_hashes);
    let changes = diff.to_add.len().saturating_add(diff.to_remove.len());

    if changes == 0 && mv2.exists() {
        match memvid_core::Memvid::open_read_only(&mv2) {
            Ok(mem) => {
                save_hashes(&hashes_file, current_hashes)?;
                return Ok((mem, 0));
            }
            Err(memvid_core::MemvidError::Lock(e)) => tracing::warn!(
                "{} is locked by another process, building a fresh copy: {}",
                mv2.display(),
                e
            ),
            Err(e) => tracing::warn!("unreadable {}, rebuilding: {}", mv2.display(), e),
        }
    }

    let coll_docs = |only: Option<&[String]>| -> Vec<&crate::types::Document> {
        documents
            .iter()
            .filter(|d| d.collection == collection.name)
            .filter(|d| only.is_none_or(|paths| paths.contains(&d.path)))
            .collect()
    };

    // Append to a copy only when the existing store is a valid base for the diff.
    let can_append = changes > 0 && !diff.is_fresh && diff.to_remove.is_empty() && mv2.exists();
    let appended = can_append
        && match build_store(
            &mv2,
            true,
            &collection.name,
            &coll_docs(Some(&diff.to_add)),
            #[cfg(feature = "hybrid")]
            embedder,
        ) {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!("cannot append to {}, rebuilding: {:#}", mv2.display(), e);
                false
            }
        };

    if !appended {
        build_store(
            &mv2,
            false,
            &collection.name,
            &coll_docs(None),
            #[cfg(feature = "hybrid")]
            embedder,
        )?;
    }

    save_hashes(&hashes_file, current_hashes)?;

    let mem = memvid_core::Memvid::open_read_only(&mv2)
        .with_context(|| format!("failed to open .mv2: {}", mv2.display()))?;
    Ok((mem, changes))
}

/// Build a store in a private temp file and atomically rename it over `mv2`.
/// With `from_existing`, the current `mv2` is copied first and `docs` appended.
fn build_store(
    mv2: &Path,
    from_existing: bool,
    collection_name: &str,
    docs: &[&crate::types::Document],
    #[cfg(feature = "hybrid")] embedder: &memvid_core::LocalTextEmbedder,
) -> Result<()> {
    let tmp = temp_sibling(mv2);
    let result = (|| -> Result<()> {
        let mut mem = if from_existing {
            std::fs::copy(mv2, &tmp)
                .with_context(|| format!("failed to copy .mv2: {}", mv2.display()))?;
            memvid_core::Memvid::open(&tmp)
                .with_context(|| format!("failed to open .mv2 copy: {}", tmp.display()))?
        } else {
            memvid_core::Memvid::create(&tmp)
                .with_context(|| format!("failed to create .mv2: {}", tmp.display()))?
        };

        ingest_docs(
            &mut mem,
            collection_name,
            docs,
            #[cfg(feature = "hybrid")]
            embedder,
        )?;
        mem.commit()
            .with_context(|| format!("failed to commit .mv2: {}", tmp.display()))?;
        drop(mem);

        std::fs::rename(&tmp, mv2)
            .with_context(|| format!("failed to replace .mv2: {}", mv2.display()))
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Unique path next to `path`, so concurrent writers (other processes, or
/// threads in this one) never share a temp file. Keeps the `.mv2` extension.
fn temp_sibling(path: &Path) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("tmp");
    path.with_extension(format!("{}-{}.tmp.{}", std::process::id(), n, ext))
}

/// Batch-ingest documents into a Memvid handle. When hybrid feature is
/// enabled, each document is embedded via ONNX for vector search.
fn ingest_docs(
    mem: &mut memvid_core::Memvid,
    collection_name: &str,
    docs: &[&crate::types::Document],
    #[cfg(feature = "hybrid")] embedder: &memvid_core::LocalTextEmbedder,
) -> Result<()> {
    if docs.is_empty() {
        return Ok(());
    }

    mem.begin_batch(memvid_core::PutManyOpts {
        skip_sync: true,
        compression_level: 1,
        wal_pre_size_bytes: 4 * 1024 * 1024,
        ..Default::default()
    })
    .context("failed to begin batch")?;

    for doc in docs {
        let uri = document_uri(collection_name, &doc.path);
        let options = memvid_core::PutOptions::builder()
            .uri(&uri)
            .title(&doc.title)
            .search_text(&doc.body)
            .build();

        #[cfg(feature = "hybrid")]
        {
            use memvid_core::EmbeddingProvider;
            let embedding = embedder
                .embed_text(&doc.body)
                .with_context(|| format!("failed to embed: {}", doc.path))?;
            mem.put_with_embedding_and_options(doc.body.as_bytes(), embedding, options)
                .with_context(|| format!("failed to ingest: {}", doc.path))?;
        }

        #[cfg(not(feature = "hybrid"))]
        {
            mem.put_bytes_with_options(doc.body.as_bytes(), options)
                .with_context(|| format!("failed to ingest: {}", doc.path))?;
        }
    }

    mem.end_batch().context("failed to end batch")?;

    Ok(())
}

fn hex_prefix(bytes: &[u8], len: usize) -> String {
    bytes
        .iter()
        .take(len.div_ceil(2))
        .fold(String::new(), |mut hex, b| {
            // Writing to a String cannot fail.
            let _ = write!(hex, "{b:02x}");
            hex
        })
        .chars()
        .take(len)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collection(path: &str, index_names: IndexNames) -> ResolvedCollection {
        ResolvedCollection {
            name: "wiki".to_string(),
            path: PathBuf::from(path),
            description: String::new(),
            writable: false,
            sections: vec![],
            index_names,
        }
    }

    #[test]
    fn path_names_differ_per_location() {
        let cache = Path::new("/cache");
        let v1 = mv2_path(cache, &collection("/plugins/1.0/wiki", IndexNames::Path));
        let v2 = mv2_path(cache, &collection("/plugins/1.1/wiki", IndexNames::Path));
        assert_ne!(v1, v2);
        assert!(
            v1.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with("-wiki.mv2")
        );
    }

    #[test]
    fn name_names_are_stable_across_moves() {
        let cache = Path::new("/cache");
        let v1 = collection("/plugins/1.0/wiki", IndexNames::Name);
        let v2 = collection("/plugins/1.1/wiki", IndexNames::Name);
        assert_eq!(mv2_path(cache, &v1), PathBuf::from("/cache/wiki.mv2"));
        assert_eq!(mv2_path(cache, &v1), mv2_path(cache, &v2));
        assert_eq!(hashes_path(cache, &v2), PathBuf::from("/cache/wiki.hashes"));
    }

    #[test]
    fn test_document_uri() {
        assert_eq!(
            document_uri("vault", "concepts/mcp.md"),
            "vault://concepts/mcp.md"
        );
    }

    #[test]
    fn test_parse_uri() {
        let (coll, path) = parse_uri("vault://concepts/mcp.md").unwrap();
        assert_eq!(coll, "vault");
        assert_eq!(path, "concepts/mcp.md");
    }

    #[test]
    fn test_parse_uri_none() {
        assert!(parse_uri("no-scheme-here").is_none());
    }

    #[test]
    fn test_hash_content() {
        let hash = hash_content(b"hello world");
        assert_eq!(hash.len(), 64); // blake3 hex is 64 chars
    }

    #[test]
    fn test_hex_prefix() {
        let bytes = [0xab, 0xcd, 0xef, 0x12, 0x34];
        assert_eq!(hex_prefix(&bytes, 8), "abcdef12");
        assert_eq!(hex_prefix(&bytes, 4), "abcd");
    }

    #[test]
    fn test_hashes_roundtrip() {
        let dir = std::env::temp_dir().join("kb-mcp-test-hashes");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.hashes");

        let mut hashes = HashIndex::new();
        hashes.insert("concepts/mcp.md".into(), "abc123".into());
        hashes.insert("patterns/keeper.md".into(), "def456".into());

        save_hashes(&path, &hashes).unwrap();
        let loaded = load_hashes(&path);

        assert_eq!(loaded.get("concepts/mcp.md").unwrap(), "abc123");
        assert_eq!(loaded.get("patterns/keeper.md").unwrap(), "def456");

        std::fs::remove_dir_all(&dir).ok();
    }
}
