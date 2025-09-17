#!/bin/bash

# Azure AI Provider Test Script
# 
# This script demonstrates how to test the Azure AI provider
# Replace the placeholders with your actual credentials

echo "🚀 Azure AI Provider Test"
echo "========================="
echo ""

# Check if environment variables are set
if [ -z "$AZURE_AI_API_KEY" ]; then
    echo "⚠️  AZURE_AI_API_KEY is not set"
    echo ""
    echo "Please set your Azure AI credentials:"
    echo "  export AZURE_AI_API_KEY='your-api-key'"
    echo "  export AZURE_AI_API_BASE='https://your-resource.cognitiveservices.azure.com'"
    echo ""
    echo "Or for testing, you can run:"
    echo "  AZURE_AI_API_KEY='your-key' AZURE_AI_API_BASE='your-url' ./test_azure_ai.sh"
    echo ""
    exit 1
fi

if [ -z "$AZURE_AI_API_BASE" ]; then
    echo "⚠️  AZURE_AI_API_BASE is not set"
    echo ""
    echo "Please set your Azure AI endpoint:"
    echo "  export AZURE_AI_API_BASE='https://your-resource.cognitiveservices.azure.com'"
    echo ""
    exit 1
fi

echo "✅ Environment variables detected"
echo "   API Base: $AZURE_AI_API_BASE"
echo ""

# Build the examples
echo "📦 Building examples..."
cargo build --examples --quiet

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo ""

# Run the simple test
echo "🧪 Running simple test..."
echo "-------------------------"
cargo run --example azure_ai_simple --quiet

echo ""
echo ""

# Ask if user wants to run the full example
read -p "Would you like to run the full example with streaming? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "🎯 Running full example..."
    echo "-------------------------"
    cargo run --example azure_ai_chat --quiet
fi

echo ""
echo "✨ Test complete!"