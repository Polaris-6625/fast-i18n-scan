const _ = require('lodash');
const ensureArray = require('ensure-array');
const { isLiteral, isTemplateLiteral } = require('@babel/types');
const { parse } = require('@babel/parser');
const colors = require('colors');

// http://codereview.stackexchange.com/questions/45991/balanced-parentheses
const matchBalancedParentheses = (str = '') => {
  const parentheses = '[]{}()';
  const stack = [];
  let bracePosition;
  let start = -1;
  let i = 0;

  str = '' + str; // ensure string
  for (i = 0; i < str.length; ++i) {
    if (start >= 0 && stack.length === 0) {
      return str.substring(start, i);
    }

    bracePosition = parentheses.indexOf(str[i]);
    if (bracePosition < 0) {
      continue;
    }
    if (bracePosition % 2 === 0) {
      if (start < 0) {
        start = i; // remember the start position
      }
      stack.push(bracePosition + 1); // push next expected brace position
      continue;
    }

    if (stack.pop() !== bracePosition) {
      return str.substring(start, i);
    }
  }

  return str.substring(start, i);
};

function mixParseFuncFromStringByBabel(parser) {
  parser.parseFuncFromStringByBabel = function (content, opts = {}, customHandler = null, onError) {
    if (_.isFunction(opts)) {
      customHandler = opts;
      opts = {};
    }

    const funcs = opts.list !== undefined ? ensureArray(opts.list) : ensureArray(this.options.func.list);

    if (funcs.length === 0) {
      return this;
    }

    const matchFuncs = funcs
      .map((func) => '(?:' + func + ')')
      .join('|')
      .replace(/\./g, '\\.');
    // `\s` matches a single whitespace character, which includes spaces, tabs, form feeds, line feeds and other unicode spaces.
    const matchSpecialCharacters = '[\\r\\n\\s]*';
    const stringGroup =
      matchSpecialCharacters +
      '(' +
      // backtick (``)
      '`(?:[^`\\\\]|\\\\(?:.|$))*`' +
      '|' +
      // double quotes ("")
      '"(?:[^"\\\\]|\\\\(?:.|$))*"' +
      '|' +
      // single quote ('')
      "'(?:[^'\\\\]|\\\\(?:.|$))*'" +
      ')' +
      matchSpecialCharacters;
    const pattern =
      '(?:(?:^\\s*)|[^a-zA-Z0-9_])' +
      '(?:' +
      matchFuncs +
      ')' +
      '\\(' +
      stringGroup +
      '(?:[\\,]' +
      stringGroup +
      ')?' +
      '[\\,\\)]';
    const re = new RegExp(pattern, 'gim');

    let r;
    while ((r = re.exec(content))) {
      const options = {};
      const full = r[0];

      let key = this.fixStringAfterRegExp(r[1], true);
      if (!key) {
        continue;
      }

      if (r[2] !== undefined) {
        const defaultValue = this.fixStringAfterRegExp(r[2], false);
        if (!defaultValue) {
          continue;
        }
        options.defaultValue = defaultValue;
      }

      const endsWithComma = full[full.length - 1] === ',';
      if (endsWithComma) {
        const {
          propsFilter = undefined,
          filepath = undefined,
          babylonOptions = this.options.func.babylon,
        } = {
          ...opts,
        };
        let code = matchBalancedParentheses(content.substr(re.lastIndex));

        if (typeof propsFilter === 'function') {
          code = propsFilter(code);
        }

        try {
          const syntax =
            code.trim() !== ''
              ? parse('(' + code + ')', {
                  ...babylonOptions,
                })
              : {};

          const props = _.get(syntax, 'program.body[0].expression.properties') || [];

          // http://i18next.com/docs/options/
          const supportedOptions = [
            'defaultValue',
            'defaultValue_plural',
            'count',
            'context',
            'ns',
            'keySeparator',
            'nsSeparator',
          ];

          props.forEach((prop) => {
            if (_.includes(supportedOptions, prop.key.name)) {
              const value = prop.value;
              if (isLiteral(value)) {
                options[prop.key.name] = value['value'];
              } else if (isTemplateLiteral(value)) {
                options[prop.key.name] = value.quasis.map((element) => element.value.cooked).join('');
              } else {
                // Unable to get value of the property
                options[prop.key.name] = '';
              }
            }
          });
        } catch (err) {
          onError(() => {
            console.error('');
            const { line, column } = (err && err.loc) || { line: 1, column: 1 };
            console.error(colors.yellow([filepath, line, column].join(':')));
            console.error(colors.cyan(`\nUnable to parse code "${colors.green(code)}"\n`));
            console.error(colors.red('    ' + err.message));
          });
        }
      }

      if (customHandler) {
        customHandler(key, options);
        continue;
      }

      this.set(key, options);
    }

    return this;
  };
}

module.exports = mixParseFuncFromStringByBabel;
