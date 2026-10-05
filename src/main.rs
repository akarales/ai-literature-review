//! Binary entry.

use ai_literature_review::AppState;
use ai_literature_review::routes;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,ai_literature_review=debug".into()),
        )
        .init();

    let state = AppState::default();
    let port = std::env::var("APP_PORT")
        .unwrap_or_else(|_| "8006".to_string())
        .parse()
        .expect("APP_PORT must be a valid port");

    let app = routes::router(state).layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(%port, "ai literature review listening");
    axum::serve(listener, app).await?;
    Ok(())
}
