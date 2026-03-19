//! JSON output formatting.
//!
//! Separate output structs decouple serialization shape from internal types.
//! Both MCP tools and CLI commands use these same formatters, ensuring
//! identical JSON output regardless of transport.

use std::collections::HashMap;

use serde::Serialize;

use crate::search::SearchResult;
use crate::types::{Document, Section};

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
