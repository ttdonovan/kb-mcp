//! Regression tests for two servers sharing one cache dir (issue #2).
//!
//! flock locks are per open file description, so two handles in one process
//! contend exactly like two processes. Each sync must neither wait on another
//! handle's lock nor mutate the file that handle has open.

#![cfg(not(feature = "hybrid"))]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test: a failed setup step should fail it"
)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use kb_core::config::{IndexNames, ResolvedCollection};
use kb_core::index::Index;
use kb_core::search::SearchEngine;
use kb_core::store;

/// memvid retries a contended lock for ~10 s; anything near that is a wait.
const MAX_SYNC: Duration = Duration::from_secs(3);

fn fixture(test_name: &str, with_doc: bool) -> (PathBuf, PathBuf, ResolvedCollection) {
    let root = std::env::temp_dir().join(format!("kb-mcp-shared-index-{test_name}"));
    let _ = std::fs::remove_dir_all(&root);
    let notes = root.join("notes");
    std::fs::create_dir_all(&notes).unwrap();
    if with_doc {
        std::fs::write(
            notes.join("ownership.md"),
            "# Ownership\n\nThe borrow checker prevents data races.\n",
        )
        .unwrap();
    }

    let collection = ResolvedCollection {
        name: "notes".to_string(),
        path: notes,
        description: "Notes".to_string(),
        writable: false,
        sections: vec![],
        index_names: IndexNames::Path,
    };
    (root.clone(), root.join("cache"), collection)
}

fn sync(
    cache_dir: &std::path::Path,
    collection: &ResolvedCollection,
) -> (Index, memvid_core::Memvid, usize) {
    let index = Index::build(std::slice::from_ref(collection));
    let hashes = index
        .content_hashes
        .get(&collection.name)
        .cloned()
        .unwrap_or_default();
    let start = Instant::now();
    let (mem, changes) = store::sync_collection(cache_dir, collection, &hashes, &index.documents)
        .expect("sync succeeds");
    assert!(
        start.elapsed() < MAX_SYNC,
        "sync waited on a lock: {:?}",
        start.elapsed()
    );
    (index, mem, changes)
}

fn finds(engine: &SearchEngine, index: &Index, query: &str, path: &str) -> bool {
    engine
        .search(&index.documents, query, None, None, 10)
        .iter()
        .any(|r| index.documents[r.doc_index].path == path)
}

#[test]
fn two_handles_on_unchanged_index_both_search() {
    let (root, cache_dir, collection) = fixture("unchanged", true);

    let (index, first, _) = sync(&cache_dir, &collection);
    let (_, second, changes) = sync(&cache_dir, &collection);
    assert_eq!(changes, 0);

    let a = SearchEngine::new(HashMap::from([("notes".to_string(), first)]));
    let b = SearchEngine::new(HashMap::from([("notes".to_string(), second)]));
    assert!(finds(&a, &index, "borrow checker", "ownership.md"));
    assert!(finds(&b, &index, "borrow checker", "ownership.md"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn sync_with_changes_does_not_disturb_open_handle() {
    let (root, cache_dir, collection) = fixture("changes", true);

    let (old_index, first, _) = sync(&cache_dir, &collection);

    std::fs::write(
        collection.path.join("lifetimes.md"),
        "# Lifetimes\n\nLifetimes annotate how long references stay valid.\n",
    )
    .unwrap();
    let (new_index, second, changes) = sync(&cache_dir, &collection);
    assert_eq!(changes, 1);

    // The first handle keeps serving its snapshot; the second sees the new doc.
    let a = SearchEngine::new(HashMap::from([("notes".to_string(), first)]));
    let b = SearchEngine::new(HashMap::from([("notes".to_string(), second)]));
    assert!(finds(&a, &old_index, "borrow checker", "ownership.md"));
    assert!(finds(&b, &new_index, "references valid", "lifetimes.md"));
    assert!(finds(&b, &new_index, "borrow checker", "ownership.md"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn empty_collection_resync_does_not_truncate_open_store() {
    let (root, cache_dir, collection) = fixture("empty", false);

    let (_, _first, _) = sync(&cache_dir, &collection);
    let mv2 = store::mv2_path(&cache_dir, &collection);
    let len = std::fs::metadata(&mv2).unwrap().len();
    assert!(len > 0);

    let (_, _second, changes) = sync(&cache_dir, &collection);
    assert_eq!(changes, 0);
    assert_eq!(std::fs::metadata(&mv2).unwrap().len(), len);

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn server_detects_store_replaced_by_another_and_reloads_without_reingest() {
    let (root, cache_dir, collection) = fixture("replaced", true);
    let mv2 = store::mv2_path(&cache_dir, &collection);

    // Server B loads the store.
    let (_, held, _) = sync(&cache_dir, &collection);
    let b = SearchEngine::new(HashMap::from([("notes".to_string(), held)]));
    assert!(!b.store_replaced("notes", &mv2));

    // Server A syncs a new document.
    std::fs::write(
        collection.path.join("lifetimes.md"),
        "# Lifetimes\n\nLifetimes annotate how long references stay valid.\n",
    )
    .unwrap();
    let (_, _a, changes) = sync(&cache_dir, &collection);
    assert_eq!(changes, 1);
    assert!(b.store_replaced("notes", &mv2));

    // B reloads: A already ingested, so B only reopens.
    let (index, reloaded, changes) = sync(&cache_dir, &collection);
    assert_eq!(changes, 0, "no re-ingest of a store another server built");
    b.replace_store("notes", reloaded);
    assert!(!b.store_replaced("notes", &mv2));
    assert!(finds(&b, &index, "references valid", "lifetimes.md"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn missing_store_with_current_sidecar_is_rebuilt() {
    let (root, cache_dir, collection) = fixture("missing", true);

    let (index, first, _) = sync(&cache_dir, &collection);
    drop(first);
    std::fs::remove_file(store::mv2_path(&cache_dir, &collection)).unwrap();

    let (_, mem, _) = sync(&cache_dir, &collection);
    let engine = SearchEngine::new(HashMap::from([("notes".to_string(), mem)]));
    assert!(finds(&engine, &index, "borrow checker", "ownership.md"));

    std::fs::remove_dir_all(&root).ok();
}
