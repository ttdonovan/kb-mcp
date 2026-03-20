//! JSON output formatting.
//!
//! Separate output structs decouple serialization shape from internal types.
//! Both MCP tools and CLI commands use these same formatters, ensuring
//! identical JSON output regardless of transport.

use std::collections::HashMap;

use serde::Serialize;

use crate::search::SearchResult;
use crate::types::{Document, Section};

// --- Digest output types ---

#[derive(Serialize)]
pub struct DigestOutput {
    pub total_documents: usize,
    pub total_sections: usize,
    pub collections: Vec<DigestCollectionOutput>,
}

#[derive(Serialize)]
pub struct DigestCollectionOutput {
    pub name: String,
    pub doc_count: usize,
    pub sections: Vec<DigestSectionOutput>,
    pub recent: Vec<DigestRecentOutput>,
}

#[derive(Serialize)]
pub struct DigestSectionOutput {
    pub name: String,
    pub doc_count: usize,
    pub topics: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Serialize)]
pub struct DigestRecentOutput {
    pub path: String,
    pub title: String,
    pub created: String,
}

#[derive(Serialize)]
pub struct SectionOutput {
    pub name: String,
    pub description: String,
    pub doc_count: usize,
    pub collection: String,
}

#[derive(Serialize)]
pub struct DocumentOutput {
    pub path: String,
    pub title: String,
    pub tags: Vec<String>,
    pub section: String,
    pub collection: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Serialize)]
pub struct ContextOutput {
    pub path: String,
    pub title: String,
    pub tags: Vec<String>,
    pub section: String,
    pub collection: String,
    pub frontmatter: HashMap<String, serde_yaml::Value>,
    pub summary: String,
}

#[derive(Serialize)]
pub struct SearchResultOutput {
    pub path: String,
    pub title: String,
    pub section: String,
    pub collection: String,
    pub score: f64,
    pub excerpt: String,
}

#[derive(Serialize)]
pub struct SearchOutput {
    pub query: String,
    pub total: usize,
    pub results: Vec<SearchResultOutput>,
}

#[derive(Serialize)]
pub struct WriteOutput {
    pub path: String,
    pub collection: String,
    pub title: String,
    pub tags: Vec<String>,
}

pub fn format_sections(sections: &[Section]) -> String {
    let output: Vec<SectionOutput> = sections
        .iter()
        .map(|s| SectionOutput {
            name: s.name.clone(),
            description: s.description.clone(),
            doc_count: s.doc_count,
            collection: s.collection.clone(),
        })
        .collect();

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

pub fn format_document(doc: &Document, include_content: bool) -> String {
    let output = DocumentOutput {
        path: doc.path.clone(),
        title: doc.title.clone(),
        tags: doc.tags.clone(),
        section: doc.section.clone(),
        collection: doc.collection.clone(),
        content: if include_content {
            Some(doc.body.clone())
        } else {
            None
        },
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

pub fn format_context(doc: &Document) -> String {
    let summary = extract_summary(&doc.body);
    let output = ContextOutput {
        path: doc.path.clone(),
        title: doc.title.clone(),
        tags: doc.tags.clone(),
        section: doc.section.clone(),
        collection: doc.collection.clone(),
        frontmatter: doc.frontmatter.clone(),
        summary,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

pub fn format_search(query: &str, results: &[SearchResult], documents: &[Document]) -> String {
    let result_outputs: Vec<SearchResultOutput> = results
        .iter()
        .map(|r| {
            let doc = &documents[r.doc_index];
            SearchResultOutput {
                path: doc.path.clone(),
                title: doc.title.clone(),
                section: doc.section.clone(),
                collection: doc.collection.clone(),
                score: (r.score * 1000.0).round() / 1000.0,
                excerpt: r.excerpt.clone(),
            }
        })
        .collect();

    let output = SearchOutput {
        query: query.to_string(),
        total: result_outputs.len(),
        results: result_outputs,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

pub fn format_write(path: &str, collection: &str, title: &str, tags: &[String]) -> String {
    let output = WriteOutput {
        path: path.to_string(),
        collection: collection.to_string(),
        title: title.to_string(),
        tags: tags.to_vec(),
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

/// Extract the first paragraph after the H1 heading as a summary.
/// Falls back to the first non-empty paragraph if no H1 is found.
/// This powers `kb_context` — agents get a meaningful preview without
/// reading the full document, saving 90%+ tokens on scan-heavy workflows.
pub fn extract_summary(body: &str) -> String {
    let mut lines = body.lines().peekable();
    let mut found_heading = false;

    // Skip to after the first H1
    while let Some(line) = lines.peek() {
        if line.starts_with("# ") {
            found_heading = true;
            lines.next();
            // Skip blank lines after heading
            while lines.peek().is_some_and(|l| l.trim().is_empty()) {
                lines.next();
            }
            break;
        }
        lines.next();
    }

    if !found_heading {
        // No heading — take first non-empty paragraph
        let mut lines = body.lines().peekable();
        while lines.peek().is_some_and(|l| l.trim().is_empty()) {
            lines.next();
        }
        return collect_paragraph(&mut lines);
    }

    collect_paragraph(&mut lines)
}

fn collect_paragraph(lines: &mut std::iter::Peekable<std::str::Lines>) -> String {
    let mut paragraph = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        paragraph.push(line);
    }
    paragraph.join("\n")
}

/// Convert a serde_yaml::Value to a display string. serde_yaml::Value doesn't
/// implement Display, so we serialize via serde_json for non-string variants.
pub fn yaml_value_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                f.to_string()
            } else {
                String::new()
            }
        }
        serde_yaml::Value::Null => "null".to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// Build a vault digest — structured overview of collections, sections, topics,
/// and recent additions. The 7-day recency window uses frontmatter `created` dates.
pub fn format_digest(
    documents: &[Document],
    sections: &[Section],
    collection_filter: Option<&str>,
) -> String {
    let today = chrono::Local::now().date_naive();
    let week_ago = today - chrono::Duration::days(7);

    // Group documents by collection
    let mut coll_docs: HashMap<&str, Vec<&Document>> = HashMap::new();
    for doc in documents {
        if collection_filter.is_some_and(|f| doc.collection != f) {
            continue;
        }
        coll_docs.entry(&doc.collection).or_default().push(doc);
    }

    // Group sections by collection
    let mut coll_sections: HashMap<&str, Vec<&Section>> = HashMap::new();
    for sec in sections {
        if collection_filter.is_some_and(|f| sec.collection != f) {
            continue;
        }
        coll_sections.entry(&sec.collection).or_default().push(sec);
    }

    let mut collection_names: Vec<&str> = coll_docs.keys().copied().collect();
    collection_names.sort();

    let mut total_documents = 0;
    let mut total_sections = 0;
    let mut collections = Vec::new();

    for coll_name in collection_names {
        let docs = &coll_docs[coll_name];
        total_documents += docs.len();

        // Build section digests with topics (from doc titles)
        let empty_secs = vec![];
        let secs = coll_sections.get(coll_name).unwrap_or(&empty_secs);
        total_sections += secs.len();

        let mut section_outputs = Vec::new();
        for sec in secs {
            let sec_docs: Vec<&&Document> = docs
                .iter()
                .filter(|d| d.section == sec.name)
                .collect();
            let topics: Vec<String> = sec_docs.iter().map(|d| d.title.clone()).collect();
            let filtered_count = sec_docs.len();
            let hint = if filtered_count < 2 {
                Some("thin — fewer than 2 documents".to_string())
            } else {
                None
            };
            section_outputs.push(DigestSectionOutput {
                name: sec.name.clone(),
                doc_count: filtered_count,
                topics,
                hint,
            });
        }

        // Recent: docs with `created` date in last 7 days
        let mut recent = Vec::new();
        for doc in docs {
            if let Some(created_val) = doc.frontmatter.get("created") {
                let created_str = yaml_value_to_string(created_val);
                if let Ok(created_date) =
                    chrono::NaiveDate::parse_from_str(&created_str, "%Y-%m-%d")
                    && created_date >= week_ago
                {
                    recent.push(DigestRecentOutput {
                        path: doc.path.clone(),
                        title: doc.title.clone(),
                        created: created_str,
                    });
                }
            }
        }
        recent.sort_by(|a, b| b.created.cmp(&a.created));

        collections.push(DigestCollectionOutput {
            name: coll_name.to_string(),
            doc_count: docs.len(),
            sections: section_outputs,
            recent,
        });
    }

    let output = DigestOutput {
        total_documents,
        total_sections,
        collections,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

// --- Query output types ---

#[derive(Serialize)]
pub struct QueryOutput {
    pub total: usize,
    pub documents: Vec<QueryDocumentOutput>,
}

#[derive(Serialize)]
pub struct QueryDocumentOutput {
    pub path: String,
    pub title: String,
    pub tags: Vec<String>,
    pub section: String,
    pub collection: String,
}

/// Format query results — document metadata without body content.
pub fn format_query(docs: &[&Document]) -> String {
    let documents: Vec<QueryDocumentOutput> = docs
        .iter()
        .map(|d| QueryDocumentOutput {
            path: d.path.clone(),
            title: d.title.clone(),
            tags: d.tags.clone(),
            section: d.section.clone(),
            collection: d.collection.clone(),
        })
        .collect();

    let output = QueryOutput {
        total: documents.len(),
        documents,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

/// Read a document's body from disk, stripping YAML frontmatter.
///
/// Shared implementation for both MCP and CLI fresh-from-disk reads.
/// Callers resolve the full filesystem path; this handles frontmatter stripping.
pub fn read_document_body(file_path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(file_path).ok()?;
    if let Some(after) = content.strip_prefix("---")
        && let Some(end) = after.find("\n---")
    {
        let body = &after[end + 4..];
        return Some(body.trim_start_matches('\n').to_string());
    }
    Some(content)
}

/// Build an export document — concatenated markdown with frontmatter headers.
///
/// Takes a list of (document, body) pairs where bodies are pre-read from disk.
/// Both the MCP tool and CLI command delegate to this function to avoid
/// duplicating the markdown assembly logic.
pub fn format_export(
    docs_with_bodies: &[(&Document, String)],
    collection_filter: Option<&str>,
) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d");

    let mut output = String::new();

    let header = match collection_filter {
        Some(c) => format!("# Vault Export: {} ({})\n\n", c, today),
        None => format!("# Vault Export ({})\n\n", today),
    };
    output.push_str(&header);

    for (doc, body) in docs_with_bodies {
        output.push_str("---\n");
        output.push_str(&format!("## {}\n", doc.path));
        for (key, value) in &doc.frontmatter {
            let val_str = yaml_value_to_string(value);
            output.push_str(&format!("{}: {}\n", key, val_str));
        }
        output.push_str("---\n\n");
        output.push_str(body);
        output.push_str("\n\n");
    }

    output
}
