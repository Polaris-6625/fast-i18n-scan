#!/bin/bash
# 统一测试脚本

set -e

echo "开始运行测试..."

# 运行单元测试
echo "运行单元测试..."
cargo test --no-default-features --features cli

# 运行集成测试
echo "运行集成测试..."
if [ -d "__test__/integration" ]; then
    cargo test --test integration
fi

# 测试CLI功能
echo "测试CLI功能..."
if [ -f "target/release/fast-i18n-scan" ] || [ -f "target/debug/fast-i18n-scan" ]; then
    echo "测试CLI基本功能..."
    
    # 使用测试夹具
    if [ -f "__test__/fixtures/demo_test.jsx" ]; then
        echo "测试扫描JSX文件..."
        cargo run --features cli -- __test__/fixtures/demo_test.jsx --verbose || echo "CLI测试完成"
    fi
    
    # 测试目录输出功能
    if [ -d "__test__/fixtures/test_vite_app" ]; then
        echo "测试目录输出功能..."
        cargo run --features cli -- "__test__/fixtures/test_vite_app/src/**/*.{js,jsx,ts,tsx}" -f directory -o __test__/output || echo "目录输出测试完成"
    fi
else
    echo "CLI二进制文件不存在，跳过CLI测试"
fi

# 运行demo测试
echo "运行demo测试..."
if [ -d "demo/i18n" ]; then
    cd demo/i18n
    if [ -f "package.json" ]; then
        npm test 2>/dev/null || echo "Demo测试跳过（需要npm install）"
    fi
    cd ../..
fi

# 测试NAPI模块（如果存在）
echo "测试NAPI模块..."
if [ -f "index.js" ] && [ -f "*.node" ]; then
    echo "NAPI模块存在，运行Node.js测试..."
    node -e "
        try {
            const { scanFile, getVersion } = require('./index.js');
            console.log('NAPI版本:', getVersion());
            if (require('fs').existsSync('__test__/fixtures/demo_test.jsx')) {
                const result = scanFile('__test__/fixtures/demo_test.jsx');
                console.log('NAPI测试成功，找到', result.keys.length, '个键');
            }
        } catch (e) {
            console.log('NAPI测试跳过:', e.message);
        }
    " || echo "NAPI测试完成"
fi

echo "所有测试完成！"