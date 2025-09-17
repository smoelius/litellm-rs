#!/bin/bash

# LiteLLM Gateway API 测试脚本
# 使用方法: ./scripts/test_api.sh [BASE_URL] [API_KEY]

set -e

# 默认配置
BASE_URL="${1:-http://localhost:8080}"
API_KEY="${2:-test-api-key}"

echo "🚀 开始测试 LiteLLM Gateway API"
echo "Base URL: $BASE_URL"
echo "API Key: $API_KEY"
echo "================================"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试函数
test_endpoint() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local data="$4"
    local expected_status="${5:-200}"
    
    echo -n "Testing $name... "
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "%{http_code}" -o /tmp/response.json \
            -H "Authorization: Bearer $API_KEY" \
            "$BASE_URL$endpoint")
    else
        response=$(curl -s -w "%{http_code}" -o /tmp/response.json \
            -X "$method" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $API_KEY" \
            -d "$data" \
            "$BASE_URL$endpoint")
    fi
    
    status_code="${response: -3}"
    
    if [ "$status_code" -eq "$expected_status" ]; then
        echo -e "${GREEN}✓ PASS${NC} (Status: $status_code)"
        return 0
    else
        echo -e "${RED}✗ FAIL${NC} (Status: $status_code, Expected: $expected_status)"
        echo "Response:"
        cat /tmp/response.json | jq . 2>/dev/null || cat /tmp/response.json
        echo ""
        return 1
    fi
}

# 1. 健康检查
echo "📊 Health Checks"
test_endpoint "Basic Health Check" "GET" "/health"
test_endpoint "Detailed Health Check" "GET" "/health/detailed"
echo ""

# 2. Models API
echo "🤖 Models API"
test_endpoint "List Models" "GET" "/v1/models"
echo ""

# 3. Chat Completions
echo "💬 Chat Completions API"

# 基础聊天请求
chat_basic='{
  "model": "gpt-3.5-turbo",
  "messages": [
    {
      "role": "user",
      "content": "Hello, how are you?"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 50
}'
test_endpoint "Basic Chat" "POST" "/v1/chat/completions" "$chat_basic"

# 多轮对话
chat_multi='{
  "model": "gpt-4",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "What is 2+2?"
    }
  ],
  "temperature": 0.1,
  "max_tokens": 10
}'
test_endpoint "Multi-turn Chat" "POST" "/v1/chat/completions" "$chat_multi"

# 流式响应
chat_stream='{
  "model": "gpt-3.5-turbo",
  "messages": [
    {
      "role": "user",
      "content": "Count from 1 to 5"
    }
  ],
  "stream": true,
  "max_tokens": 20
}'
test_endpoint "Streaming Chat" "POST" "/v1/chat/completions" "$chat_stream"

# 函数调用
chat_function='{
  "model": "gpt-4",
  "messages": [
    {
      "role": "user",
      "content": "What is the weather like in Tokyo?"
    }
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get current weather",
        "parameters": {
          "type": "object",
          "properties": {
            "city": {
              "type": "string",
              "description": "City name"
            }
          },
          "required": ["city"]
        }
      }
    }
  ]
}'
test_endpoint "Function Calling" "POST" "/v1/chat/completions" "$chat_function"
echo ""

# 4. Text Completions
echo "📝 Text Completions API"

completion_basic='{
  "model": "text-davinci-003",
  "prompt": "The future of AI is",
  "max_tokens": 50,
  "temperature": 0.7
}'
test_endpoint "Basic Completion" "POST" "/v1/completions" "$completion_basic"

completion_multiple='{
  "model": "text-davinci-003",
  "prompt": "Once upon a time",
  "max_tokens": 30,
  "n": 2,
  "temperature": 0.9
}'
test_endpoint "Multiple Completions" "POST" "/v1/completions" "$completion_multiple"
echo ""

# 5. Embeddings
echo "🔢 Embeddings API"

embedding_single='{
  "model": "text-embedding-ada-002",
  "input": "Hello world"
}'
test_endpoint "Single Embedding" "POST" "/v1/embeddings" "$embedding_single"

embedding_batch='{
  "model": "text-embedding-ada-002",
  "input": [
    "Hello world",
    "How are you?",
    "This is a test"
  ]
}'
test_endpoint "Batch Embeddings" "POST" "/v1/embeddings" "$embedding_batch"
echo ""

# 6. 错误测试
echo "❌ Error Handling Tests"

# 无效模型
invalid_model='{
  "model": "invalid-model",
  "messages": [
    {
      "role": "user",
      "content": "Hello"
    }
  ]
}'
test_endpoint "Invalid Model" "POST" "/v1/chat/completions" "$invalid_model" "400"

# 缺少必需字段
missing_field='{
  "messages": [
    {
      "role": "user",
      "content": "Hello"
    }
  ]
}'
test_endpoint "Missing Model Field" "POST" "/v1/chat/completions" "$missing_field" "400"

# 无效参数
invalid_params='{
  "model": "gpt-3.5-turbo",
  "messages": [
    {
      "role": "user",
      "content": "Hello"
    }
  ],
  "temperature": 5.0,
  "max_tokens": -100
}'
test_endpoint "Invalid Parameters" "POST" "/v1/chat/completions" "$invalid_params" "400"
echo ""

# 7. 认证测试 (如果启用了认证)
echo "🔐 Authentication Tests"

# 无API Key
echo -n "Testing No API Key... "
response=$(curl -s -w "%{http_code}" -o /tmp/response.json \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$chat_basic" \
    "$BASE_URL/v1/chat/completions")
status_code="${response: -3}"

if [ "$status_code" -eq "401" ] || [ "$status_code" -eq "403" ]; then
    echo -e "${GREEN}✓ PASS${NC} (Status: $status_code)"
else
    echo -e "${YELLOW}⚠ SKIP${NC} (Status: $status_code - Auth may be disabled)"
fi

# 无效API Key
echo -n "Testing Invalid API Key... "
response=$(curl -s -w "%{http_code}" -o /tmp/response.json \
    -X POST \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer invalid-key" \
    -d "$chat_basic" \
    "$BASE_URL/v1/chat/completions")
status_code="${response: -3}"

if [ "$status_code" -eq "401" ] || [ "$status_code" -eq "403" ]; then
    echo -e "${GREEN}✓ PASS${NC} (Status: $status_code)"
else
    echo -e "${YELLOW}⚠ SKIP${NC} (Status: $status_code - Auth may be disabled)"
fi
echo ""

# 清理
rm -f /tmp/response.json

echo "================================"
echo "🎉 API 测试完成!"
echo ""
echo "💡 提示:"
echo "- 如果某些测试失败，请检查服务器是否正在运行"
echo "- 确保配置了正确的提供商和API密钥"
echo "- 某些功能可能需要特定的配置才能工作"
echo ""
echo "📚 更多测试用例请参考: tests/api_test_examples.md"
