//! ai-literature-review — evidence-based medical literature review.
//!
//! Pipeline: question → PubMed search (E-utilities) → fetch abstracts →
//! relevance ranking (TF-IDF cosine in Rust — no embedding service for
//! the scaffold; Ollama embeddings are Phase 2) → LLM synthesis with
//! citations + confidence (stub mode default). Tests run entirely against
//! committed fixture responses — no network.

pub mod pubmed;
pub mod rank;
pub mod routes;
pub mod state;

pub use state::AppState;
