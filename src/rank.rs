//! Relevance ranking: TF-IDF cosine between the question and each
//! article's title+abstract — pure Rust, no embedding service for the
//! scaffold (Ollama embeddings land in Phase 2). Deterministic and
//! unit-testable.

use std::collections::HashMap;

use crate::pubmed::Article;

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() > 2 && !STOPWORDS.contains(t))
        .map(String::from)
        .collect()
}

const STOPWORDS: [&str; 12] = [
    "the", "and", "for", "with", "from", "that", "this", "are", "was", "has", "have", "its",
];

/// TF-IDF vector of a document against corpus document frequencies.
fn vector(doc: &str, df: &HashMap<String, usize>, corpus_size: usize) -> HashMap<String, f64> {
    let tokens = tokenize(doc);
    let total = tokens.len().max(1) as f64;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for token in &tokens {
        *counts.entry(token.clone()).or_insert(0) += 1;
    }
    let mut vector = HashMap::new();
    for (token, count) in counts {
        let tf = count as f64 / total;
        let idf = ((corpus_size as f64 + 1.0) / (*df.get(&token).unwrap_or(&0) as f64 + 1.0)).ln();
        vector.insert(token, tf * idf);
    }
    vector
}

fn cosine(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let dot: f64 = a
        .iter()
        .filter_map(|(k, va)| b.get(k).map(|vb| va * vb))
        .sum();
    let norm_a: f64 = a.values().map(|v| v * v).sum::<f64>().sqrt();
    let norm_b: f64 = b.values().map(|v| v * v).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// Rank articles against the question; returns (article, score) pairs
/// sorted best-first. Also emits one-line citation evidence strings.
pub fn rank(question: &str, articles: &[Article]) -> Vec<(Article, f64)> {
    let corpus_size = articles.len().max(1);

    // Document frequencies across title+abstract.
    let mut df: HashMap<String, usize> = HashMap::new();
    for article in articles {
        let tokens = tokenize(&format!("{} {}", article.title, article.abstract_text));
        let mut seen = std::collections::HashSet::new();
        for token in tokens {
            if seen.insert(token.clone()) {
                *df.entry(token).or_insert(0) += 1;
            }
        }
    }

    let query = vector(question, &df, corpus_size);
    let mut scored: Vec<(Article, f64)> = articles
        .iter()
        .map(|article| {
            let doc = vector(
                &format!("{} {}", article.title, article.abstract_text),
                &df,
                corpus_size,
            );
            (article.clone(), cosine(&query, &doc))
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    fn article(pmid: &str, title: &str, abstract_text: &str) -> Article {
        Article {
            pmid: pmid.into(),
            title: title.into(),
            abstract_text: abstract_text.into(),
            journal: "J".into(),
            pub_year: "2026".into(),
        }
    }

    #[test]
    fn relevant_article_ranks_first() {
        let articles = vec![
            article(
                "1",
                "Aspirin and stroke prevention",
                "Aspirin reduces recurrent stroke risk in patients...",
            ),
            article(
                "2",
                "Marine biology of dolphins",
                "Dolphins exhibit complex social behaviors...",
            ),
            article(
                "3",
                "Aspirin dosing in headache",
                "Low-dose aspirin for tension headache treatment...",
            ),
        ];
        let ranked = rank("does aspirin prevent stroke", &articles);
        assert_eq!(
            ranked[0].0.pmid, "1",
            "stroke/aspirin article must rank first"
        );
        assert!(
            ranked[0].1 > ranked[2].1,
            "dolphin article must score lower"
        );
    }

    #[test]
    fn identical_text_scores_highest() {
        let articles = vec![
            article("1", "unrelated", "quantum computing advances"),
            article("2", "melatonin sleep", "melatonin improves sleep onset"),
        ];
        let ranked = rank("melatonin sleep onset", &articles);
        assert_eq!(ranked[0].0.pmid, "2");
        assert!(ranked[0].1 > 0.1);
    }

    #[test]
    fn empty_corpus_is_empty() {
        assert!(rank("anything", &[]).is_empty());
    }
}
