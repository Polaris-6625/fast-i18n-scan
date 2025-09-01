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

# Scan files
fast-i18n-scan src/**/*.{js,jsx,ts,tsx}

# Scan with verbose output
fast-i18n-scan -v src/**/*.{js,jsx,ts,tsx}

# Output to file
fast-i18n-scan -o results.json src/**/*.{js,jsx,ts,tsx}
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

```bash
# Install dependencies
npm install

# Build native module
npm run build

# Build for development
npm run build:debug

# Run tests
npm test

# Prepare for publishing
npm run prepublishOnly
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