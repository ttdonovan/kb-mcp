//! `kb_write` must never write outside its collection: a `filename` or
//! `directory` that climbs out, is absolute, or goes through a symlink is
//! refused before anything is created.

#![expect(
    clippy::unwrap_used,
    reason = "a test: a failed setup step should fail it"
)]

use std::path::PathBuf;

use kb_core::config::{Exclude, IndexNames, ResolvedCollection};
use kb_core::write::{file_name, find_available_path, target_dir, write_new};

/// A writable collection `notes` inside `root`, with `root/outside/` beside it.
fn fixture(test_name: &str) -> (PathBuf, ResolvedCollection) {
    let root = std::env::temp_dir().join(format!("kb-mcp-write-paths-{test_name}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("notes")).unwrap();
    std::fs::create_dir_all(root.join("outside")).unwrap();
    let root = root.canonicalize().unwrap();
    let collection = ResolvedCollection {
        name: "notes".to_string(),
        path: root.join("notes"),
        description: "Notes".to_string(),
        writable: true,
        sections: vec![],
        index_names: IndexNames::Path,
        exclude: Exclude::default(),
    };
    (root, collection)
}

#[test]
fn filename_must_be_a_plain_name() {
    for bad in [
        "../x.md",
        "../../x.md",
        "/tmp/x.md",
        "sub/x.md",
        "..",
        ".",
        "",
    ] {
        assert!(file_name(Some(bad), "T", "2026-01-01").is_err(), "{bad:?}");
    }
    assert_eq!(file_name(Some("Note.MD"), "T", "d").unwrap(), "Note.MD");
    assert_eq!(file_name(Some("note"), "T", "d").unwrap(), "note.md");
    assert_eq!(
        file_name(None, "My Note", "2026-01-01").unwrap(),
        "2026-01-01-my-note.md"
    );
}

#[test]
fn directory_that_escapes_is_refused_before_creating_anything() {
    let (root, collection) = fixture("dir-escape");
    for bad in [
        "..",
        "../outside/new",
        "a/../../outside/new",
        "/tmp/kb-mcp-abs",
    ] {
        let err = target_dir(&collection, Some(bad)).unwrap_err();
        assert!(err.contains("escapes the collection root"), "{bad}: {err}");
    }
    assert!(
        !root.join("outside/new").exists(),
        "nothing created outside"
    );
    assert!(
        !root.join("notes/a").exists(),
        "nothing created inside either"
    );

    let ok = target_dir(&collection, Some("concepts/memory")).unwrap();
    assert_eq!(ok, collection.path.join("concepts/memory"));
    assert!(ok.is_dir());
    assert_eq!(target_dir(&collection, None).unwrap(), collection.path);
    std::fs::remove_dir_all(&root).ok();
}

#[cfg(unix)]
#[test]
fn symlinks_out_of_the_collection_are_not_followed() {
    let (root, collection) = fixture("symlink");
    let outside = root.join("outside");

    // A symlinked directory pointing out is refused, and nothing is created through it.
    std::os::unix::fs::symlink(&outside, collection.path.join("link")).unwrap();
    assert!(target_dir(&collection, Some("link/new")).is_err());
    assert!(!outside.join("new").exists());

    // A dangling symlink named like the target file counts as taken.
    std::os::unix::fs::symlink(outside.join("victim.md"), collection.path.join("note.md")).unwrap();
    assert_eq!(
        find_available_path(&collection.path, "note.md"),
        Ok(collection.path.join("note-2.md"))
    );

    // Writing never follows a link, even one planted after the name was chosen.
    let err = write_new(&collection.path.join("note.md"), "x").unwrap_err();
    assert!(err.contains("Failed to write file"), "{err}");
    assert!(!outside.join("victim.md").exists());

    // A symlinked directory that stays inside the collection still works.
    std::fs::create_dir_all(collection.path.join("real")).unwrap();
    std::os::unix::fs::symlink(collection.path.join("real"), collection.path.join("inlink"))
        .unwrap();
    assert_eq!(
        target_dir(&collection, Some("inlink/new")).unwrap(),
        collection.path.join("real/new")
    );

    // A dangling link in `directory` is refused as an escape, not a mkdir error.
    std::os::unix::fs::symlink(outside.join("gone"), collection.path.join("dangling")).unwrap();
    let err = target_dir(&collection, Some("dangling/new")).unwrap_err();
    assert!(err.contains("escapes the collection root"), "{err}");
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn no_free_name_is_an_error_not_a_taken_path() {
    let (root, collection) = fixture("exhausted");
    std::fs::write(collection.path.join("n.md"), "keep").unwrap();
    for i in 2..100 {
        std::fs::write(collection.path.join(format!("n-{i}.md")), "keep").unwrap();
    }
    let err = find_available_path(&collection.path, "n.md").unwrap_err();
    assert!(err.contains("No free file name for 'n.md'"), "{err}");

    let path = collection.path.join("n.md");
    assert!(write_new(&path, "new").is_err());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "keep",
        "not overwritten"
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn missing_collection_directory_is_reported_as_such() {
    let (root, mut collection) = fixture("missing-root");
    collection.path = root.join("not-yet");
    let err = target_dir(&collection, Some("sub")).unwrap_err();
    assert!(err.contains("does not exist"), "{err}");
    assert!(!root.join("not-yet").exists());
    std::fs::remove_dir_all(&root).ok();
}
