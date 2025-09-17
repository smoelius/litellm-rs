#!/bin/bash

# 查找并清理备份文件的脚本
set -e

echo "🔍 查找项目中的备份文件..."

# 查找所有可能的备份文件模式
echo "📁 查找 .!*!* 模式的备份文件..."
find . -name ".!*!*" -type f 2>/dev/null | while read file; do
    echo "  发现: $file"
done

echo "📁 查找 *~ 模式的备份文件..."
find . -name "*~" -type f 2>/dev/null | while read file; do
    echo "  发现: $file"
done

echo "📁 查找 *.bak 模式的备份文件..."
find . -name "*.bak" -type f 2>/dev/null | while read file; do
    echo "  发现: $file"
done

echo "📁 查找 *.backup 模式的备份文件..."
find . -name "*.backup" -type f 2>/dev/null | while read file; do
    echo "  发现: $file"
done

echo "📁 查找 *.orig 模式的备份文件..."
find . -name "*.orig" -type f 2>/dev/null | while read file; do
    echo "  发现: $file"
done

echo "📁 查找 .#* 模式的备份文件..."
find . -name ".#*" -type f 2>/dev/null | while read file; do
    echo "  发现: $file"
done

echo ""
echo "✅ 备份文件查找完成！"
echo ""
echo "如果要删除这些备份文件，请运行："
echo "  ./scripts/clean_backup_files.sh"
