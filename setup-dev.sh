#!/bin/bash

# LiteLLM-RS 开发环境设置脚本
# 这个脚本会帮助你快速设置开发环境

set -e

echo "🚀 LiteLLM-RS 开发环境设置"
echo "=================================="

# 检查 Docker 是否安装
if ! command -v docker &> /dev/null; then
    echo "❌ Docker 未安装。请先安装 Docker Desktop"
    echo "   下载地址: https://www.docker.com/products/docker-desktop"
    exit 1
fi

# 检查 Docker 是否运行
if ! docker info &> /dev/null; then
    echo "❌ Docker 未运行。请启动 Docker Desktop"
    exit 1
fi

echo "✅ Docker 已安装并运行"

# 启动开发服务
echo ""
echo "📦 启动开发服务 (PostgreSQL + Redis)..."
docker-compose -f deployment/docker/docker-compose.dev.yml up -d postgres-dev redis-dev

# 等待服务启动
echo "⏳ 等待服务启动..."
sleep 10

# 检查服务状态
echo ""
echo "🔍 检查服务状态..."

# 检查 PostgreSQL
if docker-compose -f deployment/docker/docker-compose.dev.yml exec -T postgres-dev pg_isready -U gateway_dev -d gateway_dev &> /dev/null; then
    echo "✅ PostgreSQL 已就绪"
else
    echo "❌ PostgreSQL 未就绪，请检查日志"
    docker-compose -f deployment/docker/docker-compose.dev.yml logs postgres-dev
    exit 1
fi

# 检查 Redis
if docker-compose -f deployment/docker/docker-compose.dev.yml exec -T redis-dev redis-cli ping | grep -q PONG; then
    echo "✅ Redis 已就绪"
else
    echo "❌ Redis 未就绪，请检查日志"
    docker-compose -f deployment/docker/docker-compose.dev.yml logs redis-dev
    exit 1
fi

echo ""
echo "🎉 开发环境设置完成！"
echo ""
echo "📋 服务信息:"
echo "   PostgreSQL: localhost:5433"
echo "   用户名: gateway_dev"
echo "   密码: dev_password"
echo "   数据库: gateway_dev"
echo ""
echo "   Redis: localhost:6380"
echo ""
echo "🚀 现在可以启动网关了:"
echo "   cargo run --bin gateway"
echo ""
echo "🛑 停止开发服务:"
echo "   docker-compose -f deployment/docker/docker-compose.dev.yml down"
