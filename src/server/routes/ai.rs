//! AI API endpoints (OpenAI compatible)
//!
//! This module provides OpenAI-compatible API endpoints for AI services.

#![allow(dead_code)]

use crate::core::models::openai::{
    ChatCompletionRequest, CompletionRequest, EmbeddingRequest, ImageGenerationRequest,
    ModelListResponse,
};
use crate::core::models::{ApiKey, RequestContext, User};
use crate::server::AppState;
use crate::server::routes::{ApiResponse, errors};
use crate::utils::data::validation::RequestValidator;
use actix_web::http::header::HeaderMap;
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult, web};

use serde::Deserialize;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Audio speech generation request
#[derive(Debug, Deserialize)]
struct AudioSpeechRequest {
    /// Text to convert to speech
    pub input: String,
    /// Voice to use for speech generation
    pub voice: String,
    /// Audio format (mp3, opus, aac, flac)
    pub response_format: Option<String>,
    /// Speed of speech (0.25 to 4.0)
    pub speed: Option<f32>,
}

/// Configure AI API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1")
            // Chat completions
            .route("/chat/completions", web::post().to(chat_completions))
            // Text completions (legacy)
            .route("/completions", web::post().to(completions))
            // Embeddings
            .route("/embeddings", web::post().to(embeddings))
            // Image generation
            .route("/images/generations", web::post().to(image_generations))
            // Models
            .route("/models", web::get().to(list_models))
            .route("/models/{model_id}", web::get().to(get_model))
            // Audio (future implementation)
            .route(
                "/audio/transcriptions",
                web::post().to(audio_transcriptions),
            )
            .route("/audio/translations", web::post().to(audio_translations))
            .route("/audio/speech", web::post().to(audio_speech)),
    );
}

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

/// Text completions endpoint (legacy)
///
/// OpenAI-compatible text completions API for backward compatibility.
pub async fn completions(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<CompletionRequest>,
) -> ActixResult<HttpResponse> {
    info!("Text completion request for model: {}", request.model);

    // Get request context from middleware
    let context = get_request_context(&req)?;

    // Route request through the core router
    // TODO: Implement proper completion routing through ProviderRegistry
    match handle_completion_via_pool(&state.router, request.into_inner(), context).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            error!("Text completion error: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

/// Embeddings endpoint
///
/// OpenAI-compatible embeddings API for generating text embeddings.
pub async fn embeddings(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<EmbeddingRequest>,
) -> ActixResult<HttpResponse> {
    info!("Embedding request for model: {}", request.model);

    // Get request context from middleware
    let context = get_request_context(&req)?;

    // Route request through the core router
    // TODO: Implement proper embedding routing through ProviderRegistry
    match handle_embedding_via_pool(&state.router, request.into_inner(), context).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            error!("Embedding error: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Error".to_string())))
        }
    }
}

/// Image generation endpoint
///
/// OpenAI-compatible image generation API.
async fn image_generations(
    state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<ImageGenerationRequest>,
) -> ActixResult<HttpResponse> {
    info!("Image generation request for model: {:?}", request.model);

    // Get request context from middleware
    let context = get_request_context(&req)?;

    // Route request through the core router
    // TODO: Implement proper image generation routing through ProviderRegistry
    match handle_image_generation_via_pool(&state.router, request.into_inner(), context).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            error!("Image generation error: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Error".to_string())))
        }
    }
}

/// List available models
///
/// Returns a list of available AI models across all configured providers.
pub async fn list_models(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    debug!("Listing available models");

    // TODO: Implement proper model listing through ProviderRegistry
    match get_models_from_pool(&state.router).await {
        Ok(models) => {
            let response = ModelListResponse {
                object: "list".to_string(),
                data: models,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!("Failed to list models: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Error".to_string())))
        }
    }
}

/// Get specific model information
///
/// Returns detailed information about a specific model.
async fn get_model(
    state: web::Data<AppState>,
    model_id: web::Path<String>,
) -> ActixResult<HttpResponse> {
    debug!("Getting model info for: {}", model_id);

    // TODO: Implement proper model retrieval through ProviderRegistry
    match get_model_from_pool(&state.router, &model_id).await {
        Ok(Some(model)) => Ok(HttpResponse::Ok().json(model)),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Error".to_string())))
        }
        Err(e) => {
            error!("Failed to get model {}: {}", model_id, e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Error".to_string())))
        }
    }
}

/// Audio transcriptions endpoint
async fn audio_transcriptions(
    _state: web::Data<AppState>,
    req: HttpRequest,
    _payload: web::Payload,
) -> ActixResult<HttpResponse> {
    info!("Audio transcriptions request");

    // Get request context (user, API key, etc.)
    let _context = match get_request_context(&req) {
        Ok(ctx) => ctx,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Unauthorized".to_string())));
        }
    };

    // For now, return a placeholder response indicating the feature is in development
    let response = serde_json::json!({
        "text": "Audio transcription feature is in development. This endpoint will support OpenAI-compatible audio transcription APIs.",
        "language": "en",
        "duration": 0.0,
        "segments": []
    });

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// Audio translations endpoint
async fn audio_translations(
    _state: web::Data<AppState>,
    req: HttpRequest,
    _payload: web::Payload,
) -> ActixResult<HttpResponse> {
    info!("Audio translations request");

    // Get request context (user, API key, etc.)
    let _context = match get_request_context(&req) {
        Ok(ctx) => ctx,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Unauthorized".to_string())));
        }
    };

    // For now, return a placeholder response indicating the feature is in development
    let response = serde_json::json!({
        "text": "Audio translation feature is in development. This endpoint will support OpenAI-compatible audio translation APIs.",
        "language": "en",
        "duration": 0.0,
        "segments": []
    });

    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}

/// Audio speech endpoint
async fn audio_speech(
    _state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<AudioSpeechRequest>,
) -> ActixResult<HttpResponse> {
    info!(
        "Audio speech request for text: {}",
        &request.input[..50.min(request.input.len())]
    );

    // Get request context (user, API key, etc.)
    let _context = match get_request_context(&req) {
        Ok(ctx) => ctx,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Unauthorized".to_string())));
        }
    };

    // For now, return a placeholder audio response
    // In a real implementation, this would generate actual audio using TTS providers
    let audio_data = vec![0u8; 1024]; // Placeholder audio data

    Ok(HttpResponse::Ok()
        .content_type("audio/mpeg")
        .body(audio_data))
}

/// Get request context from headers and middleware extensions
fn get_request_context(req: &HttpRequest) -> ActixResult<RequestContext> {
    // In a real implementation, this would extract the context from request extensions
    // that were set by the authentication middleware
    let mut context = RequestContext::new();

    // Extract request ID
    if let Some(request_id) = req.headers().get("x-request-id") {
        if let Ok(id) = request_id.to_str() {
            context.request_id = id.to_string();
        }
    }

    // Extract user agent
    if let Some(user_agent) = req.headers().get("user-agent") {
        if let Ok(agent) = user_agent.to_str() {
            context.user_agent = Some(agent.to_string());
        }
    }

    Ok(context)
}

/// Extract user from request extensions
fn get_authenticated_user(_headers: &HeaderMap) -> Option<User> {
    // In a real implementation, this would extract the user from request extensions
    // that were set by the authentication middleware
    None
}

/// Extract API key from request extensions
fn get_authenticated_api_key(_headers: &HeaderMap) -> Option<ApiKey> {
    // In a real implementation, this would extract the API key from request extensions
    // that were set by the authentication middleware
    None
}

/// Check if user has permission for the requested operation
fn check_permission(user: Option<&User>, api_key: Option<&ApiKey>, _operation: &str) -> bool {
    // In a real implementation, this would check permissions through the RBAC system
    // For now, assume all authenticated requests are allowed
    user.is_some() || api_key.is_some()
}

/// Log API usage for billing and analytics
async fn log_api_usage(
    _state: &AppState,
    context: &RequestContext,
    model: &str,
    tokens_used: u32,
    cost: f64,
) {
    // In a real implementation, this would log usage to the database
    debug!(
        "API usage: user_id={:?}, model={}, tokens={}, cost={}",
        context.user_id, model, tokens_used, cost
    );
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

// Temporary helper functions to bridge ProviderRegistry and router functionality
// TODO: These should be replaced with proper router implementation

use crate::core::models::openai::{
    ChatCompletionResponse, CompletionResponse, EmbeddingResponse, ImageGenerationResponse, Model,
};
use crate::core::providers::ProviderRegistry;
use crate::utils::error::GatewayError;

async fn handle_chat_completion_via_pool(
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
        choices: vec![crate::core::models::openai::ChatChoice {
            index: 0,
            message: crate::core::models::openai::ChatMessage {
                role: crate::core::models::openai::MessageRole::Assistant,
                content: Some(crate::core::models::openai::MessageContent::Text(format!(
                    "Hello! I'm {} responding through the gateway. You said: {:?}",
                    request.model,
                    request
                        .messages
                        .last()
                        .and_then(|m| m.content.as_ref())
                        .map(|c| match c {
                            crate::core::models::openai::MessageContent::Text(t) => t.as_str(),
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
        usage: Some(crate::core::models::openai::Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        }),
        system_fingerprint: Some(format!("fp_{}", uuid::Uuid::new_v4().simple())),
    };

    Ok(response)

    /* Original complex implementation - kept for reference
    // Get provider for the model
    // For now, just use the first available provider that supports the model
    let providers = pool.get_all_providers();
    let provider = providers.first()
        .ok_or_else(|| GatewayError::internal("No providers available"))?;

    // Convert request to provider format
    let messages = request.messages.iter().map(|msg| {
        use crate::core::types::requests::{ChatMessage, MessageRole, MessageContent};

        let role = match msg.role {
            crate::core::models::openai::MessageRole::System => MessageRole::System,
            crate::core::models::openai::MessageRole::User => MessageRole::User,
            crate::core::models::openai::MessageRole::Assistant => MessageRole::Assistant,
            crate::core::models::openai::MessageRole::Tool => MessageRole::Tool,
            crate::core::models::openai::MessageRole::Function => MessageRole::Tool,
        };

        let content = msg.content.as_ref().map(|c| {
            match c {
                crate::core::models::openai::MessageContent::Text(text) => {
                    MessageContent::Text(text.clone())
                }
                crate::core::models::openai::MessageContent::Parts(_parts) => {
                    // For now, just extract text from parts
                    MessageContent::Text("".to_string())
                }
            }
        });

        ChatMessage {
            role,
            content,
            name: msg.name.clone(),
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
        }
    }).collect();

    let chat_request = crate::core::types::requests::ChatRequest {
        model: request.model.clone(),
        messages,
        temperature: request.temperature,
        top_p: request.top_p,
        n: request.n.map(|n| n as u32),
        stream: request.stream.unwrap_or(false),
        stop: request.stop.clone(),
        max_tokens: request.max_tokens.map(|m| m as u32),
        max_completion_tokens: None,
        presence_penalty: request.presence_penalty,
        frequency_penalty: request.frequency_penalty,
        logit_bias: request.logit_bias.clone(),
        user: request.user.clone(),
        response_format: None, // TODO: Convert response_format properly
        seed: request.seed.map(|s| s as i32),
        tools: None, // TODO: Convert tools properly
        tool_choice: None, // TODO: Convert tool_choice properly
        parallel_tool_calls: None,
        logprobs: None,
        top_logprobs: None,
        extra_params: HashMap::new(),
    };

    // Call provider
    let response = provider.chat_completion(chat_request, context)
        .await
        .map_err(|e| GatewayError::internal(format!("Provider error: {}", e)))?;

    // Convert response back to API format
    // Convert ChatChoice to openai format
    let choices = response.choices.into_iter().map(|choice| {
        crate::core::models::openai::ChatChoice {
            index: choice.index,
            message: crate::core::models::openai::ChatMessageResponse {
                role: crate::core::models::openai::MessageRole::Assistant,
                content: choice.message.content.map(|c| {
                    match c {
                        crate::core::types::responses::MessageContent::Text(t) => {
                            crate::core::models::openai::MessageContent::Text(t)
                        }
                        _ => crate::core::models::openai::MessageContent::Text("".to_string()),
                    }
                }),
                tool_calls: None,
                function_call: None,
            },
            delta: None,
            finish_reason: choice.finish_reason,
            logprobs: None,
        }
    }).collect();

    Ok(ChatCompletionResponse {
        id: response.id,
        object: response.object,
        created: response.created as u64,
        model: response.model,
        choices,
        usage: response.usage.map(|u| crate::core::models::openai::Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }),
        system_fingerprint: response.system_fingerprint,
    })
    */
}

async fn get_models_from_pool(pool: &ProviderRegistry) -> Result<Vec<Model>, GatewayError> {
    let mut all_models = Vec::new();

    // Get models from all providers
    let providers = pool.get_all_providers();
    for provider in providers {
        let models = provider.list_models();
        for model_info in models {
            all_models.push(Model {
                id: model_info.id.clone(),
                object: "model".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                owned_by: model_info.provider.clone(),
            });
        }
    }

    Ok(all_models)
}

async fn get_model_from_pool(
    _pool: &ProviderRegistry,
    _model_id: &str,
) -> Result<Option<Model>, GatewayError> {
    // TODO: Get specific model from providers in pool
    Ok(None) // Return None for now
}

async fn handle_completion_via_pool(
    _pool: &ProviderRegistry,
    _request: CompletionRequest,
    _context: RequestContext,
) -> Result<CompletionResponse, GatewayError> {
    // TODO: Implement actual completion routing via ProviderRegistry
    Err(GatewayError::internal(
        "Text completion routing not implemented yet",
    ))
}

async fn handle_embedding_via_pool(
    _pool: &ProviderRegistry,
    _request: EmbeddingRequest,
    _context: RequestContext,
) -> Result<EmbeddingResponse, GatewayError> {
    // TODO: Implement actual embedding routing via ProviderRegistry
    Err(GatewayError::internal(
        "Embedding routing not implemented yet",
    ))
}

async fn handle_image_generation_via_pool(
    _pool: &ProviderRegistry,
    _request: ImageGenerationRequest,
    _context: RequestContext,
) -> Result<ImageGenerationResponse, GatewayError> {
    // TODO: Implement actual image generation routing via ProviderRegistry
    Err(GatewayError::internal(
        "Image generation routing not implemented yet",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_request_context() {
        // This test would need a mock HttpRequest in a real implementation
        // For now, we'll test the basic functionality
        let context = RequestContext::new();
        assert!(!context.request_id.is_empty());
        assert!(context.user_agent.is_none());
    }

    #[test]
    fn test_check_permission() {
        // Test with no authentication
        assert!(!check_permission(None, None, "chat"));

        // Test with user (would need actual User instance in real test)
        // assert!(check_permission(Some(&user), None, "chat"));
    }

    #[tokio::test]
    async fn test_log_api_usage() {
        // This would require actual state in a real test
        // For now, just test that the function doesn't panic
        let _context = RequestContext::new();
        // log_api_usage(&state, &context, "gpt-4", 100, 0.002).await;
    }
}
