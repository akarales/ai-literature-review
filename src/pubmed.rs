//! PubMed E-utilities client: esearch (JSON) → esummary (metadata) →
//! efetch (text abstracts, lenient parsing).
//!
//! NCBI etiquette: tool/email params, 3 req/s, cache at the caller.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Article {
    pub pmid: String,
    pub title: String,
    pub abstract_text: String,
    pub journal: String,
    pub pub_year: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PubMedError {
    #[error("network error: {0}")]
    Network(String),
    #[error("bad response: {0}")]
    BadResponse(String),
}

const BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
const TOOL: &str = "ai-literature-review";
const EMAIL: &str = "karales@gmail.com";

async fn get_json(
    http: &reqwest::Client,
    url: &str,
    params: &[(&str, &str)],
) -> Result<serde_json::Value, PubMedError> {
    http.get(url)
        .query(&params)
        .send()
        .await
        .map_err(|e| PubMedError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| PubMedError::Network(e.to_string()))?
        .json()
        .await
        .map_err(|e| PubMedError::BadResponse(e.to_string()))
}

async fn get_text(
    http: &reqwest::Client,
    url: &str,
    params: &[(&str, &str)],
) -> Result<String, PubMedError> {
    let response = http
        .get(url)
        .query(&params)
        .send()
        .await
        .map_err(|e| PubMedError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| PubMedError::Network(e.to_string()))?;
    let text = response
        .text()
        .await
        .map_err(|e| PubMedError::BadResponse(e.to_string()))?;
    Ok(text)
}

/// Search PubMed for PMIDs matching a query (relevance-sorted).
pub async fn search(
    http: &reqwest::Client,
    query: &str,
    max: usize,
) -> Result<Vec<String>, PubMedError> {
    let response = get_json(
        http,
        &format!("{BASE}/esearch.fcgi"),
        &[
            ("db", "pubmed"),
            ("term", query),
            ("retmode", "json"),
            ("retmax", &max.to_string()),
            ("sort", "relevance"),
            ("tool", TOOL),
            ("email", EMAIL),
        ],
    )
    .await?;
    Ok(response["esearchresult"]["idlist"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default())
}

/// Metadata (title, journal, year) via esummary JSON.
pub async fn summaries(
    http: &reqwest::Client,
    pmids: &[String],
) -> Result<Vec<Article>, PubMedError> {
    if pmids.is_empty() {
        return Ok(Vec::new());
    }
    let ids = pmids.join(",");
    let response = get_json(
        http,
        &format!("{BASE}/esummary.fcgi"),
        &[
            ("db", "pubmed"),
            ("id", &ids),
            ("retmode", "json"),
            ("tool", TOOL),
            ("email", EMAIL),
        ],
    )
    .await?;

    let uids: Vec<String> = response["result"]["uids"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(uids
        .into_iter()
        .filter_map(|pmid| {
            let article = &response["result"][&pmid];
            let title = article["title"].as_str()?.to_string();
            Some(Article {
                journal: article["source"].as_str().unwrap_or_default().to_string(),
                pub_year: article["pubdate"]
                    .as_str()
                    .unwrap_or_default()
                    .split(' ')
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                pmid,
                title,
                abstract_text: String::new(),
            })
        })
        .collect())
}

/// Abstracts via efetch text mode; records end with "PMID: N" lines.
pub async fn attach_abstracts(
    http: &reqwest::Client,
    articles: &mut [Article],
) -> Result<(), PubMedError> {
    if articles.is_empty() {
        return Ok(());
    }
    let ids: Vec<String> = articles.iter().map(|a| a.pmid.clone()).collect();
    let text = get_text(
        http,
        &format!("{BASE}/efetch.fcgi"),
        &[
            ("db", "pubmed"),
            ("id", &ids.join(",")),
            ("rettype", "abstract"),
            ("retmode", "text"),
            ("tool", TOOL),
            ("email", EMAIL),
        ],
    )
    .await?;

    // Lenient parse: each record ENDS with a "PMID: N" line; everything
    // accumulated before it is title+authors+abstract text.
    let mut blocks: Vec<(String, String)> = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        if let Some(pmid) = line.strip_prefix("PMID: ") {
            if !current.trim().is_empty() {
                blocks.push((pmid.trim().to_string(), current.trim().to_string()));
            }
            current.clear();
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    for (pmid, body) in blocks {
        if let Some(article) = articles.iter_mut().find(|a| a.pmid == pmid) {
            article.abstract_text = body;
        }
    }
    Ok(())
}
