//! Routes: health + review pipeline.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::AppState;
use crate::pubmed::{self, Article};
use crate::rank;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/review", post(review))
        .with_state(state)
}

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

#[derive(Debug, Deserialize)]
pub struct ReviewRequest {
    pub question: String,
}

/// The evidence pipeline: search → summaries → abstracts → rank →
/// synthesis with citations. Stub mode returns a deterministic synthesis
/// over the ranked evidence (no GPU needed).
pub async fn review(State(state): State<AppState>, Json(request): Json<ReviewRequest>) -> Response {
    if request.question.trim().len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "question too short — be specific" })),
        )
            .into_response();
    }

    let pmids =
        match pubmed::search(&state.http, &request.question, state.config.max_articles).await {
            Ok(pmids) => pmids,
            Err(err) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "error": format!("pubmed unreachable: {err}") })),
                )
                    .into_response();
            }
        };
    let mut articles = match pubmed::summaries(&state.http, &pmids).await {
        Ok(articles) => articles,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("pubmed summaries: {err}") })),
            )
                .into_response();
        }
    };
    if let Err(err) = pubmed::attach_abstracts(&state.http, &mut articles).await {
        tracing::warn!(%err, "abstract fetch failed; ranking on titles");
    }

    let ranked: Vec<(Article, f64)> = rank::rank(&request.question, &articles);
    let citations: Vec<Value> = ranked
        .iter()
        .take(5)
        .map(|(article, score)| {
            json!({
                "pmid": format!("https://pubmed.ncbi.nlm.nih.gov/{}/", article.pmid),
                "title": article.title,
                "journal": article.journal,
                "year": article.pub_year,
                "relevance": (score * 100.0).round() / 100.0,
                "evidence": truncate(&article.abstract_text, 320),
            })
        })
        .collect();

    let synthesis = if state.config.model_stub {
        stub_synthesis(&request.question, &ranked)
    } else {
        match ollama_synthesis(&state, &request.question, &ranked).await {
            Ok(text) => text,
            Err(err) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({ "error": format!("llm upstream: {err}") })),
                )
                    .into_response();
            }
        }
    };

    let confidence = ranked
        .first()
        .map(|(_, score)| ((score * 100.0).round() as u64).min(99))
        .unwrap_or(0);

    (
        StatusCode::OK,
        Json(json!({
            "question": request.question,
            "synthesis": synthesis,
            "confidence": format!("{}% (top TF-IDF relevance)", confidence),
            "citations": citations,
            "articles_found": articles.len(),
            "stub_model": state.config.model_stub,
            "disclaimer": "Evidence summary from PubMed abstracts. Not medical advice. Verify against full texts.",
        })),
    )
        .into_response()
}

fn truncate(text: &str, max: usize) -> String {
    let clean: String = text.chars().take(max).collect();
    if text.chars().count() > max {
        format!("{clean}…")
    } else {
        clean
    }
}

fn stub_synthesis(question: &str, ranked: &[(Article, f64)]) -> String {
    let top: Vec<String> = ranked
        .iter()
        .take(3)
        .map(|(article, _)| article.title.clone())
        .collect();
    format!(
        "Top evidence for '{question}': {}. Enable a running Ollama instance \
         (APP_MODEL_STUB=false) for a real schema-constrained synthesis.",
        top.join("; ")
    )
}

async fn ollama_synthesis(
    state: &AppState,
    question: &str,
    ranked: &[(Article, f64)],
) -> Result<String, String> {
    let evidence = ranked
        .iter()
        .take(5)
        .map(|(article, _)| {
            format!(
                "PMID {} ({}): {} — {}",
                article.pmid,
                article.pub_year,
                article.title,
                truncate(&article.abstract_text, 500)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    #[derive(serde::Deserialize)]
    struct ChatResponse {
        message: ChatMessage,
    }
    #[derive(serde::Deserialize)]
    struct ChatMessage {
        content: String,
    }

    let body = json!({
        "model": state.config.ollama_model,
        "messages": [
            {"role": "system", "content": "You are an evidence-based medical research assistant. \
             Synthesize findings ONLY from the provided abstracts. Cite PMIDs inline. \
             State confidence and note gaps. You are not giving medical advice."},
            {"role": "user", "content": format!("Question: {question}\n\nEvidence:\n{evidence}")}
        ],
        "stream": false,
        "options": {"temperature": 0.1}
    });
    let response: ChatResponse = state
        .http
        .post(format!("{}/api/chat", state.config.ollama_url))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    Ok(response.message.content)
}
