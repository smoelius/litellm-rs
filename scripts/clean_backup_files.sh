#!/bin/bash

# 清理备份文件的脚本
set -e

echo "🧹 开始清理项目中的备份文件..."

# 统计要删除的文件数量
backup_count=0

# 清理 .!*!* 模式的备份文件
echo "📁 清理 .!*!* 模式的备份文件..."
find . -name ".!*!*" -type f 2>/dev/null | while read file; do
    echo "  删除: $file"
    rm -f "$file"
    ((backup_count++))
done

# 清理 *~ 模式的备份文件
echo "📁 清理 *~ 模式的备份文件..."
find . -name "*~" -type f 2>/dev/null | while read file; do
    echo "  删除: $file"
    rm -f "$file"
    ((backup_count++))
done

# 清理 *.bak 模式的备份文件
echo "📁 清理 *.bak 模式的备份文件..."
find . -name "*.bak" -type f 2>/dev/null | while read file; do
    echo "  删除: $file"
    rm -f "$file"
    ((backup_count++))
done

# 清理 *.backup 模式的备份文件
echo "📁 清理 *.backup 模式的备份文件..."
find . -name "*.backup" -type f 2>/dev/null | while read file; do
    echo "  删除: $file"
    rm -f "$file"
    ((backup_count++))
done

# 清理 .#* 模式的备份文件 (Emacs锁文件)
echo "📁 清理 .#* 模式的备份文件..."
find . -name ".#*" -type f 2>/dev/null | while read file; do
    echo "  删除: $file"
    rm -f "$file"
    ((backup_count++))
done

# 注意：不清理 *.orig 文件，因为这些可能是重要的原始文件
echo "ℹ️  保留 *.orig 文件 (这些可能是重要的原始文件)"

echo ""
echo "✅ 备份文件清理完成！"
echo ""
echo "🔍 验证清理结果..."
remaining=$(find . -name ".!*!*" -o -name "*~" -o -name "*.bak" -o -name "*.backup" -o -name ".#*" 2>/dev/null | wc -l)
echo "剩余备份文件数量: $remaining"

if [ "$remaining" -eq 0 ]; then
    echo "🎉 所有备份文件已成功清理！"
else
    echo "⚠️  还有 $remaining 个备份文件未清理"
fi
