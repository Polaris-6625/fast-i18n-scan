// @ts-nocheck
const Linter = require('eslint').Linter;
const SourceCode = require('eslint').SourceCode;
const linter = new Linter();
const result = [];
const hardCodeSuggestions = [];
const noStringConcatenations = [];
linter.defineParser('ts-parser', require('@typescript-eslint/parser'));

linter.defineParser('js-eslint', require('babel-eslint'));

linter.defineRules({
  '@tencent/tea-i18n/no-bare-zh-in-js': require('@tencent/eslint-plugin-tea-i18n/lib/rules/no-bare-zh-in-js'),
  '@tencent/tea-i18n/no-bare-zh-in-jsx': require('@tencent/eslint-plugin-tea-i18n/lib/rules/no-bare-zh-in-jsx'),
  '@tencent/tea-i18n/no-hard-code-of-domain': require('@tencent/eslint-plugin-tea-i18n/lib/rules/no-hard-code-of-domain'),
  '@tencent/tea-i18n/no-string-concat': require('@tencent/eslint-plugin-tea-i18n/lib/rules/no-string-concat'),
});

/**
 * @param {string} filepath
 * @returns {import("eslint").Linter.Config}
 */
const getConfig = (filepath) => {
  const isTs = filepath.endsWith('ts') || filepath.endsWith('tsx');
  return {
    env: {
      browser: true,
      es6: true,
    },
    parser: isTs ? 'ts-parser' : 'js-eslint',
    parserOptions: {
      ecmaVersion: 6,
      sourceType: 'module',
      ecmaFeatures: {
        jsx: true,
        legacyDecorators: true,
      },
    },
    rules: {
      '@tencent/tea-i18n/no-bare-zh-in-js': 'error',
      '@tencent/tea-i18n/no-bare-zh-in-jsx': 'error',
      '@tencent/tea-i18n/no-hard-code-of-domain': 'error',
      '@tencent/tea-i18n/no-string-concat': 'error',
    },
  };
};

const getValue = (lines, { start, end }, messageId) => {
  const delta = end.line - start.line;
  const value = lines
    .filter((_, index) => index + 1 >= start.line && index + 1 <= end.line)
    .map((line, index) => {
      if (index === 0) {
        if (index === delta) {
          return line.substring(start.column - 1, end.column - 1).trim();
        }
        return line.substring(start.column - 1).trim();
      }
      if (index === delta) {
        return line.substring(0, end.column - 1).trim();
      }
      return line.trim();
    })
    .join('');

  if (messageId === 'bareZhInJsx') {
    return value.replace(/^<("[^"]*"|'[^']*'|[^'">])*>(.+)(<\/[\w\-\.]*>)$/, '$2');
  }
  return value.replace(/^['"`](.+)['"`]$/, '$1');
};

/**
 * @param {{
 *  content: string;
 *  filepath: string
 * }} param0
 */
const verify = ({ content, filepath }) => {
  const lines = SourceCode.splitLines(content);
  linter
    .verify(content, getConfig(filepath))
    .filter(
      (i) =>
        i.messageId === 'bareZhInJs' ||
        i.messageId === 'bareZhInJsx' ||
        i.messageId === 'bareZhInTemplate' ||
        i.messageId === 'forbiddenHardCodeOfDomain' ||
        i.messageId === 'noStringConcatenation',
    )
    .forEach(({ line, column, endLine, endColumn, messageId }, index) => {
      const loc = {
        start: { line, column },
        end: { line: endLine, column: endColumn },
      };

      // 当前 ESLint 规则可能出现重复
      if (index > 0 && result[result.length - 1]) {
        const { start, end } = result[result.length - 1].loc;
        if (start.line === line && start.column === column && end.line === endLine && end.column === endColumn) {
          return;
        }
      }

      if (messageId === 'forbiddenHardCodeOfDomain') {
        hardCodeSuggestions.push({
          filepath,
          loc,
          value: getValue(lines, loc, messageId),
        });
      } else if (messageId === 'noStringConcatenation') {
        noStringConcatenations.push({
          filepath,
          loc,
          value: getValue(lines, loc, messageId),
        });
      } else {
        result.push({
          filepath,
          loc,
          value: getValue(lines, loc, messageId),
        });
      }
    });
};

module.exports = { verify, result, hardCodeSuggestions, noStringConcatenations };
