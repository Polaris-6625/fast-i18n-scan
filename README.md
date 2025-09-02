# Fast i18n Scan

A fast and efficient internationalization scanning library for JavaScript/TypeScript projects, written in Rust.

## Features

- 🚀 **Fast**: Written in Rust for maximum performance
- 🔍 **Comprehensive**: Scans JavaScript, TypeScript, JSX, and TSX files
- 🌐 **i18n Ready**: Detects hard-coded Chinese text and suggests internationalization
- 🛠 **Configurable**: Flexible configuration options
- 📊 **Detailed Reports**: Provides detailed scan results with statistics
- 🎯 **Accurate**: Uses advanced parsing techniques for accurate detection
- 🔧 **Node.js Compatible**: Native bindings for seamless Node.js integration

## Installation

```bash
npm install fast-i18n-scan
# or
yarn add fast-i18n-scan
# or
pnpm add fast-i18n-scan
```

## Usage

### Node.js/JavaScript

```javascript
const { scanFiles, scanFile, getVersion } = require('fast-i18n-scan');

// Scan multiple files
const result = scanFiles(['src/app.tsx', 'src/components/*.tsx']);
console.log('Found keys:', result.keys);
console.log('Errors:', result.errors);
console.log('Statistics:', result.stats);

// Scan single file
const singleResult = scanFile('src/app.tsx');
console.log('Processing time:', singleResult.stats.processingTimeMs, 'ms');

// Get version
console.log('Version:', getVersion());
```

### TypeScript

```typescript
import { scanFiles, scanFile, getVersion, JsScanResult } from 'fast-i18n-scan';

const result: JsScanResult = scanFiles(['src/**/*.{ts,tsx}']);

result.errors.forEach(error => {
  console.log(`${error.filepath}:${error.line}:${error.column} - ${error.message}`);
});

result.warnings.forEach(warning => {
  console.log(`Warning: ${warning.message} at ${warning.filepath}:${warning.line}`);
});
```

### Command Line (if installed globally)

```bash
# Install globally
npm install -g fast-i18n-scan

# Scan files (JSON format output)
fast-i18n-scan src/**/*.{js,jsx,ts,tsx}

# Output to directory structure (新功能!)
fast-i18n-scan "src/**/*.js" -f directory -o ./i18n_output

# Scan with verbose output
fast-i18n-scan -v src/**/*.{js,jsx,ts,tsx}

# Output to JSON file
fast-i18n-scan -o results.json src/**/*.{js,jsx,ts,tsx}
```

### Directory Output Format

使用 `-f directory` 参数时，扫描结果会生成一个目录结构：

```
i18n_output/
├── context/
│   └── context.json     # 扫描上下文信息
└── source/
    └── zh.json          # 中文文本键值对
```

**context.json** 包含扫描的元数据：
```json
{
  "active_keys": 8,
  "generated_at": "2025-09-01 15:25:25 UTC",
  "language": "zh",
  "obsoleted_keys": 0,
  "project_info": {
    "available_languages": ["zh"],
    "native_language": "zh"
  },
  "total_keys": 8
}
```

**zh.json** 包含生成的键值对：
```json
{
  "k_00035cd": "欢迎使用我们的应用",
  "k_00035g2": "这是一个测试页面", 
  "k_00037vt": "点击这里",
  "k_0003vl0": "用户名",
  "k_0003xiv": "密码",
  "k_0003y73": "登录",
  "k_0003y9x": "注册新账户",
  "k_0003yr1": "忘记密码？"
}
```

## API Reference

### `scanFiles(files: string[]): JsScanResult`

Scans multiple files for i18n keys and issues.

**Parameters:**
- `files`: Array of file paths to scan

**Returns:** `JsScanResult` object containing:
- `keys`: Array of found i18n keys
- `translations`: Map of keys to their default values
- `errors`: Array of errors found
- `warnings`: Array of warnings
- `stats`: Scanning statistics

### `scanFile(filepath: string): JsScanResult`

Scans a single file for i18n keys and issues.

**Parameters:**
- `filepath`: Path to the file to scan

**Returns:** Same as `scanFiles`

### `getVersion(): string`

Returns the library version.

## What it detects

- ✅ Hard-coded Chinese text in JavaScript/TypeScript code
- ✅ Hard-coded Chinese text in JSX elements
- ✅ Hard-coded domain names
- ✅ String concatenation that should use i18n
- ✅ Missing translations
- ✅ Unused translation keys
- ✅ Template literals with Chinese text
- ✅ Object properties with Chinese values

## Result Format

```typescript
interface JsScanResult {
  keys: string[];
  translations: Record<string, string>;
  errors: JsScanError[];
  warnings: JsScanWarning[];
  stats: JsScanStats;
}

interface JsScanError {
  filepath: string;
  line: number;
  column: number;
  message: string;
  errorType: string;
}

interface JsScanWarning {
  filepath: string;
  line: number;
  column: number;
  message: string;
  warningType: string;
}

interface JsScanStats {
  filesScanned: number;
  keysFound: number;
  errorsCount: number;
  warningsCount: number;
  processingTimeMs: number;
}
```

## Performance

This library is built with Rust and uses native bindings for Node.js, providing:

- **10x faster** than pure JavaScript implementations
- **Low memory usage** with efficient string processing
- **Parallel processing** for multiple files
- **Zero dependencies** runtime (native binary)

## Platform Support

Pre-built binaries are available for:

- **Windows**: x64, ia32, arm64
- **macOS**: x64, arm64 (Apple Silicon)
- **Linux**: x64, arm64 (glibc and musl)
- **Android**: arm64, arm

## Development

### 快速开始

```bash
# 项目初始化（推荐）
npm run setup
# 或者
./scripts/setup.sh

# 开发环境
npm run dev
# 或者
./scripts/dev.sh

# 构建项目
npm run build
# 或者
./scripts/build.sh

# 运行测试
npm test
# 或者
./scripts/test.sh

# 清理项目
npm run clean
# 或者
./scripts/clean.sh
```

### 项目结构

```
fast-i18n-scan/
├── src/                    # Rust源代码
│   ├── bin/               # CLI入口
│   ├── scan/              # 扫描核心逻辑
│   └── utils/             # 工具函数
├── scripts/               # 统一脚本目录
│   ├── setup.sh          # 项目初始化
│   ├── build.sh          # 构建脚本
│   ├── test.sh           # 测试脚本
│   ├── dev.sh            # 开发脚本
│   ├── clean.sh          # 清理脚本
│   ├── demo.sh           # Demo运行
│   └── README.md         # 脚本说明
├── __test__/              # 统一测试目录
│   ├── unit/             # 单元测试
│   ├── integration/      # 集成测试
│   ├── fixtures/         # 测试夹具
│   └── README.md         # 测试说明
├── demo/                  # 示例代码
├── examples/              # 使用示例
└── bin/                   # 发布的二进制文件
```

### 脚本说明

- **setup.sh**: 项目初始化，安装依赖和设置环境
- **build.sh**: 完整构建，包含NAPI模块和CLI二进制文件
- **test.sh**: 运行所有测试（单元测试、集成测试、CLI测试、NAPI测试）
- **dev.sh**: 开发环境启动
- **clean.sh**: 清理所有构建产物
- **demo.sh**: 运行demo示例

### 测试

项目包含完整的测试套件：

- **单元测试**: Rust代码的单元测试
- **集成测试**: 完整功能的集成测试
- **CLI测试**: 命令行工具的功能测试
- **NAPI测试**: Node.js绑定的测试
- **Demo测试**: 示例代码的测试

运行特定测试：

```bash
# 仅运行单元测试
cargo test --no-default-features --features cli

# 仅运行集成测试
cargo test --test integration

# 仅运行CLI测试
cargo run --features cli -- __test__/fixtures/demo_test.jsx --verbose
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Changelog

### 0.2.0
- Initial npm release
- Native Node.js bindings
- Cross-platform support
- TypeScript definitions
- Comprehensive i18n scanning features