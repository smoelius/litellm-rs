#!/bin/bash

# 启动配置驱动的LiteLLM Gateway

echo "🚀 启动配置驱动的LiteLLM Gateway"
echo "================================"

# 检查配置文件
CONFIG_FILE="gateway_config.yaml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ 配置文件不存在: $CONFIG_FILE"
    echo "请确保配置文件存在并包含正确的Google API密钥"
    exit 1
fi

echo "📄 使用配置文件: $CONFIG_FILE"
echo "🔍 验证配置文件..."

# 检查配置文件中的API密钥
if ! grep -q "api_key:" "$CONFIG_FILE"; then
    echo "❌ 配置文件中缺少api_key配置"
    exit 1
fi

# 停止之前的进程（如果有）
echo "🛑 停止之前的进程..."
pkill -f "google-gateway" || true
pkill -f "litellm-gateway" || true
sleep 2

echo "🚀 启动配置驱动的Gateway..."
echo "================================"

# 启动Gateway，传递配置文件路径
cargo run --bin google-gateway -- "$CONFIG_FILE"
