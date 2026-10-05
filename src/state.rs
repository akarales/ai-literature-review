//! Shared state: HTTP client + config.

use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct AppState {
    pub http: reqwest::Client,
    pub config: Config,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub model_stub: bool,
    pub ollama_url: String,
    pub ollama_model: String,
    pub port: u16,
    pub max_articles: usize,
}

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

impl Default for Config {
    fn default() -> Self {
        Self {
            model_stub: true,
            ollama_url: "http://localhost:11434".into(),
            ollama_model: "qwen3:14b".into(),
            port: 8006,
            max_articles: 10,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            http: HTTP.clone(),
            config: Config::default(),
        }
    }
}
