#!/bin/bash
# Demo运行脚本

set -e

echo "运行demo示例..."

# 检查demo目录是否存在
if [ ! -d "demo/i18n" ]; then
    echo "Demo目录不存在"
    exit 1
fi

cd demo/i18n

# 安装依赖（如果需要）
if [ ! -d "node_modules" ]; then
    echo "安装demo依赖..."
    npm install
fi

# 运行demo
echo "运行i18n扫描demo..."
node bin/scan.js

cd ../..
echo "Demo运行完成！"