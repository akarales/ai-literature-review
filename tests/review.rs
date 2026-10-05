//! Pipeline tests: ranking is exercised with real unit tests in
//! `rank.rs`; here we test the request contract (validation + stub
//! shape) — PubMed itself is network I/O and stays behind the client.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;

use ai_literature_review::AppState;
use ai_literature_review::routes;

#[tokio::test]
async fn health_ok() {
    let app = routes::router(AppState::default());
    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).expect("builds"))
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn short_question_rejected() {
    let app = routes::router(AppState::default());
    let response = app
        .oneshot(
            Request::post("/api/v1/review")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"question":"aspirin"}"#))
                .expect("builds"),
        )
        .await
        .expect("request");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    assert!(body["error"].as_str().unwrap().contains("too short"));
}
