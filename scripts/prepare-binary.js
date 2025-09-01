#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const os = require('os');

const platform = os.platform();
const arch = os.arch();

let binaryName = 'fast-i18n-scan';
if (platform === 'win32') {
  binaryName += '.exe';
}

const sourcePath = path.join(__dirname, '..', 'target', 'release', binaryName);
const targetPath = path.join(__dirname, '..', 'bin', binaryName);

if (fs.existsSync(sourcePath)) {
  // 确保目标目录存在
  const targetDir = path.dirname(targetPath);
  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
  }
  
  // 复制二进制文件
  fs.copyFileSync(sourcePath, targetPath);
  
  // 设置执行权限（Unix系统）
  if (platform !== 'win32') {
    fs.chmodSync(targetPath, '755');
  }
  
  console.log(`Binary copied to ${targetPath}`);
} else {
  console.warn(`Binary not found at ${sourcePath}. Make sure to build with: cargo build --release --features cli`);
}