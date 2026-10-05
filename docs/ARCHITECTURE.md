# Architecture

## Modules

```
src/
├── pubmed.rs    # E-utilities client: search, summaries, attach_abstracts
├── rank.rs      # TF-IDF cosine ranker (pure, unit-tested)
├── routes.rs    # review pipeline + synthesis backends
└── state.rs     # shared HTTP client + config
```

## PubMed client (src/pubmed.rs)

Three E-utility calls per question, all with NCBI etiquette parameters
(`tool=ai-literature-review`, contact email) — the documented way to
stay inside the 3 req/s unauthenticated tier:

1. **esearch** (JSON) — relevance-sorted PMIDs for the question
2. **esummary** (JSON) — titles, journals, years per PMID
3. **efetch** (text mode) — abstract bodies, parsed leniently: each
   record ends with a `PMID: N` line; blocks are paired back to articles
   by PMID. Abstract failure degrades to title-only ranking (warned,
   not fatal).

## Ranker (src/rank.rs)

TF-IDF cosine between the question and each article's title+abstract,
with stopword filtering and document frequencies computed across the
result set. Deterministic and unit-tested: the aspirin/stroke article
must outrank a dolphin article; identical text must score highest; an
empty corpus must return empty. Embedding-based ranking (Ollama) is the
Phase 1 upgrade — the ranker is deliberately a pure module so the swap
is isolated.

## Synthesis discipline (src/routes.rs)

- The model receives ONLY the ranked evidence with a system prompt
  that bounds it: synthesize from the provided abstracts, cite PMIDs
  inline, state confidence and gaps, no medical advice
- Temperature 0.1
- The API response carries the machine-checkable provenance: top-5
  citations with PMIDs, titles, evidence excerpts, per-citation
  relevance scores, and a confidence string tied to the top score
- Stub mode returns a deterministic synthesis over the top titles —
  the pipeline is observable without a GPU

## Design decisions

| Decision | Why |
|----------|-----|
| TF-IDF before embeddings | Zero infra for the scaffold; pure module isolates the Phase 1 swap |
| efetch text mode, lenient parsing | The XML API is heavyweight for the scaffold; records end with PMID lines — a stable boundary |
| Failures degrade, not abort | PubMed flakiness shouldn't 500 the pipeline; ranking falls back to titles |
