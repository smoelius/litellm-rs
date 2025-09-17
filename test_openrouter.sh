#!/bin/bash

# Test script for OpenRouter integration

echo "Testing OpenRouter integration..."
echo "================================"
echo

# Set your OpenRouter API key here or export it in your environment
# export OPENROUTER_API_KEY="your-api-key-here"

if [ -z "$OPENROUTER_API_KEY" ]; then
    echo "❌ Error: OPENROUTER_API_KEY environment variable is not set"
    echo "Please export your OpenRouter API key:"
    echo "  export OPENROUTER_API_KEY='your-api-key'"
    exit 1
fi

echo "✅ API Key found"
echo "Running OpenRouter example..."
echo

cargo run --example simple_openrouter