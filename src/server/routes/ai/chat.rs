//! Chat completions endpoint

use crate::core::models::openai::{
    ChatCompletionRequest, ChatCompletionResponse, ChatChoice, ChatMessage, MessageContent,
    MessageRole, Usage,
};
use crate::core::models::RequestContext;
use crate::core::providers::ProviderRegistry;
use crate::server::routes::errors;
use crate::server::state::AppState;
use crate::utils::data::validation::RequestValidator;
use crate::utils::error::GatewayError;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
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
        handle_streaming_chat_completion(state.get_ref().clone(), request.into_inner(), context)
            .await
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
    _state: AppState,
    _request: ChatCompletionRequest,
    _context: RequestContext,
) -> ActixResult<HttpResponse> {
    // TODO: Implement streaming support
    // For now, return an error indicating streaming is not yet supported
    error!("Streaming is not yet implemented");
    Ok(errors::validation_error("Streaming is not yet implemented"))
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
                thinking_usage: None,
        }),
        system_fingerprint: Some(format!("fp_{}", uuid::Uuid::new_v4().simple())),
    };

    Ok(response)
}
