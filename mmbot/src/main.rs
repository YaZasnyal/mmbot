use mattermost_api::apis::configuration::Configuration;
use mattermost_bot::{async_trait, Event, Plugin};
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{
    self, ChatCompletionMessage, ChatCompletionRequest, FinishReason, Tool, ToolCall,
};
use openai_api_rs::v1::common::GPT4_O;
use openai_api_rs::v1::types::{Function, FunctionParameters, JSONSchemaDefine};
use serde_json::{Number, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::{env, result};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let mut cfg = mattermost_api::apis::configuration::Configuration::default();
    // cfg.base_path = "http://localhost:8065".to_string();
    // cfg.bearer_access_token = Some("8urtzsaxtjggpnu9iygqoeeepy".to_string());

    // // let mut bot = mattermost_bot::Bot::new(cfg);
    // // bot.add_plugin(ExamplePlugin {});
    // // bot.run().await;
    // let db = surrealdb::Surreal::new::<surrealdb::engine::remote::ws::Ws>("localhost:8000").await?;

    // let posts = mattermost_api::apis::posts_api::get_posts_for_channel(
    //     &cfg,
    //     "7a3zcps5qjgbm8hsf5ikusjgkw",
    //     None,
    //     Some(1000),
    //     None, //Some(1730312064000 as i64),
    //     None,
    //     None,
    //     None,
    // )
    // .await
    // .unwrap();

    // println!("{}", serde_json::to_string(&posts).unwrap());
    // db.use_ns("mmbot").use_db("mmbot").await?;

    // for (id, post) in posts.posts.unwrap() {
    //     let post: Option<Id> = db.upsert(("post", &id)).content(post).await.unwrap();
    // }

    let tools = vec![Tool {
        r#type: chat_completion::ToolType::Function,
        function: Function {
            name: "weather".to_string(),
            description: Some("Get weather for the location".to_string()),
            parameters: FunctionParameters {
                schema_type: openai_api_rs::v1::types::JSONSchemaType::Object,
                properties: Some(HashMap::from([(
                    "city".to_string(),
                    Box::new(JSONSchemaDefine {
                        schema_type: Some(openai_api_rs::v1::types::JSONSchemaType::String),
                        description: Some("City to get waetcher for".to_string()),
                        enum_values: None,
                        properties: None,
                        required: None,
                        items: None,
                    }),
                )])),
                required: Some(vec!["city".to_string()]),
            },
        },
    }];

    let client = OpenAIClient::new_with_endpoint("http://localhost:1234/v1".into(), "q".into());
    let mut messages = vec![
        chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::system,
            content: chat_completion::Content::Text(String::from("Твоя задача отвечать на вопросы пользователей. Для этого тебе предоставлены инструмент для получения погоды. ОТВЕЧАЙ НА РУССКОМ!")),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
        chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("Какая погода сейчас в Москве?")),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    loop {
        let req =
            ChatCompletionRequest::new(GPT4_O.to_string(), messages.clone()).tools(tools.clone()); //.temperature(0.1);
        let result = client.chat_completion(req).await?;

        let choice = result.choices.last().unwrap();
        messages.push(ChatCompletionMessage {
            role: choice.message.role.clone(),
            content: chat_completion::Content::Text(
                choice
                    .message
                    .content
                    .as_ref()
                    .map(String::from)
                    .unwrap_or_default(),
            ),
            name: choice.message.name.clone(),
            tool_calls: choice.message.tool_calls.clone(),
            tool_call_id: None,
        });

        match &choice.finish_reason {
            None | Some(FinishReason::stop) | Some(FinishReason::length) => {
                println!("Content: {:#?}", result.choices[0].message);
                break;
            }
            Some(FinishReason::tool_calls) => {
                for tool in choice.message.tool_calls.as_ref().unwrap() {
                    match tool.function.name.as_ref().unwrap().as_str() {
                        "weather" => {
                            messages.push(weather(tool).await.unwrap());
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
            e => {
                println!("unexpected reason: {:#?}", e);
                break;
            }
        };
    }

    // let e = openai_api_rs::v1::embedding::EmbeddingRequest::new(
    //     "bartowski/DeepSeek-R1-Distill-Qwen-14B-GGUF".to_string(),
    //     result.choices[0].message.content.as_ref().unwrap().clone(),
    // );
    // let e = client.embedding(e).await?;
    // println!("\n{:?}", e);

    Ok(())
}

async fn weather(tool: &ToolCall) -> anyhow::Result<ChatCompletionMessage> {
    let url = "https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&current_weather=true";
    let response: String = reqwest::get(url).await?.text().await?;

    println!("Weather data: {:?}", response);

    Ok(ChatCompletionMessage {
        role: chat_completion::MessageRole::tool,
        content: chat_completion::Content::Text(response.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: Some(tool.id.clone()),
    })
}

struct ChatCompletions {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Id {
    id: surrealdb::RecordId,
}

struct ExamplePlugin {}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn id(&self) -> &'static str {
        "ExamplePlugin"
    }

    fn filter(&self, _event: Arc<Event>) -> bool {
        // check channel id from config

        true
    }

    async fn process_thread(&self, event: Arc<Event>, config: Arc<Configuration>) {
        // check the filter (more detailed):
        //   check if thread is interested
        //     if root message - check for @s3duty
        //   check if thread is blocked
        //     get first message and check for specific emojis

        // check the type
        //   if new message -> process completion
        //   if new thread -> check if interested 
    }
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
