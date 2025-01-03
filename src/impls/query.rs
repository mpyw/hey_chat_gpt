use reqwest::blocking::{Client, RequestBuilder};
use std::fs;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RequestBody {
    model: String,
    messages: Vec<Message>,
    seed: u64,
    max_completion_tokens: Option<u64>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Choice {
    index: u64,
    message: Message,
    finish_reason: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ResponseBody {
    id: String,
    object: String,
    created: u64,
    choices: Vec<Choice>,
    usage: Usage,
}

fn common_header(api_key: &str) -> RequestBuilder {
    let api_key_field = format!("Bearer {}", api_key);

    Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", api_key_field.as_str())
}

pub fn query(
    api_key: &str,
    model: String,
    input_messages: &[Message],
    seed: u64,
    max_completion_tokens: Option<u64>,
    cache_path: &std::path::Path,
) -> anyhow::Result<Message> {
    let response_body = common_header(api_key)
        .json(&RequestBody {
            model,
            messages: Vec::from(input_messages),
            seed: seed % 9223372036854775807,
            max_completion_tokens,
        })
        .send()?;

    let body = response_body.text()?;

    let mut response_body = match serde_json::from_str::<ResponseBody>(&body) {
        Ok(parsed) => parsed,
        Err(e) => {
            let res = format!("---\n{}\n---\n{}", e, body);
            fs::write(cache_path, res).unwrap_or(());
            return Err(e.into());
        }
    };

    let res = response_body.choices.remove(0).message;
    Ok(res)
}
