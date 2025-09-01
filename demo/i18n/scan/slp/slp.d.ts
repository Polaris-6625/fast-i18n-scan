export function getBaseKey(key: string): string;

export function createSisulizerProject(options: { nativeLang: string; langs: string[] }): {
  load(filePath: sourcePath, lang: string): Promise<void>;
  add(key: string, lang: string, translatedString: string): void;
  get(key: string, lang: string): string;
  keys(): string[];
  obsolete(key: string): void;
  output(lang?: string): string;
};

export interface SourceRow {
  key: string;
  /**
   * * 0 = new
   * * 1 = no usage
   * * 2 = usage
   * * 3 = changed
   */
  state?: '0' | '1' | '2' | '3';
  nativeString: string;
  translateMap: Map<string, string>;
}

export interface ProjectXMLDocument {
  document: Document;
}

export interface Document {
  $: DocumentClass;
  lang: DocumentLang[];
  source: SourceElement[];
}

export interface DocumentClass {
  created: string;
  version: string;
  date: string;
  scan: string;
  scanned: string;
}

export interface DocumentLang {
  $: Lang;
}

export interface Lang {
  id: string;
  state: '0' | '1' | '2' | '3';
  locked: '1';
  marked: '1';
}

export interface SourceElement {
  $: Source;
  row: Row[];
}

export interface Source {
  class: string;
  name: string;
  original: string;
  date: string;
}

export interface Row {
  _?: string;
  $: Lang;
  native: string[];
  lang: RowLang[];
}

export interface RowLang {
  _: string;
  $: Lang;
}
