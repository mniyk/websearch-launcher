use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct TavilyRequest {
    query: String,
    max_results: u32,
    include_answer: bool,
    include_raw_content: bool,
    topic: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TavilyResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

pub async fn search(query: &str, api_key: &str, use_news: bool) -> Result<Vec<TavilyResult>> {
    if api_key.is_empty() {
        return Err(anyhow!(
            "TAVILY_API_KEY が設定されていません。.env ファイルに TAVILY_API_KEY を追加してください。"
        ));
    }

    let client = Client::new();
    let topic = if use_news { "news" } else { "general" };

    let request = TavilyRequest {
        query: query.to_string(),
        max_results: 6,
        include_answer: false,
        include_raw_content: false,
        topic: topic.to_string(),
    };

    let response = client
        .post("https://api.tavily.com/search")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow!("Tavily への接続に失敗しました: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Tavily API エラー ({}): {}", status, body));
    }

    let tavily_response: TavilyResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Tavily レスポンスの解析に失敗しました: {}", e))?;

    if tavily_response.results.is_empty() {
        return Err(anyhow!(
            "検索結果が 0 件でした。別のキーワードでお試しください。"
        ));
    }

    Ok(tavily_response.results)
}
