const _ = require('lodash');
const traverse = require('@babel/traverse').default;
const { parse } = require('@babel/parser');
const t = require('@babel/types');
const colors = require('colors');
const nodesToString = require('./nodes-to-string');
const { verify } = require('./zh-linter');
const ensureArray = require('ensure-array');

const getLiteralValue = (literal) => {
  if (t.isTemplateLiteral(literal)) {
    return literal.quasis.map((element) => element.value.cooked).join('');
  }
  return literal.value;
};

const buildJSXPath = (node, parentPath, filepath, JSXPathdefaultValue) => {
  const JSXPath = JSXPathdefaultValue ?? [
    {
      id: '',
      path: [
        {
          name: node.openingElement.name.name,
          attributes: (node.openingElement.attributes ?? []).map((attrNode) => {
            return {
              name: attrNode.type === 'JSXSpreadAttribute' ? attrNode?.argument?.name : attrNode?.name?.name,
              value: nodesToString([attrNode.value], filepath, () => {}),
            };
          }),
          // attributes: (node?.openingElement?.attributes ?? []).map(attrNode =>
          //   nodesToString([attrNode])
          // ),
        },
      ],
    },
  ];
  let id;
  let currentParent = parentPath;

  while (currentParent) {
    let name = '';
    let attributes = [];
    switch (currentParent.node.type) {
      // case "ArrowFunctionExpression": {
      //   name = currentParent.node?.body?.openingElement?.name?.name;
      //   break;
      // }
      case 'JSXElement': {
        name = currentParent.node?.openingElement?.name?.name;
        attributes = (currentParent.node.openingElement.attributes ?? []).map((attrNode) => {
          return {
            name: attrNode.type === 'JSXSpreadAttribute' ? attrNode?.argument?.name : attrNode?.name?.name,
            value: nodesToString([attrNode.value], filepath, () => {}),
          };
        });
        break;
      }
      // case "ReturnStatement": {

      // }
      case 'VariableDeclaration': {
        id = currentParent.node?.declarations?.[0]?.id?.name;
        break;
      }
      case 'VariableDeclarator': {
        // 在VariableDeclaration 中获取过，不重复获取
        break;
      }
      case 'ClassDeclaration':
      case 'FunctionDeclaration': {
        id = currentParent.node?.id?.name;
        break;
      }
      default: {
        break;
      }
    }
    if (name) {
      JSXPath[0].path.unshift({
        name,
        attributes,
      });
    } else {
      // debugger;
    }
    if (id) {
      JSXPath[0].id = id;
      JSXPath.unshift({
        id: '',
        path: [],
      });
      id = '';
    }
    currentParent = currentParent.parentPath;
  }
  return JSXPath;
};

function mixParseTransFromStringByBabel(parser) {
  parser.parseTransFromStringByBabel = function (content, opts = {}, customHandler = null, onError = () => {}) {
    if (_.isFunction(opts)) {
      customHandler = opts;
      opts = {};
    }
    let currentScopePath = [];
    const {
      transformOptions = {}, // object
      component = this.options.trans.component, // string
      i18nKey = this.options.trans.i18nKey, // string
      defaultsKey = this.options.trans.defaultsKey, // string
      fallbackKey = this.options.trans.fallbackKey, // boolean|function
      babylon: babylonOptions = this.options.trans.babylon, // object
      // @ts-ignore
      filepath,
    } = { ...opts };
    const funcs = opts.list !== undefined ? ensureArray(opts.list) : ensureArray(this.options.func.list);
    const JSXContext = [];

    const parseJSXElement = (params) => {
      const { node, parentPath } = params;
      if (!node) {
        return;
      }

      const JSXPath = buildJSXPath(node, parentPath, filepath);

      if (node.openingElement.name.name !== component) {
        return;
      }

      const attr = _.castArray(node.openingElement.attributes).reduce((acc, attribute) => {
        if (!t.isJSXAttribute(attribute) || !t.isJSXIdentifier(attribute.name)) {
          return acc;
        }
        const { name } = attribute.name;
        const value = attribute.value;
        if (t.isLiteral(value)) {
          acc[name] = getLiteralValue(value);
        } else if (t.isJSXExpressionContainer(value)) {
          const expression = value.expression;
          if (t.isIdentifier(expression)) {
            acc[name] = expression.name;
          } else if (t.isLiteral(expression)) {
            acc[name] = getLiteralValue(expression);
          } else if (t.isObjectExpression(expression)) {
            const properties = _.castArray(expression.properties);
            acc[name] = properties.reduce((obj, property) => {
              if (!t.isObjectProperty(property)) {
                return obj;
              }
              if (t.isLiteral(property.value)) {
                obj[property.key.name] = getLiteralValue(property.value);
              } else {
                // Unable to get value of the property
                obj[property.key.name] = '';
              }
              return obj;
            }, {});
            /**
             * 防止 count 被忽略，如
             * ```jsx
             * <Trans count={arr.length}>
             *  一二三{{ count: arr.length }}
             * </Trans>
             * ```
             */
          } else if (name === 'count') {
            acc[name] = 0;
          }
        }
        return acc;
      }, {});
      const transKey = _.trim(attr[i18nKey]);

      const defaultsString = attr[defaultsKey] || '';
      if (typeof defaultsString !== 'string') {
        this.log(`defaults value must be a static string, saw ${colors.yellow(defaultsString)}`);
      }

      // https://www.i18next.com/translation-function/essentials#overview-options
      const tOptions = attr.tOptions;
      const options = {
        ...tOptions,
        defaultValue: defaultsString || nodesToString(node.children, filepath, onError),
        fallbackKey,
        JSXPath,
      };

      if (Object.prototype.hasOwnProperty.call(attr, 'count')) {
        options.count = Number(attr.count) || 0;
      }

      if (Object.prototype.hasOwnProperty.call(attr, 'ns')) {
        if (typeof options.ns !== 'string') {
          this.log(`The ns attribute must be a string, saw ${colors.yellow(attr.ns)}`);
        }

        options.ns = attr.ns;
      }

      if (customHandler) {
        customHandler(transKey, options);
        return;
      }

      this.set(transKey, options);
    };
    const tFunctionResultsScope = new Map();

    const parseFuncNode = (params) => {
      const { node, parentPath } = params;
      if (!node) {
        return;
      }
      const funcName = node?.callee?.name ?? node?.callee?.property?.name;
      // const JSXPath = buildJSXPath(node, parentPath, filepath);

      if (!funcs.includes(funcName)) {
        return;
      }

      let id;
      // let isFinded = false;
      let currentParent = parentPath;
      const scopePath = currentScopePath.map((p) => p.type).join('.');
      const props = node.arguments?.[1]?.properties ?? [];

      // node.arguments.forEach(argNode => {
      //   switch (argNode.type) {
      //     case "StringLiteral": {
      //     }
      //   }
      // });

      const supportedOptions = [
        'defaultValue',
        'defaultValue_plural',
        'count',
        'context',
        'ns',
        'keySeparator',
        'nsSeparator',
      ];

      const defaultsString = '';
      // if (typeof defaultsString !== "string") {
      //   this.log(
      //     `defaults value must be a static string, saw ${colors.yellow(
      //       defaultsString
      //     )}`
      //   );
      // }
      const defaultValue = defaultsString || nodesToString(node.arguments, filepath, onError);
      const options = {
        defaultValue,
        fallbackKey,
      };
      props.forEach((prop) => {
        if (_.includes(supportedOptions, prop.key.name)) {
          const value = prop.value;
          if (t.isLiteral(value)) {
            options[prop.key.name] = value['value'];
          } else if (t.isTemplateLiteral(value)) {
            options[prop.key.name] = value.quasis.map((element) => element.value.cooked).join('');
          } else {
            // Unable to get value of the property
            options[prop.key.name] = '';
          }
        }
      });
      const JSXPath = [
        {
          id: '',
          path: [],
        },
      ];
      while (currentParent) {
        if (currentParent.node) {
          let name = '';
          let attributes = [];

          switch (currentParent.node.type) {
            // case "ArrowFunctionExpression": {
            //   name = currentParent.node?.body?.openingElement?.name?.name;
            //   break;
            // }
            case 'JSXElement': {
              name = currentParent.node?.openingElement?.name?.name;
              attributes = (currentParent.node.openingElement.attributes ?? []).map((attrNode) => {
                return {
                  name: attrNode.type === 'JSXSpreadAttribute' ? attrNode?.argument?.name : attrNode?.name?.name,
                  value: nodesToString([attrNode.value], filepath, () => {}),
                };
              });
              break;
            }
            // case "ReturnStatement": {

            // }
            case 'VariableDeclaration': {
              id = currentParent.node?.declarations?.[0]?.id?.name;
              break;
            }
            case 'VariableDeclarator': {
              // 在VariableDeclaration 中获取过，不重复获取
              break;
            }
            case 'ClassDeclaration':
            case 'FunctionDeclaration': {
              id = currentParent.node?.id?.name;
              break;
            }
            case 'CallExpression': {
              currentParent;
            }
            default: {
              break;
            }
          }
          if (name) {
            JSXPath[0].path.unshift({
              name,
              attributes,
            });
          }
          if (id) {
            // if (!isFinded) {
            if (!tFunctionResultsScope.has(scopePath)) {
              tFunctionResultsScope.set(scopePath, new Set());
            }
            tFunctionResultsScope.get(scopePath).add({
              id,
              options: {
                ...options,
                JSXPath: JSXPath.map((p) => {
                  return {
                    ...p,
                    path: [...p.path],
                  };
                }),
                // 如果不是第一次，则需要将上一次的 JSXPath 拼接到当前的 JSXPath 中
                // isFinded,
              },
            });
            // isFinded = true;
            // }

            JSXPath[0].id = id;
            JSXPath.unshift({
              id: '',
              path: [],
            });
            id = '';
          }
          currentParent = currentParent.parentPath;
        } else {
          break;
        }
      }

      options.JSXPath = JSXPath;
      const transKey = '';
      if (customHandler) {
        customHandler(transKey, options);
        return;
      }

      this.set(transKey, options);
    };

    const parseIdNode = (params) => {
      const { node, parentPath } = params;
      if (!node) {
        return;
      }

      for (let i = currentScopePath.length - 1; i >= 0; i--) {
        const scopePath = currentScopePath
          .slice(0, i + 1)
          .map((p) => p.type)
          .join('.');
        const variables = tFunctionResultsScope.get(scopePath);
        if (variables) {
          for (const var1 of variables) {
            if (var1.id === node.name) {
              //变量匹配成功，开始生成组件🌲
              const JSXPath = buildJSXPath(
                node,
                parentPath,
                filepath,
                var1.options.JSXPath
                  ? var1.options.JSXPath.map((p) => {
                      return {
                        ...p,
                        path: [...p.path],
                      };
                    })
                  : [
                      {
                        id: '',
                        path: [],
                      },
                    ],
              );

              const options = {
                ...var1.options,
                JSXPath,
              };
              const transKey = '';
              if (customHandler) {
                customHandler(transKey, options);
                continue;
              }

              this.set(transKey, options);
            }
          }
          break;
        }
      }
    };

    try {
      const ast = parse(content, {
        ...babylonOptions,
      });
      traverse(ast, {
        enter: (path) => {
          if (path.isBlockStatement() || path.isFunctionDeclaration() || path.isProgram()) {
            currentScopePath.push(path);
          }
        },
        exit(path) {
          // 离开当前作用域
          if (path.isBlockStatement() || path.isFunctionDeclaration() || path.isProgram()) {
            currentScopePath.pop();
          }
        },
        // 找Trans函数的上下文，以及找引用的变量中，有没有可能为t的值
        JSXElement: parseJSXElement,
        // t被调用时的上下文，并记录作用域
        'CallExpression|OptionalCallExpression': parseFuncNode,
        Identifier: parseIdNode,
      });
      verify({
        // 屏蔽无具体规则禁用
        content: content.replace(/(eslint-(?:en|dis)able(?:(?:-next)?-line)?)/gu, `$1 no-alert,`),
        filepath,
      });
    } catch (err) {
      onError(() => {
        console.error('');
        const { line, column } = (err && err.loc) || { line: 1, column: 1 };
        console.error(colors.yellow([filepath, line, column].join(':')));
        console.error(colors.cyan(`\nUnable to parse ${colors.green(component)} component.\n`));
        if (!filepath) {
          console.error(colors.red(content));
        }
        console.error(colors.red('    ' + err.message));
      });
    }

    return this;
  };
}

module.exports = mixParseTransFromStringByBabel;
