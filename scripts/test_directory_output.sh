#!/bin/bash

# 测试目录输出功能的脚本

echo "=== 测试目录输出功能 ==="

# 创建测试文件
mkdir -p test_files
cat > test_files/test.js << 'EOF'
const message = "你好世界";
const greeting = "欢迎使用";
const button = "点击按钮";
EOF

echo "创建了测试文件 test_files/test.js"

# 构建项目
echo "构建项目..."
cargo build --release --features cli

# 运行扫描并输出到目录
echo "运行扫描..."
./target/release/fast-i18n-scan test_files/test.js -f directory -o ./scan_output -v

# 检查输出结果
echo ""
echo "=== 检查输出结果 ==="
if [ -d "./scan_output" ]; then
    echo "✓ 输出目录已创建"
    
    if [ -f "./scan_output/context/context.json" ]; then
        echo "✓ context.json 已创建"
        echo "context.json 内容:"
        cat ./scan_output/context/context.json | jq .
    else
        echo "✗ context.json 未找到"
    fi
    
    if [ -f "./scan_output/source/zh.json" ]; then
        echo "✓ zh.json 已创建"
        echo "zh.json 内容:"
        cat ./scan_output/source/zh.json | jq .
    else
        echo "✗ zh.json 未找到"
    fi
else
    echo "✗ 输出目录未创建"
fi

# 清理
echo ""
echo "清理测试文件..."
rm -rf test_files scan_output

echo "测试完成!"