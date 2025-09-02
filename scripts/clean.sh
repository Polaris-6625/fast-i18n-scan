#!/bin/bash
# 清理脚本

set -e

echo "清理项目..."

# 清理Rust构建产物
cargo clean

# 清理demo node_modules（如果存在）
if [ -d "demo/i18n/node_modules" ]; then
    echo "清理demo依赖..."
    rm -rf demo/i18n/node_modules
fi

echo "清理完成！"