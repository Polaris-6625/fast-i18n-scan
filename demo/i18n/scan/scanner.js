const path = require('path');
const fs = require('fs');
const i18nScanner = require('i18next-scanner');
const hashKey = require('./hash-key');
const mixParseTransFromStringByBabel = require('./parse-trans-from-string-by-babel');
const mixParseFuncFromStringByBabel = require('./parse-func-from-string-by-babel');
const lngs = require('./lngs');

/**
 * 扫描词条
 */
function scanner(options, onProgress, onError, exportUntranslated) {
  options.untranslated = {};
  const optionsComponent = options.trans.component || 'Trans';
  const projectPath = options.output || 'i18n';
  const projectContextPath = path.resolve(projectPath, 'context');
  if (!fs.existsSync(projectContextPath)) {
    fs.mkdirSync(projectContextPath, {
      recursive: true,
    });
  }
  const context = {};
  /**
   * @param {import('vinyl')} file
   * @param {string} encoding
   * @param {Function} done
   */
  function transform(file, encoding, done) {
    const { parser } = this;
    const extname = path.extname(file.path);
    // 只扫描源码文件
    if (!['.js', '.jsx', '.ts', '.tsx'].includes(extname)) {
      return done();
    }

    onProgress(file);
    const content = file.contents.toString();

    mixParseFuncFromStringByBabel(parser);

    parser.parseFuncFromStringByBabel(
      content,
      { filepath: file.path },
      (sentence, options) => {
        const key = hashKey(sentence, options.context, (callback) => onError(() => callback(file.path)));
        options.defaultValue = sentence;
        parser.set(key, options);
      },
      onError,
    );

    mixParseTransFromStringByBabel(parser);

    parser.parseTransFromStringByBabel(
      content,
      { filepath: file.path },
      (transKey, options) => {
        let sentence = options.defaultValue;
        sentence = sentence.replace(/\s+/g, ' ');
        transKey = transKey || hashKey(sentence);
        options.defaultValue = sentence;
        parser.set(transKey, options);
        context[transKey] = {
          rawText: options.defaultValue,
          staticAnalysis: {
            componentStack: options.JSXPath.flatMap((block) =>
              [block.id].concat(
                block.path
                  .filter((component) => component.name !== optionsComponent)
                  .map((component) => {
                    const classNames = (component?.attributes || []).find((attr) => attr.name === 'className');
                    return `${component.name}${classNames?.value ? `.${classNames?.value}` : ''}`;
                  }),
              ),
            ).filter((stackStr) => stackStr),
          },
        };
        fs.writeFileSync(
          // `${projectContextPath}/${transKey}.json`,
          `${projectContextPath}/context.json`,
          JSON.stringify(context),
        );
      },
      onError,
    );
    setTimeout(() => done(), 0);
  }
  return i18nScanner.createStream(options, transform);
}

module.exports = scanner;
