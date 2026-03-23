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

// --- Health output types ---

#[derive(Serialize)]
pub struct HealthOutput {
    pub total_documents_checked: usize,
    pub total_issues: usize,
    pub collections: Vec<HealthCollectionOutput>,
}

#[derive(Serialize)]
pub struct HealthCollectionOutput {
    pub name: String,
    pub doc_count: usize,
    pub issues: usize,
    pub missing_created: Vec<HealthDocRef>,
    pub missing_updated: Vec<HealthDocRef>,
    pub no_tags: Vec<HealthDocRef>,
    pub stale: Vec<HealthStaleRef>,
    pub stubs: Vec<HealthStubRef>,
    pub orphans: Vec<HealthDocRef>,
    pub broken_links: Vec<HealthBrokenLinkRef>,
}

#[derive(Serialize)]
pub struct HealthDocRef {
    pub path: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct HealthStaleRef {
    pub path: String,
    pub title: String,
    pub last_date: String,
}

#[derive(Serialize)]
pub struct HealthStubRef {
    pub path: String,
    pub title: String,
    pub word_count: usize,
}

#[derive(Serialize)]
pub struct HealthBrokenLinkRef {
    pub source: String,
    pub target: String,
    pub raw: String,
}

/// Build a vault health report — document quality and hygiene checks.
///
/// Checks frontmatter completeness (created/updated/tags), content quality
/// (staleness, stub detection), and link integrity (orphans, broken wiki-links).
pub fn format_health(
    documents: &[Document],
    collection_filter: Option<&str>,
    stale_days: u32,
    min_words: u32,
) -> String {
    let today = chrono::Local::now().date_naive();
    let stale_cutoff = today - chrono::Duration::days(stale_days as i64);

    // Filter documents by collection
    let docs: Vec<&Document> = documents
        .iter()
        .filter(|d| collection_filter.is_none_or(|f| d.collection == f))
        .collect();

    // --- Wiki-link graph (Phase 2) ---
    let link_re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    // Build a set of all doc identifiers for resolution
    // Key: lowercase title or lowercase path (without .md)
    let mut doc_id_to_idx: HashMap<String, usize> = HashMap::new();
    for (i, doc) in docs.iter().enumerate() {
        doc_id_to_idx.insert(doc.title.to_lowercase(), i);
        doc_id_to_idx.insert(doc.path.to_lowercase(), i);
        let path_no_ext = doc.path.trim_end_matches(".md").to_lowercase();
        doc_id_to_idx.insert(path_no_ext, i);
    }

    // Track inbound links per doc index and broken links per source doc
    let mut inbound_count: Vec<usize> = vec![0; docs.len()];
    let mut total_wiki_links = 0usize;
    // (source_doc_idx, target_string, raw_match)
    let mut broken: Vec<(usize, String, String)> = Vec::new();

    for (src_idx, doc) in docs.iter().enumerate() {
        for cap in link_re.captures_iter(&doc.body) {
            total_wiki_links += 1;
            let raw = cap[0].to_string();
            let inner = &cap[1];
            // Strip alias after | and heading anchor after #
            let target = inner
                .split('|')
                .next()
                .unwrap_or(inner)
                .split('#')
                .next()
                .unwrap_or(inner)
                .trim();
            let target_lower = target.to_lowercase();

            if let Some(&tgt_idx) = doc_id_to_idx.get(&target_lower) {
                if tgt_idx != src_idx {
                    inbound_count[tgt_idx] += 1;
                }
            } else {
                // Also try with .md extension
                let with_ext = format!("{}.md", target_lower);
                if let Some(&tgt_idx) = doc_id_to_idx.get(&with_ext) {
                    if tgt_idx != src_idx {
                        inbound_count[tgt_idx] += 1;
                    }
                } else {
                    broken.push((src_idx, target.to_string(), raw));
                }
            }
        }
    }

    // --- Group by collection ---
    let mut coll_docs: HashMap<&str, Vec<(usize, &&Document)>> = HashMap::new();
    for (i, doc) in docs.iter().enumerate() {
        coll_docs
            .entry(&doc.collection)
            .or_default()
            .push((i, doc));
    }

    let mut coll_names: Vec<&str> = coll_docs.keys().copied().collect();
    coll_names.sort();

    let mut total_checked = 0;
    let mut total_issues = 0;
    let mut collections = Vec::new();

    for coll_name in coll_names {
        let coll_entries = &coll_docs[coll_name];
        total_checked += coll_entries.len();

        let mut missing_created = Vec::new();
        let mut missing_updated = Vec::new();
        let mut no_tags = Vec::new();
        let mut stale = Vec::new();
        let mut stubs = Vec::new();
        let mut orphans = Vec::new();
        let mut coll_broken = Vec::new();

        for &(doc_global_idx, doc) in coll_entries {
            let doc_ref = || HealthDocRef {
                path: doc.path.clone(),
                title: doc.title.clone(),
            };

            // Missing created
            if !doc.frontmatter.contains_key("created") {
                missing_created.push(doc_ref());
            }

            // Missing updated
            if !doc.frontmatter.contains_key("updated") {
                missing_updated.push(doc_ref());
            }

            // No tags
            if doc.tags.is_empty() {
                no_tags.push(doc_ref());
            }

            // Staleness: use updated, fall back to created
            let last_date_str = doc
                .frontmatter
                .get("updated")
                .or_else(|| doc.frontmatter.get("created"))
                .map(yaml_value_to_string);

            if let Some(ref date_str) = last_date_str
                && let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                && date < stale_cutoff
            {
                stale.push(HealthStaleRef {
                    path: doc.path.clone(),
                    title: doc.title.clone(),
                    last_date: date_str.clone(),
                });
            }

            // Stubs
            let word_count = doc.body.split_whitespace().count();
            if word_count < min_words as usize {
                stubs.push(HealthStubRef {
                    path: doc.path.clone(),
                    title: doc.title.clone(),
                    word_count,
                });
            }

            // Orphans (only if vault has wiki-links at all)
            if total_wiki_links > 0 && inbound_count[doc_global_idx] == 0 {
                orphans.push(doc_ref());
            }

            // Broken links from this doc
            for (src_idx, target, raw) in &broken {
                if *src_idx == doc_global_idx {
                    coll_broken.push(HealthBrokenLinkRef {
                        source: doc.path.clone(),
                        target: target.clone(),
                        raw: raw.clone(),
                    });
                }
            }
        }

        let issues = missing_created.len()
            + missing_updated.len()
            + no_tags.len()
            + stale.len()
            + stubs.len()
            + orphans.len()
            + coll_broken.len();
        total_issues += issues;

        collections.push(HealthCollectionOutput {
            name: coll_name.to_string(),
            doc_count: coll_entries.len(),
            issues,
            missing_created,
            missing_updated,
            no_tags,
            stale,
            stubs,
            orphans,
            broken_links: coll_broken,
        });
    }

    let output = HealthOutput {
        total_documents_checked: total_checked,
        total_issues,
        collections,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}
