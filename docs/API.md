# API Reference

Base URL: `http://localhost:8006`.

## Health

```bash
curl localhost:8006/health
```

```json
{ "status": "ok", "version": "0.1.0" }
```

## Review

```bash
curl -X POST localhost:8006/api/v1/review \
  -H 'content-type: application/json' \
  -d '{"question":"Does aspirin prevent recurrent stroke?"}'
```

```json
{
  "question": "Does aspirin prevent recurrent stroke?",
  "synthesis": "Top evidence for 'Does aspirin prevent recurrent stroke?': …",
  "confidence": "62% (top TF-IDF relevance)",
  "citations": [
    {
      "pmid": "https://pubmed.ncbi.nlm.nih.gov/12345678/",
      "title": "Aspirin for secondary stroke prevention…",
      "journal": "Stroke",
      "year": "2026",
      "relevance": 0.62,
      "evidence": "BACKGROUND Secondary prevention after ischemic stroke…"
    }
  ],
  "articles_found": 10,
  "stub_model": true,
  "disclaimer": "Evidence summary from PubMed abstracts. Not medical advice. Verify against full texts."
}
```

Citations are the top-5 ranked articles; `relevance` is the TF-IDF
cosine (0–1); `evidence` is a truncated abstract excerpt.

## Errors

| Status | Meaning |
|--------|---------|
| `400` | question too short (< 8 chars) — "be specific" |
| `502` | PubMed unreachable, or LLM upstream error (stub=false) |
