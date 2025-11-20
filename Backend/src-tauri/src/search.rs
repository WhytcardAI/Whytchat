use anyhow::{Context, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

#[tauri::command]
pub async fn search_web(query: String) -> Result<Vec<SearchResult>, String> {
    // Utilisation de DuckDuckGo HTML (version légère)
    let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(&query));
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let html_content = resp.text().await.map_err(|e| e.to_string())?;
    
    let document = Html::parse_document(&html_content);
    let result_selector = Selector::parse(".result").unwrap();
    let title_selector = Selector::parse(".result__a").unwrap();
    let snippet_selector = Selector::parse(".result__snippet").unwrap();

    let mut results = Vec::new();

    for element in document.select(&result_selector) {
        let title_el = element.select(&title_selector).next();
        let snippet_el = element.select(&snippet_selector).next();

        if let (Some(title_el), Some(snippet_el)) = (title_el, snippet_el) {
            let title = title_el.text().collect::<Vec<_>>().join("");
            let link = title_el.value().attr("href").unwrap_or_default().to_string();
            let snippet = snippet_el.text().collect::<Vec<_>>().join("");

            if !title.is_empty() && !snippet.is_empty() {
                results.push(SearchResult {
                    title,
                    link,
                    snippet,
                });
            }
        }
        
        if results.len() >= 5 {
            break;
        }
    }

    Ok(results)
}