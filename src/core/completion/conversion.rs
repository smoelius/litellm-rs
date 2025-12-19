//! Type conversion functions

use super::types::{Choice, CompletionOptions, CompletionResponse};
use crate::core::types::{ChatMessage, ChatRequest, ChatResponse, Usage};
use crate::utils::error::Result;

/// Convert to chat completion request
pub fn convert_to_chat_completion_request(
    model: &str,
    messages: Vec<ChatMessage>,
    options: CompletionOptions,
) -> Result<ChatRequest> {
    Ok(ChatRequest {
        model: model.to_string(),
        messages,
        temperature: options.temperature,
        max_tokens: options.max_tokens,
        max_completion_tokens: None,
        top_p: options.top_p,
        frequency_penalty: options.frequency_penalty,
        presence_penalty: options.presence_penalty,
        stop: options.stop,
        stream: options.stream,
        tools: None,
        tool_choice: None,
        parallel_tool_calls: None,
        response_format: None,
        user: options.user,
        seed: options.seed,
        n: options.n,
        logit_bias: None,
        functions: None,
        function_call: None,
        logprobs: options.logprobs,
        top_logprobs: options.top_logprobs,
        thinking: None,
        extra_params: options.extra_params,
    })
}

/// Convert from chat completion response
pub fn convert_from_chat_completion_response(response: ChatResponse) -> Result<CompletionResponse> {
    let choices = response
        .choices
        .into_iter()
        .map(|choice| Choice {
            index: choice.index,
            message: choice.message,
            finish_reason: choice.finish_reason,
        })
        .collect();

    Ok(CompletionResponse {
        id: response.id,
        object: response.object,
        created: response.created,
        model: response.model,
        choices,
        usage: response.usage,
    })
}

/// Convert from usage response
pub fn convert_usage(usage: &crate::core::types::Usage) -> Usage {
    Usage {
        prompt_tokens: usage.prompt_tokens,
        completion_tokens: usage.completion_tokens,
        total_tokens: usage.total_tokens,
        prompt_tokens_details: None,
        completion_tokens_details: None,
        thinking_usage: usage.thinking_usage.clone(),
    }
}
