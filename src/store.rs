//! Persistent `.mv2` storage via memvid-core.
//!
//! Each collection gets its own `.mv2` file at `<cache_dir>/<hash>-<name>.mv2`.
//! The hash prefix (first 8 chars of SHA-256 of the absolute collection path)
//! prevents collisions when two projects define collections with the same name.
//!
//! This module manages the lifecycle: open-or-create, bulk ingest, incremental
//! sync, and search. The `Index` (Vec<Document>) continues to handle metadata
//! operations — memvid-core is the search layer only.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::config::ResolvedCollection;

/// Derive the `.mv2` file path for a collection.
///
/// Uses SHA-256 of the absolute collection path (first 8 hex chars) as a
/// prefix to avoid collisions across projects with same-named collections.
pub fn mv2_path(cache_dir: &Path, collection: &ResolvedCollection) -> PathBuf {
    let hash = {
        let mut hasher = Sha256::new();
        hasher.update(collection.path.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        hex_prefix(&result, 8)
    };

    cache_dir.join(format!("{}-{}.mv2", hash, collection.name))
}

/// Derive the sidecar `.hashes` file path (content hashes for incremental sync).
pub fn hashes_path(cache_dir: &Path, collection: &ResolvedCollection) -> PathBuf {
    let mv2 = mv2_path(cache_dir, collection);
    mv2.with_extension("hashes")
}

/// Format a URI for a document within a collection.
///
/// URI scheme: `<collection>://<relative-path>` — ensures uniqueness across
/// collections and is parseable back to collection name + filesystem path.
pub fn document_uri(collection_name: &str, relative_path: &str) -> String {
    format!("{}://{}", collection_name, relative_path)
}

/// Parse a document URI back into (collection_name, relative_path).
pub fn parse_uri(uri: &str) -> Option<(&str, &str)> {
    uri.split_once("://")
}

/// Content hash map: relative_path → blake3 hex hash.
/// Serialized as a simple newline-delimited format: `<hash> <path>`.
pub type HashIndex = HashMap<String, String>;

/// Load content hashes from the sidecar file.
pub fn load_hashes(path: &Path) -> HashIndex {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
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
pub fn save_hashes(path: &Path, hashes: &HashIndex) -> Result<()> {
    let mut lines: Vec<String> = hashes
        .iter()
        .map(|(path, hash)| format!("{} {}", hash, path))
        .collect();
    lines.sort();

    std::fs::write(path, lines.join("\n"))
        .with_context(|| format!("failed to write hashes: {}", path.display()))
}

/// Compute blake3 hash of file content.
pub fn hash_content(content: &[u8]) -> String {
    let hash = blake3::hash(content);
    hash.to_hex().to_string()
}

/// Ensure the cache directory exists.
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
/// Opens or creates the .mv2 file, computes the diff against the sidecar,
/// and ingests new/changed documents. When an embedder is provided (hybrid
/// feature), documents are embedded for vector search alongside BM25.
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

    // Open or create .mv2
    let mut mem = if mv2.exists() && !diff.is_fresh {
        match memvid_core::Memvid::open(&mv2) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("corrupted .mv2, rebuilding: {}", e);
                let _ = std::fs::remove_file(&mv2);
                memvid_core::Memvid::create(&mv2)
                    .with_context(|| format!("failed to create .mv2: {}", mv2.display()))?
            }
        }
    } else {
        memvid_core::Memvid::create(&mv2)
            .with_context(|| format!("failed to create .mv2: {}", mv2.display()))?
    };

    let changes = diff.to_add.len() + diff.to_remove.len();

    if changes == 0 {
        save_hashes(&hashes_file, current_hashes)?;
        return Ok((mem, 0));
    }

    let needs_full_rebuild = !diff.to_remove.is_empty() && !diff.is_fresh;

    if needs_full_rebuild {
        drop(mem);
        let _ = std::fs::remove_file(&mv2);
        mem = memvid_core::Memvid::create(&mv2)
            .with_context(|| format!("failed to recreate .mv2: {}", mv2.display()))?;

        let coll_docs: Vec<&crate::types::Document> = documents
            .iter()
            .filter(|d| d.collection == collection.name)
            .collect();

        ingest_docs(
            &mut mem,
            &collection.name,
            &coll_docs,
            #[cfg(feature = "hybrid")]
            embedder,
        )?;
    } else {
        let docs_to_add: Vec<&crate::types::Document> = documents
            .iter()
            .filter(|d| d.collection == collection.name && diff.to_add.contains(&d.path))
            .collect();

        if !docs_to_add.is_empty() {
            ingest_docs(
                &mut mem,
                &collection.name,
                &docs_to_add,
                #[cfg(feature = "hybrid")]
                embedder,
            )?;
        }
    }

    mem.commit()
        .with_context(|| format!("failed to commit .mv2: {}", mv2.display()))?;

    save_hashes(&hashes_file, current_hashes)?;

    Ok((mem, changes))
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
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
        .chars()
        .take(len)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
