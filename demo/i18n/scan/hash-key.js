const hashString = require('hash-string');
const colors = require('colors');

const keyMap = new Map();

// 从句子计算哈希一个 key 值，该算法需要和 scanner 保持一致
const hashKey = (value, context, onError) => {
  const key = 'k_' + ('0000' + hashString(value.replace(/\s+/g, '')).toString(36)).slice(-7);
  const existedValue = keyMap.get(context ? `${key}_${context}` : key);
  if (existedValue && existedValue !== value) {
    onError &&
      onError((filepath) => {
        console.error('');
        console.error(colors.yellow(filepath));
        console.error(colors.red('Same sentence in different forms found:'));
        console.error(`    "${existedValue}"`);
        console.error(`    "${value}"`);
      });
  } else {
    keyMap.set(context ? `${key}_${context}` : key, value);
  }
  return key;
};

module.exports = hashKey;
