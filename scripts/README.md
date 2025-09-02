# 脚本目录

本目录包含项目的所有脚本文件。

## 脚本说明

### 开发脚本
- `setup.sh` - 项目初始化脚本，安装依赖和设置环境
- `dev.sh` - 开发环境启动脚本
- `build.sh` - 项目构建脚本（包含NAPI和CLI构建）
- `test.sh` - 统一测试脚本，运行所有测试
- `clean.sh` - 清理构建产物和依赖

### 发布脚本
- `publish.sh` - 发布脚本
- `prepare-binary.js` - 准备二进制文件的Node.js脚本

### 测试脚本
- `test_directory_output.sh` - 目录输出测试脚本

## 使用方法

```bash
# 项目初始化
./scripts/setup.sh

# 开发环境
./scripts/dev.sh

# 构建项目
./scripts/build.sh

# 运行测试
./scripts/test.sh

# 清理项目
./scripts/clean.sh
```

## 通过npm运行

也可以通过npm脚本运行：

```bash
npm run setup
npm run dev
npm run build
npm test
npm run clean