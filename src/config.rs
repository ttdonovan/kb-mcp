//! Configuration loading and resolution.
//!
//! Separates raw RON config (relative paths, optional fields) from resolved
//! config (absolute paths, defaults applied). This two-phase approach lets
//! collection paths in the RON file be relative to the config file's location,
//! enabling the same binary to serve different projects via different configs.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Raw configuration as deserialized from `collections.ron`.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub cache_dir: Option<String>,
    pub collections: Vec<CollectionDef>,
}

/// A collection definition from the RON config, with paths still relative.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectionDef {
    pub name: String,
    pub path: String,
    pub description: String,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub sections: Vec<SectionDef>,
}

/// Maps a directory prefix to a human-readable description for `list_sections` output.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SectionDef {
    pub prefix: String,
    pub description: String,
}

/// Resolved config with absolute paths.
#[derive(Debug)]
pub struct ResolvedConfig {
    pub cache_dir: PathBuf,
    pub collections: Vec<ResolvedCollection>,
}

#[derive(Debug, Clone)]
pub struct ResolvedCollection {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub writable: bool,
    pub sections: Vec<SectionDef>,
}

impl Config {
    /// Resolve relative paths against the config file's parent directory.
    pub fn resolve(self, config_dir: &Path) -> ResolvedConfig {
        let cache_dir = self
            .cache_dir
            .map(|s| {
                let expanded = shellexpand(&s);
                PathBuf::from(expanded)
            })
            .unwrap_or_else(default_cache_dir);

        let collections = self
            .collections
            .into_iter()
            .map(|c| {
                let path = config_dir.join(&c.path);
                let path = path.canonicalize().unwrap_or(path);
                ResolvedCollection {
                    name: c.name,
                    path,
                    description: c.description,
                    writable: c.writable,
                    sections: c.sections,
                }
            })
            .collect();

        ResolvedConfig {
            cache_dir,
            collections,
        }
    }
}

/// Load config using resolution chain:
/// 1. --config CLI flag
/// 2. KB_MCP_CONFIG env var
/// 3. ./collections.ron
/// 4. ~/.config/kb-mcp/collections.ron
pub fn load_config(explicit_path: Option<&Path>) -> Result<ResolvedConfig> {
    let (config_path, config_dir) = find_config(explicit_path)?;
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read config: {}", config_path.display()))?;
    let config: Config =
        ron::from_str(&content).with_context(|| format!("failed to parse: {}", config_path.display()))?;
    Ok(config.resolve(&config_dir))
}

fn find_config(explicit_path: Option<&Path>) -> Result<(PathBuf, PathBuf)> {
    // 1. Explicit --config flag
    if let Some(path) = explicit_path {
        let path = path.to_path_buf();
        let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        return Ok((path, dir));
    }

    // 2. KB_MCP_CONFIG env var
    if let Ok(env_path) = std::env::var("KB_MCP_CONFIG") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            return Ok((path, dir));
        }
    }

    // 3. ./collections.ron (current working directory)
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let local = cwd.join("collections.ron");
    if local.exists() {
        return Ok((local, cwd));
    }

    // 4. ~/.config/kb-mcp/collections.ron
    if let Some(config_home) = dirs::config_dir() {
        let global = config_home.join("kb-mcp").join("collections.ron");
        if global.exists() {
            let dir = global.parent().unwrap().to_path_buf();
            return Ok((global, dir));
        }
    }

    anyhow::bail!(
        "No collections.ron found. Searched:\n  \
         - KB_MCP_CONFIG env var\n  \
         - ./collections.ron\n  \
         - ~/.config/kb-mcp/collections.ron\n\n\
         Create a collections.ron or pass --config <path>"
    )
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("kb-mcp")
}

/// Minimal `~` expansion — only handles `~/...` prefix. We avoid pulling in
/// a full shellexpand crate for this single use case.
fn shellexpand(s: &str) -> String {
    if s.starts_with("~/")
        && let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &s[1..]);
        }
    s.to_string()
}
