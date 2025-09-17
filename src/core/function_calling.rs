//! Function calling support for AI providers
//!
//! This module provides OpenAI-compatible function calling capabilities.

use crate::core::models::openai::*;
use crate::utils::error::{GatewayError, Result};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{error, warn};

/// Function definition for AI models
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    pub description: Option<String>,
    /// Function parameters schema (JSON Schema)
    pub parameters: Value,
    /// Whether the function is strict (OpenAI specific)
    pub strict: Option<bool>,
}

/// Tool definition for AI models
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    /// Tool type (currently only "function")
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function definition
    pub function: FunctionDefinition,
}

/// Tool choice options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// No tools should be called
    None,
    /// Let the model decide
    Auto,
    /// Force a specific tool
    Required,
    /// Specific tool to use
    Specific {
        /// Tool type identifier
        #[serde(rename = "type")]
        tool_type: String,
        /// Function choice details
        function: FunctionChoice,
    },
}

/// Specific function choice
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionChoice {
    /// Function name to call
    pub name: String,
}

/// Function call in a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments (JSON string)
    pub arguments: String,
}

/// Tool call in a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function call details
    pub function: FunctionCall,
}

/// Function calling handler
pub struct FunctionCallingHandler {
    /// Available functions
    functions: HashMap<String, FunctionDefinition>,
    /// Function execution handlers
    executors: HashMap<String, Box<dyn FunctionExecutor>>,
}

/// Trait for executing functions
#[async_trait::async_trait]
pub trait FunctionExecutor: Send + Sync {
    /// Execute the function with given arguments
    async fn execute(&self, arguments: Value) -> Result<Value>;

    /// Get function schema
    fn get_schema(&self) -> FunctionDefinition;

    /// Validate function arguments
    fn validate_arguments(&self, _arguments: &Value) -> Result<()> {
        // Default implementation - can be overridden
        Ok(())
    }
}

impl FunctionCallingHandler {
    /// Create a new function calling handler
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            executors: HashMap::new(),
        }
    }

    /// Register a function
    pub fn register_function<F>(&mut self, name: String, executor: F) -> Result<()>
    where
        F: FunctionExecutor + 'static,
    {
        let schema = executor.get_schema();
        self.functions.insert(name.clone(), schema);
        self.executors.insert(name, Box::new(executor));
        Ok(())
    }

    /// Get available functions as tool definitions
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.functions
            .values()
            .map(|function| ToolDefinition {
                tool_type: "function".to_string(),
                function: function.clone(),
            })
            .collect()
    }

    /// Process tool calls in a chat completion request
    pub async fn process_tool_calls(&self, tool_calls: &[ToolCall]) -> Result<Vec<ChatMessage>> {
        let mut tool_responses = Vec::new();

        for tool_call in tool_calls {
            if tool_call.tool_type != "function" {
                warn!("Unsupported tool type: {}", tool_call.tool_type);
                continue;
            }

            let function_name = &tool_call.function.name;

            if let Some(executor) = self.executors.get(function_name) {
                // Parse function arguments
                let arguments: Value = match serde_json::from_str(&tool_call.function.arguments) {
                    Ok(args) => args,
                    Err(e) => {
                        error!("Failed to parse function arguments: {}", e);
                        return Err(GatewayError::Validation(format!(
                            "Invalid function arguments: {}",
                            e
                        )));
                    }
                };

                // Validate arguments
                if let Err(e) = executor.validate_arguments(&arguments) {
                    error!("Function argument validation failed: {}", e);
                    return Err(e);
                }

                // Execute function
                match executor.execute(arguments).await {
                    Ok(result) => {
                        let tool_message = ChatMessage {
                            role: MessageRole::Tool,
                            content: Some(MessageContent::Text(result.to_string())),
                            name: Some(function_name.clone()),
                            function_call: None,
                            tool_calls: None,
                            tool_call_id: None,
                            audio: None,
                        };
                        tool_responses.push(tool_message);
                    }
                    Err(e) => {
                        error!("Function execution failed: {}", e);
                        let error_message = ChatMessage {
                            role: MessageRole::Tool,
                            content: Some(MessageContent::Text(format!("Error: {}", e))),
                            name: Some(function_name.clone()),
                            function_call: None,
                            tool_calls: None,
                            tool_call_id: None,
                            audio: None,
                        };
                        tool_responses.push(error_message);
                    }
                }
            } else {
                warn!("Unknown function: {}", function_name);
                let error_message = ChatMessage {
                    role: MessageRole::Tool,
                    content: Some(MessageContent::Text(format!(
                        "Unknown function: {}",
                        function_name
                    ))),
                    name: Some(function_name.clone()),
                    function_call: None,
                    tool_calls: None,
                    tool_call_id: None,
                    audio: None,
                };
                tool_responses.push(error_message);
            }
        }

        Ok(tool_responses)
    }

    /// Convert function definitions to provider-specific format
    pub fn convert_tools_for_provider(
        &self,
        provider_type: &crate::core::providers::ProviderType,
        tools: &[ToolDefinition],
    ) -> Result<Value> {
        match provider_type {
            crate::core::providers::ProviderType::OpenAI
            | crate::core::providers::ProviderType::Azure => {
                // OpenAI format
                Ok(json!(tools))
            }
            crate::core::providers::ProviderType::Anthropic => {
                // Anthropic format
                let anthropic_tools: Vec<Value> = tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "name": tool.function.name,
                            "description": tool.function.description,
                            "input_schema": tool.function.parameters
                        })
                    })
                    .collect();
                Ok(json!(anthropic_tools))
            }
            crate::core::providers::ProviderType::Google => {
                // Google format
                let google_tools: Vec<Value> = tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "function_declarations": [{
                                "name": tool.function.name,
                                "description": tool.function.description,
                                "parameters": tool.function.parameters
                            }]
                        })
                    })
                    .collect();
                Ok(json!(google_tools))
            }
            _ => Err(GatewayError::bad_request(format!(
                "Function calling not supported for provider: {:?}",
                provider_type
            ))),
        }
    }

    /// Extract tool calls from provider response
    pub fn extract_tool_calls_from_response(
        &self,
        provider_type: &crate::core::providers::ProviderType,
        response: &Value,
    ) -> Result<Vec<ToolCall>> {
        match provider_type {
            crate::core::providers::ProviderType::OpenAI
            | crate::core::providers::ProviderType::Azure => {
                // OpenAI format
                if let Some(choices) = response.get("choices").and_then(|c| c.as_array()) {
                    if let Some(choice) = choices.first() {
                        if let Some(message) = choice.get("message") {
                            if let Some(tool_calls) = message.get("tool_calls") {
                                let tool_calls: Vec<ToolCall> =
                                    serde_json::from_value(tool_calls.clone())?;
                                return Ok(tool_calls);
                            }
                        }
                    }
                }
                Ok(vec![])
            }
            crate::core::providers::ProviderType::Anthropic => {
                // Anthropic format
                if let Some(content) = response.get("content").and_then(|c| c.as_array()) {
                    let mut tool_calls = Vec::new();
                    for item in content {
                        if let Some(tool_type) = item.get("type").and_then(|t| t.as_str()) {
                            if tool_type == "tool_use" {
                                if let (Some(id), Some(name), Some(input)) = (
                                    item.get("id").and_then(|i| i.as_str()),
                                    item.get("name").and_then(|n| n.as_str()),
                                    item.get("input"),
                                ) {
                                    tool_calls.push(ToolCall {
                                        id: id.to_string(),
                                        tool_type: "function".to_string(),
                                        function: FunctionCall {
                                            name: name.to_string(),
                                            arguments: input.to_string(),
                                        },
                                    });
                                }
                            }
                        }
                    }
                    return Ok(tool_calls);
                }
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }
}

impl Default for FunctionCallingHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in function executors
pub mod builtin {
    use super::*;

    /// Weather function executor (example)
    pub struct WeatherFunction;

    #[async_trait::async_trait]
    impl FunctionExecutor for WeatherFunction {
        async fn execute(&self, arguments: Value) -> Result<Value> {
            let location = arguments
                .get("location")
                .and_then(|l| l.as_str())
                .ok_or_else(|| {
                    GatewayError::Validation("Missing location parameter".to_string())
                })?;

            // Mock weather data
            let weather_data = json!({
                "location": location,
                "temperature": "22°C",
                "condition": "Sunny",
                "humidity": "65%",
                "wind": "10 km/h"
            });

            Ok(weather_data)
        }

        fn get_schema(&self) -> FunctionDefinition {
            FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get current weather information for a location".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city and state, e.g. San Francisco, CA"
                        }
                    },
                    "required": ["location"]
                }),
                strict: Some(false),
            }
        }

        fn validate_arguments(&self, arguments: &Value) -> Result<()> {
            if !arguments.is_object() {
                return Err(GatewayError::Validation(
                    "Arguments must be an object".to_string(),
                ));
            }

            if arguments.get("location").is_none() {
                return Err(GatewayError::Validation(
                    "Missing required parameter: location".to_string(),
                ));
            }

            Ok(())
        }
    }

    /// Calculator function executor (example)
    pub struct CalculatorFunction;

    #[async_trait::async_trait]
    impl FunctionExecutor for CalculatorFunction {
        async fn execute(&self, arguments: Value) -> Result<Value> {
            let expression = arguments
                .get("expression")
                .and_then(|e| e.as_str())
                .ok_or_else(|| {
                    GatewayError::Validation("Missing expression parameter".to_string())
                })?;

            // Simple calculator (in real implementation, use a proper expression parser)
            let result =
                match expression {
                    expr if expr.contains("+") => {
                        let parts: Vec<&str> = expr.split('+').collect();
                        if parts.len() == 2 {
                            let a: f64 = parts[0].trim().parse().map_err(|_| {
                                GatewayError::Validation("Invalid number".to_string())
                            })?;
                            let b: f64 = parts[1].trim().parse().map_err(|_| {
                                GatewayError::Validation("Invalid number".to_string())
                            })?;
                            a + b
                        } else {
                            return Err(GatewayError::Validation("Invalid expression".to_string()));
                        }
                    }
                    _ => {
                        return Err(GatewayError::Validation(
                            "Unsupported operation".to_string(),
                        ));
                    }
                };

            Ok(json!({
                "expression": expression,
                "result": result
            }))
        }

        fn get_schema(&self) -> FunctionDefinition {
            FunctionDefinition {
                name: "calculate".to_string(),
                description: Some("Perform basic mathematical calculations".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Mathematical expression to evaluate (e.g., '2 + 3')"
                        }
                    },
                    "required": ["expression"]
                }),
                strict: Some(false),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::builtin::*;
    use super::*;

    #[test]
    fn test_function_calling_handler_creation() {
        let handler = FunctionCallingHandler::new();
        assert!(handler.functions.is_empty());
        assert!(handler.executors.is_empty());
    }

    #[tokio::test]
    async fn test_weather_function() {
        let weather_fn = WeatherFunction;
        let args = json!({"location": "San Francisco, CA"});

        let result = weather_fn.execute(args).await.unwrap();
        assert!(result.get("location").is_some());
        assert!(result.get("temperature").is_some());
    }

    #[tokio::test]
    async fn test_calculator_function() {
        let calc_fn = CalculatorFunction;
        let args = json!({"expression": "2 + 3"});

        let result = calc_fn.execute(args).await.unwrap();
        assert_eq!(result.get("result").unwrap().as_f64().unwrap(), 5.0);
    }

    #[test]
    fn test_function_registration() {
        let mut handler = FunctionCallingHandler::new();
        let weather_fn = WeatherFunction;

        handler
            .register_function("get_weather".to_string(), weather_fn)
            .unwrap();
        assert_eq!(handler.functions.len(), 1);
        assert_eq!(handler.executors.len(), 1);
    }

    #[test]
    fn test_tool_definitions() {
        let mut handler = FunctionCallingHandler::new();
        let weather_fn = WeatherFunction;

        handler
            .register_function("get_weather".to_string(), weather_fn)
            .unwrap();
        let tools = handler.get_tool_definitions();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.name, "get_weather");
        assert_eq!(tools[0].tool_type, "function");
    }
}
