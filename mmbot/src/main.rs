use mattermost_api::apis::configuration::Configuration;
use mattermost_bot::{async_trait, Plugin};
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_O;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cfg = mattermost_api::apis::configuration::Configuration::default();
    cfg.base_path = "http://localhost:8065".to_string();
    cfg.bearer_access_token = Some("8urtzsaxtjggpnu9iygqoeeepy".to_string());

    let mut bot = mattermost_bot::Bot::new(cfg);
    bot.add_plugin(ExamplePlugin {});
    bot.run().await;

    Ok(())
}

struct ExamplePlugin {}

#[async_trait]
impl Plugin for ExamplePlugin {
    async fn process_event(&self, event: Arc<Event>, config: Arc<Configuration>) {}
}

// let client = OpenAIClient::new_with_endpoint("http://localhost:1234/v1".into(), "q".into());

// let req = ChatCompletionRequest::new(
//     GPT4_O.to_string(),
//     vec![chat_completion::ChatCompletionMessage {
//         role: chat_completion::MessageRole::user,
//         content: chat_completion::Content::Text(String::from("What is bitcoin? ОТВЕЧАЙ НА РУССКОМ!")),
//         name: None,
//         tool_calls: None,
//         tool_call_id: None,
//     }],
// );
// let result = client.chat_completion(req).await?;
// println!("Content: {:?}", result.choices[0].message.content);

// let e = openai_api_rs::v1::embedding::EmbeddingRequest::new(
//     "lmstudio-community/Meta-Llama-3.1-8B-Instruct-GGUF/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf".to_string(),
//     result.choices[0].message.content.as_ref().unwrap().clone(),
// );
// let e = client.embedding(e).await?;
// println!("\n{:?}", e);
