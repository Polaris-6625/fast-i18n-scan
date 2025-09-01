// @ts-check
const path = require('path');
const colors = require('colors');
const lngs = require('../lngs');
// const defaultLng = require("../../../lib/i18n").getDefaultLng();
const defaultLng = 'zh';
/**
 * 得到不包含单复数形式的 Key
 * @param {string} key
 */
const getBaseKey = (key) => {
  let baseKey = key;
  if (/^(k_.+)_(\d|plural)$/.test(key)) {
    baseKey = RegExp.$1;
  }
  return baseKey;
};

const isSameSentence = (a, b) => {
  return a.replace(/\s+/g, '') === b.replace(/\s+/g, '');
};

/**
 * 管理一个 Sisulizer 项目的内部数据结构
 */
function createSisulizerProject(options = { nativeLang: defaultLng, langs: Object.keys(lngs) }) {
  // 本地语言，应该为 'zh'
  let nativeLang = options.nativeLang;

  // 要国际化的语言列表
  let langs = options.langs.filter((x) => x != nativeLang);

  /**
   * 翻译行记录
   * @type {Map<string, import('./slp').SourceRow>}
   */
  const rowMap = new Map();

  /**
   * 标记为废弃的行
   * @type {Set<string>}
   */
  const obsoletedSet = new Set();

  /**
   * 创建新的翻译行
   * @param {string} key
   * @returns {import('./slp').SourceRow}
   */
  function createRow(key) {
    const row = {
      key,
      nativeString: null,
      translateMap: new Map(),
    };
    rowMap.set(key, row);
    return row;
  }

  /**
   * 获取指定 Key 的翻译行，如不存在则创建
   * @param {string} key
   */
  function getOrCreateRow(key) {
    return rowMap.get(key) || createRow(key);
  }

  /**
   * 添加翻译数据
   * @param {string} key
   * @param {string} lang
   * @param {string} translatedString
   */
  function add(key, lang, translatedString) {
    const row = getOrCreateRow(key);
    const nativeString = get(key, nativeLang);
    // 如果就是本地语言，直接写到 nativeString 上
    if (lang === nativeLang) {
      row.nativeString = translatedString;
    }
    // 如果翻译句子和本地句子一样，则不写入
    else if (translatedString && nativeString && !isSameSentence(translatedString, nativeString)) {
      row.translateMap.set(lang, translatedString);
    }
  }

  /**
   * 获取翻译数据
   * @param {string} key
   * @param {string} lang
   */
  function get(key, lang) {
    const row = rowMap.get(key);
    if (!row) return null;
    if (lang === nativeLang) {
      return row.nativeString || tryFindNativeString(key);
    }
    return row.translateMap.get(lang);
  }

  /**
   * 获取所有的翻译键
   */
  function keys() {
    return Array.from(rowMap.keys());
  }

  /**
   * 标记为废弃词条
   * @param {string} key
   */
  function obsolete(key) {
    obsoletedSet.add(key);
  }

  /**
   * 从指定的目录中加载数据
   * @param {string} sourcePath
   */
  async function load(sourcePath, lang) {
    try {
      const data = require(path.resolve(sourcePath, `${lang}.json`));
      Object.entries(data).forEach(([key, translatedString]) => {
        const row = getOrCreateRow(key);
        const nativeString = get(key, nativeLang);
        if (lang === nativeLang) {
          row.nativeString = translatedString;
        }
        // 认为 json 中一定是翻译后的词条，不进行 isSameSentence 比较
        else if (translatedString && nativeString) {
          if (lang === lngs.en && /[\u4e00-\u9fa5]/.test(translatedString)) {
            // 抛错误 否则在tea build阶段调用无法正常抛出异常
            throw new Error(
              `${colors.red(`\n\n${lang}.json 词条中发现中文：\n`)}  ${translatedString}\n${colors.red(
                '请检查词条文件\n',
              )}`,
            );
          }
          row.translateMap.set(lang, translatedString);
        }
      });
    } catch (error) {
      console.error(error);
      process.exit(1);
    }
  }

  /**
   * 导出要保存的 vinyl 内容
   */
  function output(lang) {
    const json = {};
    for (let key of keys().sort()) {
      if (!obsoletedSet.has(getBaseKey(key))) {
        json[key] = get(key, lang || nativeLang);
      }
    }
    return JSON.stringify(json, null, 2);
  }

  /**
   * 翻译的词条 Key 可能是带上下文或者带复数形式的
   * @param {string} key
   */
  function tryFindNativeString(key) {
    const baseKey = getBaseKey(key);
    const possibleKeys = [baseKey];
    // 中文复数形式
    possibleKeys.push([baseKey, '0'].join('_'));
    for (let possibleKey of possibleKeys) {
      const possibleRow = rowMap.get(possibleKey);
      const nativeString = possibleRow && possibleRow.nativeString;
      if (nativeString) {
        return nativeString;
      }
    }
  }

  return { load, output, add, get, keys, obsolete };
}

module.exports = { createSisulizerProject, getBaseKey };
