//! Drives the real `kb-mcp` binary over stdio, as an MCP client does, so an rmcp upgrade that
//! changes the handshake, the tool list or the tool output fails here (issue #12).
//!
//! BM25 build only: a hybrid build needs a local embedding model.

#![cfg(not(feature = "hybrid"))]
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: a failed step or a missing JSON field should fail it"
)]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

const RON: &str = r#"(
    collections: [(
        name: "notes",
        path: "notes",
        description: "Test notes",
        writable: false,
        sections: [(prefix: "guides", description: "How-to guides")],
    )],
)"#;

struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Server {
    fn start(root: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_kb-mcp"))
            .env("KB_MCP_CONFIG", root.join("collections.ron"))
            .env("KB_MCP_CACHE_DIR", root.join("cache"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, message: &Value) {
        writeln!(self.stdin, "{message}").unwrap();
        self.stdin.flush().unwrap();
    }

    /// Send a request and return the response with the same id.
    fn request(&mut self, id: u64, method: &str, params: &Value) -> Value {
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            let mut line = String::new();
            assert!(
                self.stdout.read_line(&mut line).unwrap() > 0,
                "server closed stdout"
            );
            let message: Value = serde_json::from_str(&line).unwrap();
            if message["id"] == json!(id) {
                return message;
            }
        }
    }

    fn call(&mut self, id: u64, tool: &str, arguments: &Value) -> Value {
        let response = self.request(
            id,
            "tools/call",
            &json!({"name": tool, "arguments": arguments}),
        );
        response["result"].clone()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn setup() -> PathBuf {
    let root = std::env::temp_dir().join(format!("kb-mcp-stdio-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("notes/guides")).unwrap();
    std::fs::write(root.join("collections.ron"), RON).unwrap();
    std::fs::write(
        root.join("notes/guides/rate-limits.md"),
        "---\ntags: [api]\n---\n\n# Rate Limits\n\nRetry with exponential backoff.\n",
    )
    .unwrap();
    root
}

/// The text of a tool result's single content block, parsed as JSON.
fn text_json(result: &Value) -> Value {
    assert_eq!(result["content"][0]["type"], "text");
    serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap()
}

#[test]
fn stdio_round_trip() {
    let root = setup();
    let mut server = Server::start(&root);

    let init = server.request(
        1,
        "initialize",
        &json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "stdio-test", "version": "0"},
        }),
    );
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(init["result"]["serverInfo"]["name"], "kb-mcp");
    assert!(init["result"]["capabilities"]["tools"].is_object());
    server.send(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));

    // Tool names and parameters are kb-mcp's public interface (CONTRIBUTING.md, Versioning).
    let listed = server.request(2, "tools/list", &json!({}));
    let mut tools: Vec<(String, Vec<String>)> = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| {
            let mut params: Vec<String> = tool["inputSchema"]["properties"]
                .as_object()
                .map(|p| p.keys().cloned().collect())
                .unwrap_or_default();
            params.sort();
            (tool["name"].as_str().unwrap().to_string(), params)
        })
        .collect();
    tools.sort();
    let expected: Vec<(String, Vec<String>)> = [
        ("get_document", vec!["path"]),
        ("kb_context", vec!["path"]),
        ("kb_digest", vec!["collection"]),
        ("kb_export", vec!["collection", "max_documents"]),
        ("kb_health", vec!["collection", "min_words", "stale_days"]),
        (
            "kb_query",
            vec![
                "collection",
                "created_after",
                "has_sources",
                "status",
                "tag",
            ],
        ),
        (
            "kb_write",
            vec![
                "body",
                "collection",
                "directory",
                "filename",
                "source",
                "status",
                "tags",
                "title",
            ],
        ),
        ("list_sections", vec![]),
        ("reindex", vec![]),
        (
            "search",
            vec!["collection", "max_results", "query", "scope"],
        ),
    ]
    .into_iter()
    .map(|(name, params)| {
        (
            name.to_string(),
            params.into_iter().map(String::from).collect(),
        )
    })
    .collect();
    assert_eq!(tools, expected);

    let sections = text_json(&server.call(3, "list_sections", &json!({})));
    assert_eq!(sections[0]["name"], "guides");
    assert_eq!(sections[0]["description"], "How-to guides");
    assert_eq!(sections[0]["doc_count"], 1);

    let found = text_json(&server.call(4, "search", &json!({"query": "backoff"})));
    assert_eq!(found["results"][0]["path"], "guides/rate-limits.md");
    assert_eq!(found["results"][0]["title"], "Rate Limits");

    let doc = text_json(&server.call(5, "get_document", &json!({"path": "guides/rate-limits.md"})));
    assert!(
        doc["content"]
            .as_str()
            .unwrap()
            .contains("exponential backoff")
    );

    let missing = server.call(6, "get_document", &json!({"path": "no/such.md"}));
    assert_eq!(missing["isError"], true);

    let read_only = server.call(
        7,
        "kb_write",
        &json!({"collection": "notes", "title": "x", "body": "y"}),
    );
    assert_eq!(read_only["isError"], true);

    drop(server);
    let _ = std::fs::remove_dir_all(&root);
}
