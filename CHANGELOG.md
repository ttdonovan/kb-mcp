# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/ttdonovan/kb-mcp/releases/tag/v0.3.0) - 2026-11-20

### Added

- add --version to kb and kb-mcp.
- upgrade rmcp to 1.5, add memvid-core hybrid dep, fix tool_handler router binding

### Fixed

- auto-reindex a collection when another server replaced its .mv2.
- surface root-level docs via "(root)" section; make explicit reindex a full rebuild

### Other

- release with release-plz in git-only mode, build and attest binaries for each release, and gate PRs on fmt, clippy, tests, audit and crate age.
- apply cargo fmt to the workspace, so CI can check formatting.
- split into cargo workspace (kb-core, kb-cli, kb-mcp-server)
