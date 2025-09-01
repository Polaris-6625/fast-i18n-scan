use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// JavaScript 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsConfig {
    pub input: Vec<String>,
    pub output: String,
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub sort: bool,
    #[serde(default)]
    #[serde(rename = "removeUnusedKeys")]
    pub remove_unused_keys: bool,
    pub lngs: Vec<String>,
    #[serde(rename = "defaultLng")]
    pub default_lng: String,
}

impl Default for JsConfig {
    fn default() -> Self {
        Self {
            input: vec!["./src/**/!(*.d).{js,jsx,ts,tsx}".to_string()],
            output: "./i18n".to_string(),
            debug: false,
            sort: true,
            remove_unused_keys: false,
            lngs: vec!["zh".to_string(), "en".to_string()],
            default_lng: "zh".to_string(),
        }
    }
}

impl JsConfig {
    /// 从 JavaScript 配置文件加载
    pub fn from_js_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::from_js_string(&content)
    }

    /// 从 JavaScript 字符串解析配置
    pub fn from_js_string(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 简单的 JavaScript 配置解析
        // 移除 module.exports = 和注释
        let cleaned = content
            .lines()
            .filter(|line| !line.trim().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        
        // 提取 JSON 部分
        let start = cleaned.find('{').ok_or("Invalid config format")?;
        let end = cleaned.rfind('}').ok_or("Invalid config format")?;
        let json_str = &cleaned[start..=end];
        
        // 处理 JavaScript 对象语法到 JSON
        let json_str = json_str
            .replace("'", "\"")  // 单引号转双引号
            .replace(",\n}", "\n}")  // 移除尾随逗号
            .replace(",\n  }", "\n  }")
            // 为 JavaScript 对象键添加引号
            .replace("input:", "\"input\":")
            .replace("output:", "\"output\":")
            .replace("debug:", "\"debug\":")
            .replace("sort:", "\"sort\":")
            .replace("removeUnusedKeys:", "\"removeUnusedKeys\":")
            .replace("lngs:", "\"lngs\":")
            .replace("defaultLng:", "\"defaultLng\":");
        
        let config: JsConfig = serde_json::from_str(&json_str)?;
        Ok(config)
    }

    /// 转换为扫描配置
    pub fn to_scan_config(&self) -> crate::scan::config::ScanConfig {
        crate::scan::config::ScanConfig {
            input: self.input.clone(),
            lngs: self.lngs.clone(),
            ns: vec!["translation".to_string()],
            default_lng: self.default_lng.clone(),
            default_ns: "translation".to_string(),
            resource: crate::scan::config::ResourceConfig {
                load_path: "".to_string(),
                save_path: format!("{}/{{{{lng}}}}.json", self.output),
            },
            func: crate::scan::config::FuncConfig::default(),
            trans: crate::scan::config::TransConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_js_config() {
        let js_content = r#"
module.exports = {
  input: ['./src/**/!(*.d).{js,jsx,ts,tsx}', '../../packages/admin-component/src/**/!(*.d).{js,jsx,ts,tsx}'],
  output: './i18n',
  debug: true,
  sort: true,
  removeUnusedKeys: false,
  lngs: ['zh', 'en'],
  defaultLng: 'zh',
};
        "#;

        let config = JsConfig::from_js_string(js_content).unwrap();
        assert_eq!(config.input.len(), 2);
        assert_eq!(config.output, "./i18n");
        assert_eq!(config.debug, true);
        assert_eq!(config.default_lng, "zh");
        assert_eq!(config.lngs, vec!["zh", "en"]);
    }
}