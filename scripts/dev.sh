#!/bin/bash
# 开发环境启动脚本

set -e

echo "启动开发环境..."

# 构建项目
cargo build

# 运行示例
echo "可以使用以下命令运行示例："
echo "cargo run -- --help"
echo "或者运行demo: cd demo/i18n && npm run dev"