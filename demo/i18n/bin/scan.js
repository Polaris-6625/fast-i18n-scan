#!/usr/bin/env node
const path = require('path');
const program = require('commander');
const { runInNewContext } = require('vm');
const fs = require('fs');
const scanner = require('../scan');

// 读取配置文件
const readConfig = (filePath) => {
  const configFs = fs.readFileSync(filePath.toString(), 'utf8');
  // 创建一个沙箱环境的对象
  const sandbox = {
    module: { exports: {} },
    exports: {},
  };
  runInNewContext(configFs, sandbox);
  return sandbox.module.exports;
};

function list(val) {
  return val.split(';');
}

function mixinCommandOptions(program) {
  const config = null;
  const options = Object.assign({}, program.opts());
  if (config && typeof config.command === 'object') {
    const cmdName = program.name().replace('tea-', '');
    const configOptions = config.command[cmdName] || {};
    Object.entries(configOptions).forEach(([key, value]) => {
      if (typeof options[key] === 'undefined') {
        options[key] = value;
      }
    });
  }
  return options;
}

program.option(
  '-i, --input [input]',
  '需要扫描的目录，多个目录用分号分割，采用 glob pattern 匹配，默认为 `src/**/*.{js,jsx,ts,tsx}`',
  list,
);
program.option('-o, --output [output]', '需要保存输出文件的目录，默认为 `i18n`');
program.option('-a, --append', '追加模式，不改变已有词条');
program.option('-c, --config [config]', '自定义配置文件，配置可参考 https://github.com/i18next/i18next-scanner');
program.parse(process.argv);

async function run() {
  const cwd = path.resolve('.');
  const options = mixinCommandOptions(program);

  const config = options.config ? readConfig(path.join(cwd, options.config)) : {};

  if (options.input) {
    config.input = options.input;
  }
  if (options.output) {
    config.output = options.output;
  }
  config.appendMode = options.append;

  scanner.run(cwd, config);
}

run();
