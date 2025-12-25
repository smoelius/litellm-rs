//! Chat completions endpoint

use crate::core::completion::{CompletionOptions, completion_stream};
use crate::core::models::RequestContext;
use crate::core::models::openai::{
    ChatChoice, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, MessageContent,
    MessageRole, Usage,
};
use crate::core::providers::ProviderRegistry;
use crate::core::streaming::types::{
    ChatCompletionChunk, ChatCompletionChunkChoice, ChatCompletionDelta, Event,
};
use crate::server::routes::errors;
use crate::server::state::AppState;
use crate::utils::data::validation::RequestValidator;
use crate::utils::error::GatewayError;
use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};
use futures::StreamExt;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::context::get_request_context;

/// Chat completions endpoint
///
/// OpenAI-compatible chat completions API that supports streaming and non-streaming responses.
pub async fn chat_completions(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<ChatCompletionRequest>,
) -> ActixResult<HttpResponse> {
    info!("Chat completion request for model: {}", request.model);

    // Get request context from middleware
    let context = get_request_context(&req)?;

    // Validate request
    if let Err(e) = RequestValidator::validate_chat_completion_request(
        &request.model,
        &request.messages,
        request.max_tokens,
        request.temperature,
    ) {
        warn!("Invalid chat completion request: {}", e);
        return Ok(errors::validation_error(&e.to_string()));
    }

    // Check if streaming is requested
    if request.stream.unwrap_or(false) {
        // Handle streaming request
        handle_streaming_chat_completion(state.get_ref(), request.into_inner(), context).await
    } else {
        // Handle non-streaming request
        // TODO: Implement proper routing through ProviderRegistry
        match handle_chat_completion_via_pool(&state.router, request.into_inner(), context).await {
            Ok(response) => Ok(HttpResponse::Ok().json(response)),
            Err(e) => {
                error!("Chat completion error: {}", e);
                Ok(errors::gateway_error_to_response(e))
            }
        }
    }
}

/// Handle streaming chat completion
async fn handle_streaming_chat_completion(
    _state: &AppState,
    request: ChatCompletionRequest,
    _context: RequestContext,
) -> ActixResult<HttpResponse> {
    info!(
        "Handling streaming chat completion for model: {}",
        request.model
    );

    // Convert ChatCompletionRequest messages to core Message format
    let messages: Vec<crate::core::types::ChatMessage> = request
        .messages
        .into_iter()
        .map(|msg| {
            // Convert MessageRole
            let role = match msg.role {
                MessageRole::System => crate::core::types::MessageRole::System,
                MessageRole::User => crate::core::types::MessageRole::User,
                MessageRole::Assistant => crate::core::types::MessageRole::Assistant,
                MessageRole::Tool => crate::core::types::MessageRole::Tool,
                MessageRole::Function => crate::core::types::MessageRole::Function,
            };

            // Convert MessageContent
            let content = msg.content.map(|c| match c {
                MessageContent::Text(t) => crate::core::types::MessageContent::Text(t),
                MessageContent::Parts(parts) => {
                    let converted_parts: Vec<crate::core::types::ContentPart> = parts
                        .into_iter()
                        .map(|p| match p {
                            crate::core::models::openai::ContentPart::Text { text } => {
                                crate::core::types::ContentPart::Text { text }
                            }
                            crate::core::models::openai::ContentPart::ImageUrl { image_url } => {
                                crate::core::types::ContentPart::ImageUrl {
                                    image_url: crate::core::types::content::ImageUrl {
                                        url: image_url.url,
                                        detail: image_url.detail,
                                    },
                                }
                            }
                            crate::core::models::openai::ContentPart::Audio { audio } => {
                                // For audio, we'll use text as a fallback since audio types differ
                                crate::core::types::ContentPart::Text {
                                    text: format!("[audio: {:?}]", audio),
                                }
                            }
                        })
                        .collect();
                    crate::core::types::MessageContent::Parts(converted_parts)
                }
            });

            // Convert tool calls
            let tool_calls = msg.tool_calls.map(|tcs| {
                tcs.into_iter()
                    .map(|tc| crate::core::types::ToolCall {
                        id: tc.id,
                        tool_type: tc.tool_type,
                        function: crate::core::types::FunctionCall {
                            name: tc.function.name,
                            arguments: tc.function.arguments,
                        },
                    })
                    .collect()
            });

            // Convert function call (legacy)
            let function_call = msg
                .function_call
                .map(|fc| crate::core::types::FunctionCall {
                    name: fc.name,
                    arguments: fc.arguments,
                });

            crate::core::types::ChatMessage {
                role,
                content,
                thinking: None,
                name: msg.name,
                tool_calls,
                tool_call_id: msg.tool_call_id,
                function_call,
            }
        })
        .collect();

    // Build completion options from request
    let options = CompletionOptions {
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        top_p: request.top_p,
        frequency_penalty: request.frequency_penalty,
        presence_penalty: request.presence_penalty,
        stop: request.stop,
        stream: true,
        user: request.user,
        seed: request.seed.map(|s| s as i32),
        n: request.n,
        logprobs: request.logprobs,
        top_logprobs: request.top_logprobs,
        ..Default::default()
    };

    // Get the streaming response from core layer
    let stream_result = completion_stream(&request.model, messages, Some(options)).await;

    match stream_result {
        Ok(mut stream) => {
            let request_id = format!("chatcmpl-{}", Uuid::new_v4());
            let model = request.model.clone();
            let created = chrono::Utc::now().timestamp() as u64;

            // Create SSE stream that converts CompletionChunks to SSE events
            let sse_stream = async_stream::stream! {
                let mut is_first_chunk = true;

                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            // Convert CompletionChunk to ChatCompletionChunk (OpenAI format)
                            let chat_chunk = ChatCompletionChunk {
                                id: request_id.clone(),
                                object: "chat.completion.chunk".to_string(),
                                created,
                                model: model.clone(),
                                system_fingerprint: None,
                                choices: chunk.choices.into_iter().map(|c| {
                                    ChatCompletionChunkChoice {
                                        index: c.index,
                                        delta: ChatCompletionDelta {
                                            role: if is_first_chunk {
                                                Some(crate::core::types::MessageRole::Assistant)
                                            } else {
                                                None
                                            },
                                            content: c.delta.content,
                                            tool_calls: None,
                                        },
                                        finish_reason: c.finish_reason.map(|fr| format!("{:?}", fr).to_lowercase()),
                                        logprobs: None,
                                    }
                                }).collect(),
                                usage: None,
                            };

                            is_first_chunk = false;

                            // Serialize to SSE event
                            match serde_json::to_string(&chat_chunk) {
                                Ok(json) => {
                                    let event = Event::default().data(&json);
                                    yield Ok::<_, GatewayError>(event.to_bytes());
                                }
                                Err(e) => {
                                    error!("Failed to serialize chunk: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Stream error: {}", e);
                            let error_event = Event::default()
                                .event("error")
                                .data(&format!("{{\"error\": \"{}\"}}", e));
                            yield Ok::<_, GatewayError>(error_event.to_bytes());
                            break;
                        }
                    }
                }

                // Send [DONE] event
                let done_event = Event::default().data("[DONE]");
                yield Ok::<_, GatewayError>(done_event.to_bytes());
            };

            Ok(HttpResponse::Ok()
                .insert_header((CONTENT_TYPE, "text/event-stream"))
                .insert_header((CACHE_CONTROL, "no-cache"))
                .insert_header(("Connection", "keep-alive"))
                .streaming(sse_stream))
        }
        Err(e) => {
            error!("Failed to create streaming response: {}", e);
            Ok(errors::gateway_error_to_response(e))
        }
    }
}

/// Handle chat completion via provider pool
pub async fn handle_chat_completion_via_pool(
    pool: &ProviderRegistry,
    request: ChatCompletionRequest,
    _context: RequestContext,
) -> Result<ChatCompletionResponse, GatewayError> {
    // Simplified working implementation

    // Get the appropriate provider based on model
    let _provider = if request.model.starts_with("claude") {
        // Use Anthropic provider for Claude models
        pool.get_provider("anthropic")
            .ok_or_else(|| GatewayError::internal("Anthropic provider not available"))?
    } else {
        // Use OpenAI provider for all other models
        pool.get_provider("openai")
            .ok_or_else(|| GatewayError::internal("OpenAI provider not available"))?
    };

    // For now, return a mock response to test the system
    let response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp() as u64,
        model: request.model.clone(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: Some(MessageContent::Text(format!(
                    "Hello! I'm {} responding through the gateway. You said: {:?}",
                    request.model,
                    request
                        .messages
                        .last()
                        .and_then(|m| m.content.as_ref())
                        .map(|c| match c {
                            MessageContent::Text(t) => t.as_str(),
                            _ => "[non-text content]",
                        })
                        .unwrap_or("[no message]")
                ))),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            },
            finish_reason: Some("stop".to_string()),
            logprobs: None,
        }],
        usage: Some(Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        }),
        system_fingerprint: Some(format!("fp_{}", uuid::Uuid::new_v4().simple())),
    };

    Ok(response)
}
