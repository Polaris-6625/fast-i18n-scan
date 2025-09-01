const fs = require('fs');
const path = require('path');
const Vinyl = require('vinyl');
const rimraf = require('rimraf');
const through2 = require('through2');
const colors = require('colors');
const CliTable = require('cli-table3');
const lngs = require('./lngs');
const { createSisulizerProject, getBaseKey } = require('./slp/slp');
const { result: unmarked, hardCodeSuggestions, noStringConcatenations } = require('./zh-linter');
// const nativeLang = require('../../lib/i18n').getDefaultLng();
const nativeLang = 'zh';

const fallbackLangMap = {
  ja: 'en',
  ko: 'en',
};

// 生成的语言文件模板
const LNG_FILE_TPL = fs.readFileSync(path.join(__dirname, '../resource/lng.js.tpl'), 'utf8');

/**
 * 更新词条
 * @param {string} appPath
 * @param {object} options
 */
function writer(appPath, options, onFinish, exportUntranslated) {
  // 从 options 对象中获取 untranslated 对象
  const untranslated = options.untranslated;

  // 每个语言的扫描统计信息
  const statsMap = new Map();
  const statsDetail = {
    stats: {},
  };

  // JS 词条文件位置
  const lngFilePath = options.resource.savePath;

  // 国际化 Sisu 项目文件
  const projectPath = options.output || 'i18n';
  const projectSourcePath = path.resolve(projectPath, 'source');

  // Sisu 项目数据
  const project = createSisulizerProject({
    nativeLang,
    langs: Object.keys(lngs),
  });

  const otherLangs = Object.keys(lngs).filter((lang) => lang !== nativeLang);
  let otherLangsExistedSet = new Set();

  [nativeLang, ...otherLangs].forEach((lang) => {
    // Sisu 项目文件存在，加载进来
    if (fs.existsSync(path.resolve(appPath, projectSourcePath, `${lang}.json`))) {
      if (lang !== nativeLang) {
        otherLangsExistedSet.add(lang);
      }
      project.load(path.resolve(appPath, projectSourcePath), lang);
    }
    // 不存在，一次性从现有词条文件同步
    else {
      const { translation } = read(appPath, lngFilePath, lngs[lang]);
      for (let [key, translatedString] of Object.entries(translation)) {
        project.add(key, lang, translatedString);
      }
    }
  });
  /**
   * 以一定规则计算翻译团队需要“翻译”的字数,规则基于word字数统计的规则，详情如下:
   * - 每个中文字符、标点等等一系列全角类型字符，算 1个字；
     - 每个连续的半角（单字节）字符段落，算 1 个字 ；如「.」、「,」、「tencent」、「3.14159」、"a_very_long_word_is_also_count_as_one_character"
     - 跨目录的重复字数，人工会进行扣除;
   * @param {string} str 要计算的字符串
   */
  function countCharacters(str) {
    // 使用正则表达式匹配中文字、符和全角符号（这里统一称为全角了）
    const fullWidthSymbols = str.match(
      /[\u4e00-\u9fa5\u3000-\u303f\ufe30-\ufe4f\uf900-\ufaff\uff10-\uff19\uff01-\uff5e\uff00-\uffef]/g,
    );

    // 统计全角字符数
    const fullWidthSymbolCount = fullWidthSymbols ? fullWidthSymbols.length : 0;

    // 将全角符号替换为空格
    const stringWithoutFullWidthSymbols = str.replace(
      /[\u4e00-\u9fa5\u3000-\u303f\ufe30-\ufe4f\uf900-\ufaff\uff10-\uff19\uff01-\uff5e\uff00-\uffef]/g,
      ' ',
    );

    // 使用trim()方法去除字符串的前导和后导空格
    const trimmedString = stringWithoutFullWidthSymbols.trim();
    let segmentCount = 0;
    if (trimmedString.length !== 0) {
      // 使用split()方法将字符串分割为段落
      const segments = trimmedString.split(/\s+/);

      // 计算段落数， 即连续半角字符的段落数
      segmentCount = segments.length;
    }

    // 返回总字数
    return fullWidthSymbolCount + segmentCount;
  }
  /**
   *
   * i18n-scanner 会把扫描结果以每个语言一个文件的形式提供
   *
   * @this {import('stream').Transform}
   * @param {import('vinyl')} i18nextFile 翻译文件，其中 `i18nextFile.contents` 包含翻译的扫描结果
   * @param {string} enc
   * @param {Function} cb
   */
  function write(i18nextFile, enc, cb) {
    // 文件名就是对应语言
    const lang = path.parse(i18nextFile.basename).name;

    // 缺少翻译时的 fallback 查找
    const fallbackLang = fallbackLangMap[lang];

    // 我们使用的语言别名
    const lng = lngs[lang];

    // 统计信息
    const stats = {
      scanned: 0,
      translated: 0,
      untranslated: 0,
      fallback: 0,
      unused: 0,
      words: 0,
    };
    statsMap.set(lng, stats);
    statsDetail[lng] = {
      untranslated: [],
      untranslatedWords: [],
      unused: [],
    };

    // 词条扫描结果
    const scanResult = JSON.parse(i18nextFile.contents.toString('utf8'));

    // 最后要生成的翻译文件的行内容
    const translationLines = [];

    const usedBaseKeySet = new Set();

    // 遍历扫描结果
    for (let [key, resource] of Object.entries(scanResult)) {
      // 忽略空词条
      if (!resource) {
        continue;
      }

      stats.scanned++;

      usedBaseKeySet.add(getBaseKey(key));

      // 获取现有翻译结果
      let translatedString = project.get(key, lang);
      let fallbackTranslatedString = null;

      // 扫描到的词条已有翻译数据
      if (translatedString) {
        stats.translated++;
      }
      // 扫描到新词条，未有翻译数据，添加到项目数据中 同时统计需要翻译的“字数”（该字数以一定规则计算）
      else {
        project.add(key, lang, resource);
        stats.untranslated++;
        stats.words += countCharacters(resource);
        statsDetail[lng].untranslated.push(key);

        // 如果有 fallback，使用 fallback
        if (fallbackLang) {
          if ((fallbackTranslatedString = project.get(key, fallbackLang))) {
            translatedString = fallbackTranslatedString;
            stats.fallback++;
          }
        }
      }
      // 遍历词条后，把统计的未翻译的“字数”赋值给对应的statsDetail
      statsDetail[lng].untranslatedWords = stats.words;

      // 如果有翻译内容了，写入翻译行
      if (translatedString) {
        let content = JSON.stringify(translatedString) + ',';
        if (fallbackTranslatedString) {
          content += ` // fallback from ${fallbackLang}`;
        }
        if (lang !== nativeLang) {
          translationLines.push(`  "${key}": ${content}`);
        }
      }

      // 如果需要导出未翻译词条，将未翻译词条保存到 untranslated
      if (exportUntranslated && !translatedString) {
        untranslated[lang] = untranslated[lang] || {};
        untranslated[lang][key] = resource;
      }
    }

    // 遍历原词条，对于那些没在扫描结果里的词条，标记为废弃词条
    for (let key of project.keys()) {
      // 废弃空词条
      if (!options.appendMode && !project.get(key, nativeLang)) {
        project.obsolete(key);
        continue;
      }

      if (!scanResult[key]) {
        stats.unused++;
        statsDetail[lng].unused.push(key);
      }
      // usedBaseKeySet 中没有对应 baseKey，则可废弃词条
      if (!options.appendMode && !usedBaseKeySet.has(getBaseKey(key))) {
        project.obsolete(key);
      }
    }

    statsDetail[lng].untranslated.sort();
    statsDetail[lng].unused.sort();
    statsDetail.stats.marked = stats.scanned;
    if (lang !== nativeLang) {
      statsDetail.stats[`${lang}Untranslated`] = stats.untranslated;
      statsDetail.stats[`${lang}UntranslatedWords`] = stats.words;
    }

    if (stats.translated > 0) {
      const fileContentString = LNG_FILE_TPL.replace(/__\$\$\(translation\)__/g, translationLines.sort().join('\n'));

      const resourceFile = new Vinyl({
        cwd: i18nextFile.cwd,
        base: i18nextFile.base,
        contents: Buffer.from(fileContentString, 'utf8'),
        path: lngFilePath.replace(/{{lng}}/g, lng),
      });

      this.push(resourceFile);
    }

    cb();
  }

  /**
   * @this {import('stream').Transform}
   */
  function flush(cb) {
    if (nativeLang === 'zh') {
      // untranslated/unused/unmarked 统计
      statsDetail.stats.unmarked = unmarked.length;
    }
    this.push(
      new Vinyl({
        cwd: appPath,
        base: appPath,
        path: path.resolve(projectPath, `stats.json`),
        contents: Buffer.from(JSON.stringify(statsDetail, null, 2), 'utf8'),
      }),
    );
    // 如果需要导出未翻译词条，创建 lng.untranslated.json 文件
    if (exportUntranslated) {
      for (const lang of Object.keys(lngs)) {
        const untranslatedContent = JSON.stringify(untranslated[lang] || {}, null, 2);

        this.push(
          new Vinyl({
            cwd: appPath,
            base: appPath,
            path: path.join(appPath, 'i18n', 'untranslated', `${lang}.untranslated.json`),
            contents: Buffer.from(untranslatedContent, 'utf8'),
          }),
        );
      }
    }

    // nativeLang 每次输出
    Object.keys(lngs)
      .filter((lang) => !otherLangsExistedSet.has(lang))
      .forEach((lang) => {
        const content = project.output(lang);
        if (content !== '{}') {
          this.push(
            new Vinyl({
              cwd: appPath,
              base: appPath,
              path: path.resolve(projectSourcePath, `${lang}.json`),
              contents: Buffer.from(content, 'utf8'),
            }),
          );
        }
      });
    const table = new CliTable({
      head: [
        'Language',
        'Keys',
        colors.green('Translated'),
        colors.red('Not Translated'),
        colors.yellow('Fallback'),
        colors.gray('No Usage'),
        colors.blue('Words'),
      ],
      colAligns: ['right', 'right', 'right', 'right', 'right', 'right', 'right'],
      style: { head: [] },
      chars: {
        top: '',
        'top-mid': '',
        'top-left': '',
        'top-right': '',
        bottom: '',
        'bottom-mid': '',
        'bottom-left': '',
        'bottom-right': '',
        left: '',
        'left-mid': '',
        mid: '',
        'mid-mid': '',
        right: '',
        'right-mid': '',
        middle: ' ',
      },
    });
    // @ts-ignore
    for (let [lng, { scanned, translated, untranslated, fallback, unused, words }] of statsMap.entries()) {
      if (lng === nativeLang) {
        continue;
      }
      // @ts-ignore
      table.push([
        lng,
        scanned,
        colors.green(translated),
        colors.red(untranslated),
        colors.yellow(fallback),
        colors.gray(unused),
        colors.blue(words),
      ]);
    }
    if (typeof onFinish === 'function') {
      onFinish(table.toString());
    }

    if (nativeLang === 'zh') {
      outputSuggestions.call(this, appPath, projectPath, unmarked);
    }

    // 检查是否存在写死的腾讯云域名
    outputSuggestions.call(
      this,
      appPath,
      projectPath,
      hardCodeSuggestions,
      'hard-code-of-domain',
      '⚠️  发现代码中存在写死域名',
      '，需要使用平台提供的全局常量代替，详情参考：http://tapd.oa.com/tcp_access/markdown_wikis/show/#1220399462001958727@toc2 ',
    );

    // 检查是否存在字符串拼接
    outputSuggestions.call(
      this,
      appPath,
      projectPath,
      noStringConcatenations,
      'no-string-concat',
      '⚠️  发现代码中存在字符串拼接',
      '，需要使用平台提供的全局常量代替，详情参考：http://tapd.oa.com/tcp_access/markdown_wikis/show/#1220399462001958727@toc2 ',
    );

    cb();
  }

  return through2.obj(write, flush);
}

function outputSuggestions(
  appPath,
  projectPath,
  suggestions,
  type = 'unmarked',
  tipsText = '⚠️  发现未标记的中文字符',
  extraTips = '',
) {
  if (!Array.isArray(suggestions) || suggestions.length <= 0) {
    return;
  }

  const suggestionFilePath = path.resolve(projectPath, `${type}.md`);
  if (fs.existsSync(suggestionFilePath)) {
    rimraf.sync(suggestionFilePath);
  }
  if (suggestions.length) {
    console.error(colors.yellow(`\n${tipsText} ${suggestions.length} 处${extraTips}：\n`));
    suggestions.forEach(({ value, filepath, loc: { start } }, index) => {
      if (index >= 3) return;
      console.error(
        `  ${colors.white(`"${value}"`)} ${colors.blue.underline(
          `${path.relative(path.resolve('.'), filepath)}:${start.line}:${start.column + 1}`,
        )}`,
      );
    });
  }

  this.push(
    new Vinyl({
      cwd: appPath,
      base: appPath,
      path: suggestionFilePath,
      contents: Buffer.from(
        suggestions
          .map(
            ({ value, filepath, loc }) =>
              `- [${value.trim()}](../${path.relative(path.resolve('.'), filepath).replace(/\\/g, '/')}#L${
                loc.start.line
              })`,
          )
          .join('\n'),
      ),
    }),
  );

  if (suggestions.length > 3) {
    console.error('  ...');
    console.error(
      colors.yellow(
        `\n  完整结果请查看 ${colors.blue.underline(path.relative(path.resolve('.'), suggestionFilePath))}`,
      ),
    );
  }
}

/**
 * 读取现有词条
 * @param {string} appPath
 * @param {string} savePath
 * @param {string} lng
 * @returns {{ translation: {[key: string]: string }}}
 */
function read(appPath, savePath, lng) {
  try {
    return require(path.join(appPath, savePath.replace(/{{lng}}/g, lng)));
  } catch (err) {
    return { translation: {} };
  }
}

module.exports = writer;
