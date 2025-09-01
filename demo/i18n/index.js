import i18next from 'i18next';
import { I18nextProvider, Trans, reactI18nextModule } from 'react-i18next';
import hashString from 'hash-string';

// 从句子计算哈希一个 key 值，该算法需要和 scanner 保持一致
const hashKey = (value) => `k_${`0000${hashString(value.replace(/\s+/g, '')).toString(36)}`.slice(-7)}`;

export const LANGS = {
  ZH: 'zh',
  EN: 'en',
};

/**
 * 初始化当前语言的国际化配置
 */
export const init = ({ translation }, lng = LANGS.ZH) => {
  if (i18next.isInitialized) {
    // @ts-ignore
    if (process.env.NODE_ENV !== 'production') {
      console.warn('你已经初始化过 i18n，请勿重复初始化');
    }
    return;
  }

  const opts = {
    // 使用语言
    lng,

    // 英文版 fallback 到中文版，其它语言 fallback 到英文版
    fallbackLng: lng === LANGS.EN ? LANGS.ZH : LANGS.EN,

    // 翻译资源
    resources: {
      [lng]: {
        translation: Object.assign({}, translation),
      },
    },

    ns: 'translation',
    defaultNS: 'translation',

    interpolation: {
      escapeValue: false, // not needed for react as it escapes by default
    },

    react: {
      hashTransKey: hashKey,
    },
  };

  i18next.use(reactI18nextModule).init(opts);

  return i18next;
};

/**
 * 标记翻译句子
 * 详细的标记说明，请参考 http://tapd.oa.com/tcp_access/markdown_wikis/0#1020399462009480783
 */
export const t = (sentence, options) => {
  if (!i18next.isInitialized) {
    console.warn('未初始化 i18n 实例，使用默认值自动初始化');
    init({ translation: {} }, LANGS.ZH);
  }
  if (!sentence) return sentence;
  const key = hashKey(sentence);
  return (
    i18next.t(key, {
      ...(options || {}),
      defaultValue: sentence,
    }) || sentence
  );
};

export const getI18NInstance = () => i18next;

export { I18nextProvider, Trans };
