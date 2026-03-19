//! Smoke test for memvid-core API — verifies create, put, commit, open, search lifecycle.

use memvid_core::{AclEnforcementMode, Memvid, PutOptions, SearchRequest};

fn make_search_request(query: &str, top_k: usize) -> SearchRequest {
    SearchRequest {
        query: query.to_string(),
        top_k,
        snippet_chars: 200,
        uri: None,
        scope: None,
        cursor: None,
        as_of_frame: None,
        as_of_ts: None,
        no_sketch: false,
        acl_context: None,
        acl_enforcement_mode: AclEnforcementMode::Audit,
    }
}

#[test]
fn memvid_create_put_search_lifecycle() {
    let dir = std::env::temp_dir().join("kb-mcp-memvid-smoke");
    std::fs::create_dir_all(&dir).unwrap();
    let mv2_path = dir.join("test.mv2");
    let _ = std::fs::remove_file(&mv2_path);

    // Create and add a document
    {
        let mut mem = Memvid::create(&mv2_path).expect("create failed");

        let options = PutOptions::builder()
            .uri("test://concepts/memory.md")
            .title("Cognitive Memory Model")
            .build();

        mem.put_bytes_with_options(
            b"AI agents benefit from memory systems modeled on human cognition.",
            options,
        )
        .expect("put_bytes_with_options failed");

        mem.commit().expect("commit failed");
    }

    // Reopen and search
    {
        let mut mem = Memvid::open(&mv2_path).expect("open failed");

        let resp = mem
            .search(make_search_request("memory cognition", 5))
            .expect("search failed");

        assert!(!resp.hits.is_empty(), "expected at least one search hit");

        let hit = &resp.hits[0];
        assert_eq!(hit.uri, "test://concepts/memory.md");
        assert!(hit.score.unwrap_or(0.0) > 0.0);
    }

    // Verify read-only access works
    {
        let mut mem = Memvid::open_read_only(&mv2_path).expect("open_read_only failed");

        let resp = mem
            .search(make_search_request("memory", 5))
            .expect("read-only search failed");

        assert!(!resp.hits.is_empty());
    }

    std::fs::remove_dir_all(&dir).ok();
}
