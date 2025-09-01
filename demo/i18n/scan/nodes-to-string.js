const _ = require('lodash');
const t = require('@babel/types');
const colors = require('colors');

const nodesToString = (nodes, filepath, onError) => {
  let memo = '';
  let nodeIndex = 0;
  nodes.forEach((node, i) => {
    if (t.isJSXText(node) || t.isStringLiteral(node)) {
      const value = node.value
        .replace(/^[\r\n]+\s*/g, '') // remove leading spaces containing a leading newline character
        .replace(/[\r\n]+\s*$/g, '') // remove trailing spaces containing a leading newline character
        .replace(/[\r\n]+\s*/g, ' '); // replace spaces containing a leading newline character with a single space character

      if (!value) {
        return;
      }
      memo += value;
    } else if (t.isJSXExpressionContainer(node)) {
      const { expression = {} } = node;

      if (t.isNumericLiteral(expression)) {
        // Numeric literal is ignored in react-i18next
        memo += '';
      }
      if (t.isStringLiteral(expression)) {
        memo += expression.value;
      } else if (t.isObjectExpression(expression) && t.isObjectProperty(_.get(expression, 'properties[0]'))) {
        // @ts-ignore
        memo += `{{${expression.properties[0].key.name}}}`;
      } else if (t.isTemplateLiteral(expression)) {
        expression.quasis.forEach((tmpEle) => (memo += tmpEle?.value?.raw ?? ''));
      } else {
        onError(() => {
          const { line, column } = (node.expression && node.expression.loc.start) || { line: 1, column: 1 };
          console.error('');
          console.error(colors.yellow([filepath, line, column].join(':')));
          console.error(
            colors.red(`Unsupported JSX expression. Only static values or {{interpolation}} blocks are supported.`),
          );
        });
      }
    } else if (node?.children) {
      memo += `<${nodeIndex}>${nodesToString(node.children, filepath, onError)}</${nodeIndex}>`;
    }

    ++nodeIndex;
  });

  return memo;
};

module.exports = nodesToString;
