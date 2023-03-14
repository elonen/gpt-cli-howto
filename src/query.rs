use std::sync::mpsc;
use futures_util::stream::StreamExt;
use log::{debug, warn};

use crate::config::Config;

/// Initiate a streaming request to OpenAI, and return the answer.
/// 
/// @param conf Configuration
/// @param question The question to ask
/// @param out_queue A channel to send the answer to, progressively token by token
/// @return The complete answer, and the total number of tokens used
pub async fn perform_streaming_request(
    conf: Config,
    question: String,
    out_queue: mpsc::Sender<String> )
        -> anyhow::Result<(String, Option<u64>)>
{
    let url = "https://api.openai.com/v1/chat/completions";
    let authorization = format!("Bearer {}", conf.openai_token);

    let req_body_json = serde_json::json!({
      "model": conf.model,
      "stream": true,
      "temperature": conf.temperature,
      "messages": [
        {"role": "system", "content": conf.priming_msg},
        {"role": "user", "content": question},
    ]});

    let mut stream = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?
        .post(url)
        .header("Authorization", authorization)
        .header("Cache-Control", "no-cache")
        .header("Content-Type", "text/event-stream")
        .header("Access-Control-Allow-Origin", "*")
        .header("Connection", "keep-alive")
        .json(&req_body_json)
        .send()
        .await?
        .error_for_status()?
        .bytes_stream();

    let mut total_tokens = 0;
    let mut answer = String::new();

    while let Some(event) = stream.next().await {
        let evt_bytes = match event {
            Ok(evt_bytes) => evt_bytes,
            Err(e) => {
                warn!("Error reading event: {}", e);
                continue;
            }
        };
        let data = evt_bytes.to_vec();
        let txt = String::from_utf8_lossy(&data);
        debug!("> RAW EVENT: {:?}", txt);
        for line in txt.lines() {
            let (key, value) = match line.split_once(":") {
                Some((key, value)) => (key, value),
                None => {
                    if line.trim() != "" {
                        warn!("Unexpected line format: {}", line);
                    }
                    continue;
                }
            };
            if key != "data" {
                warn!("Unexpected key: {}", key);
                continue;
            }
            if value.trim() == "[DONE]" {
                break;
            };
            let parsed = match serde_json::from_str::<serde_json::Value>(&value) {
                Ok(parsed) => parsed,
                Err(e) => {
                    warn!("<Error parsing JSON: {}>", e);
                    continue;
                }
            };
            debug!("> PARSED: {:?}", parsed);
            let choices = match parsed["choices"].as_array() {
                Some(choices) => choices,
                None => {
                    warn!("<No 'choices' in response: {:?}>", parsed);
                    continue;
                }
            };
            for c in choices {
                debug!("> CHOICE: {:?}", c);
                if let Some(delta) = c["delta"].as_object() {
                    if let Some(Some(next_bit)) = delta.get("content").map(|v| v.as_str())
                    {
                        out_queue.send(next_bit.to_string())?;
                        answer.push_str(next_bit);
                        total_tokens += 1;
                    }
                    if delta.get("finish_reason").is_some() {
                        break;
                    }
                }
            }
        }
    }
    Ok((answer, Some(total_tokens)))
}
