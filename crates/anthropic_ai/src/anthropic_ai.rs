use anyhow::{anyhow, Result};
use futures::{io::BufReader, stream::BoxStream, AsyncBufReadExt, AsyncReadExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::str;
use util::http::{AsyncBody, HttpClient, Request};

#[derive(Debug, Default, Serialize)]
pub enum Model {
    #[default]
    #[serde(rename = "claude-3-opus-20240229")]
    ClaudeOpus,
    #[serde(rename = "claude-3-sonnet-20240229")]
    ClaudeSonnet,
    #[serde(rename = "claude-3-haiku-20240307")]
    ClaudeHaiku,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Default, Serialize)]
pub struct MessagesRequest {
    pub model: Model,
    pub messages: Vec<RequestMessage>,
    pub max_tokens: usize,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct RequestMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageEvent {
    MessageStart {
        message: ResponseMessage,
    },
    MessageDelta {
        delta: MessageDelta,
    },
    MessageStop,
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },
    ContentBlockDelta {
        index: usize,
        delta: ContentBlockDelta,
    },
    ContentBlockStop {
        index: usize,
    },
    Ping,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    id: String,
    role: Role,
    content: Vec<String>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct MessageDelta {
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlockDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    input_tokens: usize,
    output_tokens: usize,
}

pub async fn stream_messages<T: HttpClient>(
    client: &T,
    api_key: &str,
    request: MessagesRequest,
) -> Result<BoxStream<'static, Result<MessageEvent>>> {
    let request = Request::builder()
        .uri("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "messages-2023-12-15") // todo!("needed?")
        .header("x-api-key", api_key)
        .body(AsyncBody::from(serde_json::to_string(&request)?))?;
    let mut response = client.send(request).await?;

    if response.status().is_success() {
        let reader = BufReader::new(response.into_body());
        Ok(reader
            .lines()
            .filter_map(|line| async move {
                match line {
                    Ok(line) if line.starts_with("data: ") => {
                        let data = line.trim_start_matches("data: ");
                        match serde_json::from_str(data) {
                            Ok(event) => Some(Ok(event)),
                            Err(error) => Some(Err(anyhow!(error))),
                        }
                    }
                    Ok(_) => None,
                    Err(error) => Some(Err(anyhow!(error))),
                }
            })
            .boxed())
    } else {
        let mut text = String::new();
        response.body_mut().read_to_string(&mut text).await?;
        Err(anyhow!(
            "error during stream_messages, status code: {:?}, body: {}",
            response.status(),
            text
        ))
    }
}
