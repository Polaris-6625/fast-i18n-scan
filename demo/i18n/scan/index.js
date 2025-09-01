const path = require('path');
const vfs = require('vinyl-fs');
const fs = require('fs');
const map = require('map-stream');
const sort = require('gulp-sort');
const ora = require('ora');
const colors = require('colors');
const cliTruncate = require('cli-truncate');
const scanner = require('./scanner');
const writer = require('./writer');
const defaultConfig = require('./config');

const errorCollector = [];

/**
 * 扫描源码中的词条，合并到词条文件中
 * @param {string} appPath 要扫描的应用目录
 * @param {object} config 配置信息
 */
const run = (appPath, config) => {
  let fileCount = 0;
  let scannedFileCount = 0;
  const options = Object.assign({}, defaultConfig, config);
  // 自定义输出目录
  if (config.output) {
    Object.assign(options, {
      resource: {
        loadPath: '',
        savePath: path.resolve(config.output, 'translation/{{lng}}.js'),
      },
    });
  }

  // 如果使用 导出未翻译词条文件的指令，则创建 untranslated 目录
  if (config.exportUntranslated) {
    const untranslatedDir = path.join(appPath, 'i18n', 'untranslated');
    if (!fs.existsSync(untranslatedDir)) {
      fs.mkdirSync(untranslatedDir);
    }
  }

  const spinner = ora({ interval: 66 }).start('Scanning...');
  vfs
    .src(options.input, { cwd: appPath, allowEmpty: true })
    // 每个文件扫描前，统计文件数量
    .pipe(
      // @ts-ignore
      map((file, cb, count) => {
        // 支持 `// @i18n-noscan` 忽略扫描
        if (file.contents === null || /\/\/\s*@i18n-noscan\s/.test(file.contents.toString())) {
          return cb();
        }
        ++fileCount;
        cb(null, file);
      }),
    )

    // 对文件进行排序，保证词条有一定的顺序
    // @ts-ignore
    .pipe(sort())

    // 扫描词条
    .pipe(
      scanner(
        options,
        (file) => {
          ++scannedFileCount;
          const percent = Math.round((100 * scannedFileCount) / fileCount);
          const text = `Scanning (${percent}%): ${colors.green(file.path)}`;
          spinner.text = cliTruncate(text, process.stdout.columns - 5, {
            position: 'middle',
            // ellipsis: " ... ",
          });
        },
        (error) => errorCollector.push(error),
        config.exportUntranslated,
      ),
    )

    // 创建词条文件
    .pipe(
      writer(
        appPath,
        options,
        (stats) => {
          spinner.stop();
          console.log(`Scanned ${fileCount} files. Occured ${errorCollector.length} errors.`);
          console.log(stats);
        },
        config.exportUntranslated,
      ),
    )

    // 写入词条文件
    .pipe(vfs.dest(appPath))

    .on('end', () => {
      errorCollector.forEach((errorHandler) => errorHandler());
      if (errorCollector.length) {
        console.log(
          '\n以上错误可能是由不规范的词条标记导致，标记规范可见：%s',
          colors.underline.blue('http://tapd.oa.com/tcp_access/markdown_wikis/view/#1020399462008817031'),
        );
      }
    });
};

module.exports = { run };
