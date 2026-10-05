# Development Guide

Machine-facing commands live in [AGENTS.md](../AGENTS.md).

## Prerequisites

Rust 1.96, cargo · pnpm 11 / Node 24 · network access for PubMed
(only when actually reviewing — tests don't need it) · optional Ollama
for real synthesis.

## Daily loop

```bash
cargo run                  # :8006
cd frontend && pnpm dev     # :5173 → /api proxied
cargo test -q              # 5 tests — ranker + contract
cargo clippy --all-targets -- -D warnings
```

## Testing notes

- Ranker tests are pure (no network): relevance ordering, exact-match
  scoring, empty corpus
- Contract tests cover validation (too-short questions → 400) without
  touching PubMed; the client's network behavior is exercised manually
  or in later live tests

## Gotchas learned here

- **reqwest 0.13**: `.query()` requires the `query` feature — without it
  the method doesn't exist; TLS feature is `rustls` (not `rustls-tls`)
- PubMed efetch text mode: records END with `PMID: N` lines — parse on
  that boundary and pair by PMID, not by order
- Port 8006

## Conventions

Conventional commits; hygiene hook strips AI attribution. The PubMed
client stays in `src/pubmed.rs` — tool/email params always, no retries
faster than 3 req/s, failures degrade to title-only ranking. Ranker
changes require updated unit tests.
