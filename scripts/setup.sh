#!/bin/bash
# 项目初始化脚本

set -e

echo "初始化项目..."

# 安装Rust依赖
echo "构建Rust项目..."
cargo build

# 设置demo环境
if [ -d "demo/i18n" ]; then
    echo "设置demo环境..."
    cd demo/i18n
    if [ -f "package.json" ]; then
        npm install
    fi
    cd ../..
fi

echo "项目初始化完成！"
echo "使用 ./scripts/dev.sh 启动开发环境"
echo "使用 ./scripts/test.sh 运行测试"