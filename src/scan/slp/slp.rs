use colored::*;
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// 翻译行记录
#[derive(Debug, Clone)]
pub struct SourceRow {
    pub key: String,
    pub native_string: Option<String>,
    pub translate_map: HashMap<String, String>,
}

impl SourceRow {
    pub fn new(key: String) -> Self {
        Self {
            key,
            native_string: None,
            translate_map: HashMap::new(),
        }
    }
}

/// Sisulizer 项目选项
#[derive(Debug, Clone)]
pub struct SisulizerProjectOptions {
    pub native_lang: String,
    pub langs: Vec<String>,
}

impl Default for SisulizerProjectOptions {
    fn default() -> Self {
        Self {
            native_lang: "zh".to_string(),
            langs: vec!["en".to_string()],
        }
    }
}

/// Sisulizer 项目管理器
pub struct SisulizerProject {
    /// 本地语言，应该为 'zh'
    native_lang: String,
    /// 要国际化的语言列表
    langs: Vec<String>,
    /// 翻译行记录
    row_map: HashMap<String, SourceRow>,
    /// 标记为废弃的行
    obsoleted_set: HashSet<String>,
    /// 中文字符正则表达式
    chinese_regex: Regex,
}

impl SisulizerProject {
    /// 创建新的 Sisulizer 项目
    pub fn new(options: Option<SisulizerProjectOptions>) -> Self {
        let options = options.unwrap_or_default();
        let native_lang = options.native_lang;
        let langs: Vec<String> = options
            .langs
            .into_iter()
            .filter(|x| x != &native_lang)
            .collect();

        Self {
            native_lang,
            langs,
            row_map: HashMap::new(),
            obsoleted_set: HashSet::new(),
            chinese_regex: Regex::new(r"[\u4e00-\u9fa5]").unwrap(),
        }
    }

    /// 创建新的翻译行
    fn create_row(&mut self, key: &str) -> &mut SourceRow {
        let row = SourceRow::new(key.to_string());
        self.row_map.insert(key.to_string(), row);
        self.row_map.get_mut(key).unwrap()
    }

    /// 获取指定 Key 的翻译行，如不存在则创建
    pub fn get_or_create_row(&mut self, key: &str) -> &mut SourceRow {
        if !self.row_map.contains_key(key) {
            self.create_row(key);
        }
        self.row_map.get_mut(key).unwrap()
    }

    /// 添加翻译数据
    pub fn add(&mut self, key: &str, lang: &str, translated_string: &str) {
        let native_string = self.get(key, &self.native_lang.clone());
        
        // 如果就是本地语言，直接写到 native_string 上
        if lang == self.native_lang {
            let row = self.get_or_create_row(key);
            row.native_string = Some(translated_string.to_string());
        }
        // 如果翻译句子和本地句子一样，则不写入
        else if !translated_string.is_empty() 
            && native_string.is_some() 
            && !self.is_same_sentence(translated_string, &native_string.unwrap()) {
            let row = self.get_or_create_row(key);
            row.translate_map.insert(lang.to_string(), translated_string.to_string());
        }
    }

    /// 获取翻译数据
    pub fn get(&self, key: &str, lang: &str) -> Option<String> {
        let row = self.row_map.get(key)?;
        
        if lang == self.native_lang {
            row.native_string.clone().or_else(|| self.try_find_native_string(key))
        } else {
            row.translate_map.get(lang).cloned()
        }
    }

    /// 获取所有的翻译键
    pub fn keys(&self) -> Vec<String> {
        self.row_map.keys().cloned().collect()
    }

    /// 标记为废弃词条
    pub fn obsolete(&mut self, key: &str) {
        self.obsoleted_set.insert(key.to_string());
    }

    /// 从指定的目录中加载数据
    pub async fn load(&mut self, source_path: &str, lang: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = Path::new(source_path).join(format!("{}.json", lang));
        
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let data: Value = serde_json::from_str(&content)?;
                
                if let Some(obj) = data.as_object() {
                    for (key, value) in obj {
                        if let Some(translated_string) = value.as_str() {
                            let native_string = self.get(key, &self.native_lang.clone());
                            
                            if lang == self.native_lang {
                                let row = self.get_or_create_row(key);
                                row.native_string = Some(translated_string.to_string());
                            }
                            // 认为 json 中一定是翻译后的词条，不进行 is_same_sentence 比较
                            else if !translated_string.is_empty() && native_string.is_some() {
                                // 检查非中文词条中是否包含中文
                                if lang != "zh" && self.chinese_regex.is_match(translated_string) {
                                    return Err(format!(
                                        "{}\n  {}\n{}",
                                        format!("\n\n{}.json 词条中发现中文：\n", lang).red(),
                                        translated_string,
                                        "请检查词条文件\n".red()
                                    ).into());
                                }
                                let row = self.get_or_create_row(key);
                                row.translate_map.insert(lang.to_string(), translated_string.to_string());
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(error) => {
                eprintln!("Error loading file {}: {}", file_path.display(), error);
                std::process::exit(1);
            }
        }
    }

    /// 导出要保存的 JSON 内容
    pub fn output(&self, lang: Option<&str>) -> String {
        let target_lang = lang.unwrap_or(&self.native_lang);
        let mut json_obj = serde_json::Map::new();
        
        let mut sorted_keys = self.keys();
        sorted_keys.sort();
        
        for key in sorted_keys {
            let base_key = get_base_key(&key);
            if !self.obsoleted_set.contains(&base_key) {
                if let Some(value) = self.get(&key, target_lang) {
                    json_obj.insert(key, Value::String(value));
                }
            }
        }
        
        serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
    }

    /// 输出到目录结构 (新增方法)
    pub fn output_to_directory(&self, output_dir: &str, lang: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let target_lang = lang.unwrap_or(&self.native_lang);
        let output_path = Path::new(output_dir);
        
        // 创建输出目录
        fs::create_dir_all(output_path)?;
        
        // 创建 context 和 source 子目录
        let context_dir = output_path.join("context");
        let source_dir = output_path.join("source");
        fs::create_dir_all(&context_dir)?;
        fs::create_dir_all(&source_dir)?;
        
        // 生成 context.json
        let stats = self.get_stats();
        let context_data = serde_json::json!({
            "language": target_lang,
            "total_keys": stats.total_keys,
            "active_keys": stats.active_keys,
            "obsoleted_keys": stats.obsoleted_keys,
            "generated_at": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "project_info": {
                "native_language": self.native_lang,
                "available_languages": self.langs.clone()
            }
        });
        
        let context_file = context_dir.join("context.json");
        fs::write(context_file, serde_json::to_string_pretty(&context_data)?)?;
        
        // 生成 source/zh.json (或其他语言) - 使用哈希键
        let mut json_obj = serde_json::Map::new();
        let mut sorted_keys = self.keys();
        sorted_keys.sort();
        
        for key in sorted_keys {
            let base_key = get_base_key(&key);
            if !self.obsoleted_set.contains(&base_key) {
                if let Some(value) = self.get(&key, target_lang) {
                    // 生成哈希键
                    let hash_key = crate::scan::hash_key::hash_key(&value, None, None);
                    json_obj.insert(hash_key, Value::String(value));
                }
            }
        }
        
        let source_file = source_dir.join(format!("{}.json", target_lang));
        fs::write(source_file, serde_json::to_string_pretty(&json_obj)?)?;
        
        Ok(())
    }

    /// 翻译的词条 Key 可能是带上下文或者带复数形式的
    fn try_find_native_string(&self, key: &str) -> Option<String> {
        let base_key = get_base_key(key);
        let mut possible_keys = vec![base_key.clone()];
        
        // 中文复数形式
        possible_keys.push(format!("{}_{}", base_key, "0"));
        
        for possible_key in possible_keys {
            if let Some(row) = self.row_map.get(&possible_key) {
                if let Some(native_string) = &row.native_string {
                    return Some(native_string.clone());
                }
            }
        }
        
        None
    }

    /// 判断两个句子是否相同（忽略空白字符）
    fn is_same_sentence(&self, a: &str, b: &str) -> bool {
        let clean_a = a.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        let clean_b = b.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        clean_a == clean_b
    }

    /// 获取本地语言
    pub fn get_native_lang(&self) -> &str {
        &self.native_lang
    }

    /// 获取支持的语言列表
    pub fn get_langs(&self) -> &[String] {
        &self.langs
    }

    /// 检查是否存在指定的键
    pub fn has_key(&self, key: &str) -> bool {
        self.row_map.contains_key(key)
    }

    /// 获取废弃的键集合
    pub fn get_obsoleted_keys(&self) -> &HashSet<String> {
        &self.obsoleted_set
    }

    /// 清空所有数据
    pub fn clear(&mut self) {
        self.row_map.clear();
        self.obsoleted_set.clear();
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> ProjectStats {
        let total_keys = self.row_map.len();
        let obsoleted_keys = self.obsoleted_set.len();
        let active_keys = total_keys - obsoleted_keys;
        
        let mut lang_stats = HashMap::new();
        for lang in &self.langs {
            let translated_count = self.row_map
                .values()
                .filter(|row| row.translate_map.contains_key(lang))
                .count();
            lang_stats.insert(lang.clone(), translated_count);
        }
        
        ProjectStats {
            total_keys,
            active_keys,
            obsoleted_keys,
            lang_stats,
        }
    }
}

/// 项目统计信息
#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub obsoleted_keys: usize,
    pub lang_stats: HashMap<String, usize>,
}

/// 得到不包含单复数形式的 Key
pub fn get_base_key(key: &str) -> String {
    let regex = Regex::new(r"^(k_.+)_(\d|plural)$").unwrap();
    
    if let Some(captures) = regex.captures(key) {
        captures.get(1).map_or(key.to_string(), |m| m.as_str().to_string())
    } else {
        key.to_string()
    }
}

/// 创建 Sisulizer 项目
pub fn create_sisulizer_project(options: Option<SisulizerProjectOptions>) -> SisulizerProject {
    SisulizerProject::new(options)
}

/// 创建默认的 Sisulizer 项目
pub fn create_default_sisulizer_project() -> SisulizerProject {
    SisulizerProject::new(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_get_base_key() {
        assert_eq!(get_base_key("k_hello_world"), "k_hello_world");
        assert_eq!(get_base_key("k_hello_world_0"), "k_hello_world");
        assert_eq!(get_base_key("k_hello_world_plural"), "k_hello_world");
        assert_eq!(get_base_key("k_hello_world_1"), "k_hello_world");
        assert_eq!(get_base_key("normal_key"), "normal_key");
    }

    #[test]
    fn test_is_same_sentence() {
        let project = SisulizerProject::new(None);
        assert!(project.is_same_sentence("Hello World", "HelloWorld"));
        assert!(project.is_same_sentence("Hello  World", "Hello World"));
        assert!(project.is_same_sentence("  Hello World  ", "HelloWorld"));
        assert!(!project.is_same_sentence("Hello World", "Hello Universe"));
    }

    #[test]
    fn test_add_and_get() {
        let mut project = SisulizerProject::new(None);
        
        // 添加本地语言
        project.add("greeting", "zh", "你好");
        assert_eq!(project.get("greeting", "zh"), Some("你好".to_string()));
        
        // 添加翻译
        project.add("greeting", "en", "Hello");
        assert_eq!(project.get("greeting", "en"), Some("Hello".to_string()));
        
        // 相同句子不应该被添加
        project.add("greeting", "en", "你好");
        assert_eq!(project.get("greeting", "en"), Some("Hello".to_string()));
    }

    #[test]
    fn test_obsolete() {
        let mut project = SisulizerProject::new(None);
        project.add("old_key", "zh", "旧词条");
        project.obsolete("old_key");
        
        assert!(project.get_obsoleted_keys().contains("old_key"));
    }

    #[test]
    fn test_keys() {
        let mut project = SisulizerProject::new(None);
        project.add("key1", "zh", "值1");
        project.add("key2", "zh", "值2");
        
        let keys = project.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[test]
    fn test_output() {
        let mut project = SisulizerProject::new(None);
        project.add("greeting", "zh", "你好");
        project.add("greeting", "en", "Hello");
        project.add("farewell", "zh", "再见");
        project.obsolete("farewell");
        
        let output = project.output(Some("zh"));
        let parsed: Value = serde_json::from_str(&output).unwrap();
        
        assert!(parsed.get("greeting").is_some());
        assert!(parsed.get("farewell").is_none()); // 应该被过滤掉
    }

    #[tokio::test]
    async fn test_load() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // 创建测试 JSON 文件
        let zh_content = r#"{"greeting": "你好", "farewell": "再见"}"#;
        fs::write(temp_path.join("zh.json"), zh_content).unwrap();
        
        let en_content = r#"{"greeting": "Hello", "farewell": "Goodbye"}"#;
        fs::write(temp_path.join("en.json"), en_content).unwrap();
        
        let mut project = SisulizerProject::new(None);
        
        // 加载中文
        project.load(temp_path.to_str().unwrap(), "zh").await.unwrap();
        assert_eq!(project.get("greeting", "zh"), Some("你好".to_string()));
        
        // 加载英文
        project.load(temp_path.to_str().unwrap(), "en").await.unwrap();
        assert_eq!(project.get("greeting", "en"), Some("Hello".to_string()));
    }

    #[tokio::test]
    async fn test_load_with_chinese_in_english() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // 创建包含中文的英文文件
        let en_content = r#"{"greeting": "你好"}"#;
        fs::write(temp_path.join("en.json"), en_content).unwrap();
        
        let zh_content = r#"{"greeting": "你好"}"#;
        fs::write(temp_path.join("zh.json"), zh_content).unwrap();
        
        let mut project = SisulizerProject::new(None);
        project.load(temp_path.to_str().unwrap(), "zh").await.unwrap();
        
        // 加载包含中文的英文文件应该出错
        let result = project.load(temp_path.to_str().unwrap(), "en").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_get_stats() {
        let mut project = SisulizerProject::new(None);
        project.add("key1", "zh", "值1");
        project.add("key1", "en", "Value1");
        project.add("key2", "zh", "值2");
        project.obsolete("key2");
        
        let stats = project.get_stats();
        assert_eq!(stats.total_keys, 2);
        assert_eq!(stats.active_keys, 1);
        assert_eq!(stats.obsoleted_keys, 1);
        assert_eq!(stats.lang_stats.get("en"), Some(&1));
    }
}