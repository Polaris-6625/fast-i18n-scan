#!/bin/bash
# 项目结构验证脚本

set -e

echo "验证项目结构..."

# 检查必要的目录
directories=(
    "src"
    "src/bin"
    "src/scan"
    "src/utils"
    "scripts"
    "__test__"
    "__test__/unit"
    "__test__/integration"
    "__test__/fixtures"
    "demo"
    "examples"
)

for dir in "${directories[@]}"; do
    if [ -d "$dir" ]; then
        echo "✓ 目录存在: $dir"
    else
        echo "✗ 目录缺失: $dir"
    fi
done

# 检查必要的文件
files=(
    "Cargo.toml"
    "package.json"
    "README.md"
    "src/lib.rs"
    "src/bin/main.rs"
    "scripts/build.sh"
    "scripts/test.sh"
    "scripts/setup.sh"
    "__test__/README.md"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "✓ 文件存在: $file"
    else
        echo "✗ 文件缺失: $file"
    fi
done

# 检查脚本权限
scripts=(
    "scripts/build.sh"
    "scripts/test.sh"
    "scripts/dev.sh"
    "scripts/clean.sh"
    "scripts/setup.sh"
    "scripts/demo.sh"
)

for script in "${scripts[@]}"; do
    if [ -x "$script" ]; then
        echo "✓ 脚本可执行: $script"
    else
        echo "✗ 脚本不可执行: $script"
    fi
done

echo "项目结构验证完成！"