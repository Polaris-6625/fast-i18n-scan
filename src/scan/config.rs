use serde::{Deserialize, Serialize};

/// Babel 解析器选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BabelOptions {
    pub plugins: Vec<BabelPlugin>,
    pub source_type: String,
}

/// Babel 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BabelPlugin {
    Simple(String),
    WithOptions(String, serde_json::Value),
}

/// 资源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub load_path: String,
    pub save_path: String,
}

/// 函数配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncConfig {
    pub list: Vec<String>,
    pub extensions: Vec<String>,
    pub babylon: BabelOptions,
}

/// 转换配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransConfig {
    pub extensions: Vec<String>,
    pub babylon: BabelOptions,
}

/// i18next 扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub input: Vec<String>,
    pub lngs: Vec<String>,
    pub ns: Vec<String>,
    pub default_lng: String,
    pub default_ns: String,
    pub resource: ResourceConfig,
    pub func: FuncConfig,
    pub trans: TransConfig,
}

impl Default for BabelOptions {
    fn default() -> Self {
        Self {
            plugins: vec![
                BabelPlugin::Simple("jsx".to_string()),
                BabelPlugin::Simple("typescript".to_string()),
                BabelPlugin::Simple("doExpressions".to_string()),
                BabelPlugin::Simple("objectRestSpread".to_string()),
                BabelPlugin::WithOptions(
                    "decorators".to_string(),
                    serde_json::json!({ "decoratorsBeforeExport": true })
                ),
                BabelPlugin::Simple("classProperties".to_string()),
                BabelPlugin::Simple("classPrivateProperties".to_string()),
                BabelPlugin::Simple("classPrivateMethods".to_string()),
                BabelPlugin::Simple("exportDefaultFrom".to_string()),
                BabelPlugin::Simple("exportNamespaceFrom".to_string()),
                BabelPlugin::Simple("asyncGenerators".to_string()),
                BabelPlugin::Simple("functionBind".to_string()),
                BabelPlugin::Simple("functionSent".to_string()),
                BabelPlugin::Simple("dynamicImport".to_string()),
                BabelPlugin::Simple("numericSeparator".to_string()),
                BabelPlugin::Simple("optionalChaining".to_string()),
                BabelPlugin::Simple("importMeta".to_string()),
                BabelPlugin::Simple("bigInt".to_string()),
                BabelPlugin::Simple("optionalCatchBinding".to_string()),
                BabelPlugin::Simple("throwExpressions".to_string()),
                BabelPlugin::WithOptions(
                    "pipelineOperator".to_string(),
                    serde_json::json!({ "proposal": "minimal" })
                ),
                BabelPlugin::Simple("nullishCoalescingOperator".to_string()),
            ],
            source_type: "module".to_string(),
        }
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        let default_lng = "zh".to_string();
        let babel_options = BabelOptions::default();

        Self {
            input: vec!["src/**/*.{js,jsx,ts,tsx}".to_string()],
            lngs: vec![default_lng.clone()],
            ns: vec!["translation".to_string()],
            default_lng: default_lng.clone(),
            default_ns: "translation".to_string(),
            resource: ResourceConfig {
                load_path: "".to_string(), // 避免 i18next-scanner 读取报错
                save_path: "i18n/translation/{{lng}}.js".to_string(),
            },
            func: FuncConfig {
                list: vec![
                    "i18next.t".to_string(),
                    "i18n.t".to_string(),
                    "t".to_string(),
                ],
                extensions: vec![], // 避免在 transform 中执行原生的 parseFuncFromString
                babylon: babel_options.clone(),
            },
            trans: TransConfig {
                extensions: vec![], // 避免在 transform 中执行原生的 parseTransFromString
                babylon: babel_options,
            },
        }
    }
}

impl ScanConfig {
    /// 创建新的扫描配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置输入文件模式
    pub fn with_input(mut self, input: Vec<String>) -> Self {
        self.input = input;
        self
    }

    /// 设置语言列表
    pub fn with_languages(mut self, lngs: Vec<String>) -> Self {
        self.lngs = lngs;
        self
    }

    /// 设置命名空间
    pub fn with_namespaces(mut self, ns: Vec<String>) -> Self {
        self.ns = ns;
        self
    }

    /// 设置默认语言
    pub fn with_default_language(mut self, default_lng: String) -> Self {
        self.default_lng = default_lng;
        self
    }

    /// 设置默认命名空间
    pub fn with_default_namespace(mut self, default_ns: String) -> Self {
        self.default_ns = default_ns;
        self
    }

    /// 设置资源配置
    pub fn with_resource(mut self, resource: ResourceConfig) -> Self {
        self.resource = resource;
        self
    }

    /// 设置函数配置
    pub fn with_func(mut self, func: FuncConfig) -> Self {
        self.func = func;
        self
    }

    /// 设置转换配置
    pub fn with_trans(mut self, trans: TransConfig) -> Self {
        self.trans = trans;
        self
    }
}

/// 获取默认的 i18next 扫描配置
pub fn get_default_config() -> ScanConfig {
    ScanConfig::default()
}

/// 创建自定义配置的构建器
pub fn config_builder() -> ScanConfig {
    ScanConfig::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = get_default_config();
        assert_eq!(config.default_lng, "zh");
        assert_eq!(config.default_ns, "translation");
        assert_eq!(config.input, vec!["src/**/*.{js,jsx,ts,tsx}"]);
        assert_eq!(config.lngs, vec!["zh"]);
        assert_eq!(config.ns, vec!["translation"]);
    }

    #[test]
    fn test_config_builder() {
        let config = config_builder()
            .with_input(vec!["app/**/*.tsx".to_string()])
            .with_languages(vec!["zh".to_string(), "en".to_string()])
            .with_default_language("en".to_string());

        assert_eq!(config.input, vec!["app/**/*.tsx"]);
        assert_eq!(config.lngs, vec!["zh", "en"]);
        assert_eq!(config.default_lng, "en");
    }

    #[test]
    fn test_babel_options() {
        let babel_options = BabelOptions::default();
        assert_eq!(babel_options.source_type, "module");
        assert!(!babel_options.plugins.is_empty());
    }
}