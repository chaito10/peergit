# Changelog

All notable changes to PeerGit will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.1.0] - 2026-08-19

### Added
- Node identity management with Ed25519 keys, DID:key, and libp2p PeerId
- Fossil transport adapter bridging `--transport-command` with libp2p
- Request-response protocol (`/peergit/xfer/1.0`) with length-prefixed framing
- Kademlia DHT for decentralized peer discovery
- Embedded web dashboard (HTML/JS/CSS, no Node/npm) on port 3000
- Repository publishing and discovery via RID (SHA256)
- SQLite application metadata database for peers, repos, and identity
- Configuration via JSON with XDG directory support
- Fossil CLI passthrough
- Full test suite (11 tests)
- Pre-built binaries for Windows x64, Linux x64, Linux ARM64, macOS x64, macOS ARM64
- GitHub Actions CI for multi-platform release builds
- Examples folder with detailed step-by-step guides (basic setup, peer discovery, fossil sync, web dashboard, multi-node, transport command)
