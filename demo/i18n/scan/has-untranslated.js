const fs = require('fs');
const path = require('path');
const defaultLng = require('../../lib/i18n').getDefaultLng();

/**
 * 是否有未翻译词条
 * @param {string} lng 词条语言
 * @param {string} dir 词条目录
 */
function hasUntranslated(lng, dir = 'i18n') {
  if (lng === defaultLng) {
    return false;
  }
  const filePath = path.resolve(dir, 'stats.json');
  try {
    if (fs.existsSync(filePath)) {
      const content = require(filePath);
      return content.stats.unmarked > 0 || content[lng].untranslated.length > 0;
    }
  } catch (_) {}
  return false;
}

module.exports = hasUntranslated;
