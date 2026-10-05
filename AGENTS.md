# AGENTS.md

## Commands

```bash
cargo run                  # :8006
cargo test -q              # ranker unit tests + request contract
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cd frontend && pnpm install && pnpm dev && pnpm build
```

## Environment

`APP_PORT` (8006) · `APP_MODEL_STUB` (true) · `APP_OLLAMA_URL` ·
`APP_OLLAMA_MODEL` (qwen3:14b) · `APP_MAX_ARTICLES` (10)

## Conventions

- Conventional commits; hygiene hook strips AI attribution
- PubMed client stays in `src/pubmed.rs`: tool/email params always, no
  retries faster than 3 req/s, failures degrade to title-only ranking
- The ranker is pure (no I/O) — extend there with embeddings in Phase 1
- Deps ≥7 days old (BEST_PRACTICES/INDEX.md)
