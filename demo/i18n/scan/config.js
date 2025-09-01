const lngs = require('./lngs');
// const defaultLng = require('../../lib/i18n').getDefaultLng();
const defaultLng = 'zh';
/**
 * @type {import('@babel/parser').ParserOptions}
 */
const babelOptions = {
  plugins: [
    'jsx',
    'typescript',
    'doExpressions',
    'objectRestSpread',
    ['decorators', { decoratorsBeforeExport: true }],
    'classProperties',
    'classPrivateProperties',
    'classPrivateMethods',
    'exportDefaultFrom',
    'exportNamespaceFrom',
    'asyncGenerators',
    'functionBind',
    'functionSent',
    'dynamicImport',
    'numericSeparator',
    'optionalChaining',
    'importMeta',
    'bigInt',
    'optionalCatchBinding',
    'throwExpressions',
    ['pipelineOperator', { proposal: 'minimal' }],
    'nullishCoalescingOperator',
  ],
  sourceType: 'module',
};

/**
 * 默认 i18next 扫描配置
 */
const defaultConfig = {
  input: ['src/**/*.{js,jsx,ts,tsx}'],
  lngs: [defaultLng, ...Object.keys(lngs).filter((i) => i !== defaultLng)],
  ns: ['translation'],
  defaultLng: defaultLng,
  defaultNs: 'translation',
  resource: {
    loadPath: '', // 避免 i18next-scanner 读取报错
    savePath: 'i18n/translation/{{lng}}.js',
  },
  func: {
    list: ['i18next.t', 'i18n.t', 't'],
    extensions: [], // 避免在 transform 中执行原生的 parseFuncFromString
    babylon: babelOptions,
  },
  trans: {
    extensions: [], // 避免在 transform 中执行原生的 parseTransFromString
    babylon: babelOptions,
  },
};

module.exports = defaultConfig;
