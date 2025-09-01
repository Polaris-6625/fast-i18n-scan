import i18next from 'i18next';
import { I18nextProvider, Trans } from 'react-i18next';


type ProviderType = typeof I18nextProvider;
type TransType = typeof Trans;

interface I18NTranslation {
  [key: string]: string;
}

interface I18NInitOptions {
  translation: I18NTranslation;
}

export interface I18NTranslationOptions {
  /** 用于确定单复数的数量值 */
  count?: number;

  /** 用于确定上下文的说明文本，只能使用字符串常量，否则无法扫描 */
  context?: string;

  // 允许传入插值
  [key: string]: any;
}

declare module '@tcwd/tcb-i18n' {
  export const init: (opts: I18NInitOptions, lang: 'zh' | 'en') => i18next.i18n;

  /**
   * 标记翻译句子
   * 详细的标记说明，请参考 http://tapd.oa.com/tcp_access/markdown_wikis/0#1020399462009480783
   */
  export const t: (sentence: string, options?: I18NTranslationOptions) => string;

  export const getI18NInstance: () => i18next.i18n;

  export const I18nextProvider: ProviderType;

  export const Trans: TransType;

  export enum LANGS {
    ZH = 'zh',
    EN = 'en',
  }
}

