use anyhow::{anyhow, Result};
use rig::completion::Prompt;
use rig::providers::ollama;

use crate::tavily::TavilyResult;

pub async fn summarize(
    query: &str,
    results: &[TavilyResult],
    ollama_host: &str,
    model_name: &str,
) -> Result<String> {
    let context_text = results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "[{}] タイトル: {}\nURL: {}\n内容: {}\n",
                i + 1,
                r.title,
                r.url,
                r.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt_text = format!(
        "以下の検索結果のみを根拠として「{query}」について日本語で要約してください。\n\
        検索結果に含まれていない情報を推測・補完しないでください。\n\
        事実のみを簡潔にまとめ、重要なポイントを箇条書きで示してください。\n\n\
        【検索結果】\n{context_text}\n\
        【要約】"
    );

    let client = ollama::Client::from_url(ollama_host);
    let agent = client.agent(model_name).build();

    let response = agent.prompt(prompt_text.as_str()).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("Connection refused")
            || msg.contains("error sending request")
            || msg.contains("connect")
            || msg.contains("tcp connect")
        {
            anyhow!(
                "Ollama に接続できません。Ollama が起動しているか、ホスト設定 ({}) が正しいか確認してください。",
                ollama_host
            )
        } else {
            anyhow!("Ollama エラー: {}", msg)
        }
    })?;

    Ok(response)
}
