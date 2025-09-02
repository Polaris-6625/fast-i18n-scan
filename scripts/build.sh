#!/bin/bash
# 构建脚本

set -e

echo "开始构建项目..."

# 构建NAPI模块
echo "构建NAPI模块..."
napi build --platform --release --features napi

# 构建CLI二进制文件
echo "构建CLI二进制文件..."
cargo build --release --features cli

# 准备二进制文件
echo "准备二进制文件..."
node scripts/prepare-binary.js

echo "构建完成！"