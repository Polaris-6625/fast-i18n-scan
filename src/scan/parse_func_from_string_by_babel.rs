use colored::*;
use regex::Regex;

use std::collections::HashMap;

/// 错误回调函数类型
pub type OnErrorCallback = Box<dyn Fn() + Send + Sync>;

/// 自定义处理函数类型
pub type CustomHandler = Box<dyn Fn(&str, &ParseOptions) + Send + Sync>;

/// 属性过滤函数类型
pub type PropsFilter = Box<dyn Fn(&str) -> String + Send + Sync>;

/// 解析选项
#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub default_value: Option<String>,
    pub default_value_plural: Option<String>,
    pub count: Option<String>,
    pub context: Option<String>,
    pub ns: Option<String>,
    pub key_separator: Option<String>,
    pub ns_separator: Option<String>,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            default_value: None,
            default_value_plural: None,
            count: None,
            context: None,
            ns: None,
            key_separator: None,
            ns_separator: None,
        }
    }
}

/// Babel 解析器选项
#[derive(Debug, Clone)]
pub struct BabylonOptions {
    pub plugins: Vec<String>,
    pub source_type: String,
}

impl Default for BabylonOptions {
    fn default() -> Self {
        Self {
            plugins: vec![
                "jsx".to_string(),
                "typescript".to_string(),
                "objectRestSpread".to_string(),
            ],
            source_type: "module".to_string(),
        }
    }
}

/// 函数解析配置
#[derive(Debug, Clone)]
pub struct FuncParseConfig {
    pub list: Vec<String>,
    pub babylon: BabylonOptions,
}

impl Default for FuncParseConfig {
    fn default() -> Self {
        Self {
            list: vec!["t".to_string(), "i18n.t".to_string(), "i18next.t".to_string()],
            babylon: BabylonOptions::default(),
        }
    }
}

/// 解析器选项
pub struct ParserOpts {
    pub list: Option<Vec<String>>,
    pub props_filter: Option<PropsFilter>,
    pub filepath: Option<String>,
    pub babylon_options: Option<BabylonOptions>,
}

impl Default for ParserOpts {
    fn default() -> Self {
        Self {
            list: None,
            props_filter: None,
            filepath: None,
            babylon_options: None,
        }
    }
}

/// 解析器结构体
pub struct Parser {
    pub options: ParserOptions,
    pub translations: HashMap<String, ParseOptions>,
}

/// 解析器选项
#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub func: FuncParseConfig,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            func: FuncParseConfig::default(),
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            options: ParserOptions::default(),
            translations: HashMap::new(),
        }
    }

    pub fn with_options(options: ParserOptions) -> Self {
        Self {
            options,
            translations: HashMap::new(),
        }
    }

    /// 设置翻译键值对
    pub fn set(&mut self, key: &str, options: ParseOptions) {
        self.translations.insert(key.to_string(), options);
    }

    /// 修复正则表达式匹配后的字符串
    pub fn fix_string_after_reg_exp(&self, s: &str, is_key: bool) -> Option<String> {
        if s.is_empty() {
            return None;
        }

        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        // 移除引号
        let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            || (trimmed.starts_with('`') && trimmed.ends_with('`'))
        {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        if unquoted.is_empty() && is_key {
            return None;
        }

        Some(unquoted.to_string())
    }

    /// 从字符串中解析函数调用
    pub fn parse_func_from_string_by_babel(
        &mut self,
        content: &str,
        opts: ParserOpts,
        custom_handler: Option<CustomHandler>,
        on_error: Option<OnErrorCallback>,
    ) -> &mut Self {
        let funcs = opts.list.unwrap_or_else(|| self.options.func.list.clone());

        if funcs.is_empty() {
            return self;
        }

        let match_funcs = funcs
            .iter()
            .map(|func| format!("(?:{})", regex::escape(func)))
            .collect::<Vec<_>>()
            .join("|");

        let match_special_characters = r"[\r\n\s]*";
        let string_group = format!(
            "{}({}|{}|{}){}",
            match_special_characters,
            r"`(?:[^`\\]|\\(?:.|$))*`",      // backtick
            r#""(?:[^"\\]|\\(?:.|$))*""#,   // double quotes
            r"'(?:[^'\\]|\\(?:.|$))*'",     // single quote
            match_special_characters
        );

        let pattern = format!(
            r"(?:(?:^\s*)|[^a-zA-Z0-9_])(?:{})\({}(?:[,]{})?[,)]",
            match_funcs, string_group, string_group
        );

        let re = Regex::new(&pattern).unwrap();

        for captures in re.captures_iter(content) {
            let mut options = ParseOptions::default();
            let full = captures.get(0).unwrap().as_str();

            let key_match = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let key = match self.fix_string_after_reg_exp(key_match, true) {
                Some(k) => k,
                None => continue,
            };

            if let Some(default_value_match) = captures.get(2) {
                if let Some(default_value) = self.fix_string_after_reg_exp(default_value_match.as_str(), false) {
                    options.default_value = Some(default_value);
                } else {
                    continue;
                }
            }

            let ends_with_comma = full.ends_with(',');
            if ends_with_comma {
                let babylon_options = opts.babylon_options.as_ref().map(|o| o.clone()).unwrap_or_else(|| self.options.func.babylon.clone());
                let filepath = opts.filepath.as_deref().unwrap_or("");

                // 找到匹配位置后的代码
                let match_end = captures.get(0).unwrap().end();
                let remaining_content = &content[match_end..];
                let mut code = match_balanced_parentheses(remaining_content);

                // 应用属性过滤器
                if let Some(ref filter) = opts.props_filter {
                    code = filter(&code);
                }

                if !code.trim().is_empty() {
                    match parse_babel_code(&format!("({})", code), &babylon_options) {
                        Ok(props) => {
                            self.process_parsed_properties(&mut options, &props);
                        }
                        Err(err) => {
                            if let Some(ref error_callback) = on_error {
                                let error_handler = create_parse_error_handler(filepath, &code, &err);
                                error_callback();
                                error_handler();
                            }
                        }
                    }
                }
            }

            if let Some(ref handler) = custom_handler {
                handler(&key, &options);
                continue;
            }

            self.set(&key, options);
        }

        self
    }

    /// 处理解析后的属性
    fn process_parsed_properties(&self, options: &mut ParseOptions, props: &[Property]) {
        let supported_options = [
            "defaultValue",
            "defaultValue_plural",
            "count",
            "context",
            "ns",
            "keySeparator",
            "nsSeparator",
        ];

        for prop in props {
            if supported_options.contains(&prop.key.as_str()) {
                let value = match &prop.value {
                    PropertyValue::Literal(s) => s.clone(),
                    PropertyValue::TemplateLiteral(parts) => parts.join(""),
                    PropertyValue::Unknown => String::new(),
                };

                match prop.key.as_str() {
                    "defaultValue" => options.default_value = Some(value),
                    "defaultValue_plural" => options.default_value_plural = Some(value),
                    "count" => options.count = Some(value),
                    "context" => options.context = Some(value),
                    "ns" => options.ns = Some(value),
                    "keySeparator" => options.key_separator = Some(value),
                    "nsSeparator" => options.ns_separator = Some(value),
                    _ => {}
                }
            }
        }
    }
}

/// 属性结构体
#[derive(Debug, Clone)]
pub struct Property {
    pub key: String,
    pub value: PropertyValue,
}

/// 属性值枚举
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Literal(String),
    TemplateLiteral(Vec<String>),
    Unknown,
}

/// 匹配平衡的括号
pub fn match_balanced_parentheses(s: &str) -> String {
    let parentheses = "[]{}()";
    let mut stack = Vec::new();
    let mut start = None;

    for (i, ch) in s.char_indices() {
        if start.is_some() && stack.is_empty() {
            return s[start.unwrap()..i].to_string();
        }

        if let Some(brace_pos) = parentheses.find(ch) {
            if brace_pos % 2 == 0 {
                // 开括号
                if start.is_none() {
                    start = Some(i);
                }
                stack.push(brace_pos + 1);
            } else {
                // 闭括号
                if stack.pop() != Some(brace_pos) {
                    return s[start.unwrap_or(0)..i].to_string();
                }
            }
        }
    }

    s[start.unwrap_or(0)..].to_string()
}

/// 解析 Babel 代码（简化版本）
fn parse_babel_code(code: &str, _options: &BabylonOptions) -> Result<Vec<Property>, String> {
    // 这里是一个简化的解析器，实际使用时应该集成真正的 Babel 解析器
    // 目前只做基本的对象属性解析
    let mut properties = Vec::new();

    // 简单的正则匹配对象属性
    let prop_regex = Regex::new(r#"(\w+)\s*:\s*(['"`])(.*?)\2"#).unwrap();
    
    for captures in prop_regex.captures_iter(code) {
        let key = captures.get(1).unwrap().as_str().to_string();
        let value = captures.get(3).unwrap().as_str().to_string();
        
        properties.push(Property {
            key,
            value: PropertyValue::Literal(value),
        });
    }

    Ok(properties)
}

/// 创建解析错误处理函数
fn create_parse_error_handler(filepath: &str, code: &str, error: &str) -> Box<dyn Fn()> {
    let filepath = filepath.to_string();
    let code = code.to_string();
    let error = error.to_string();
    
    Box::new(move || {
        eprintln!();
        eprintln!("{}", format!("{}:1:1", filepath).yellow());
        eprintln!("{}", format!("\nUnable to parse code \"{}\"\n", code.green()).cyan());
        eprintln!("{}", format!("    {}", error).red());
    })
}

/// 创建解析器实例
pub fn create_parser() -> Parser {
    Parser::new()
}

/// 创建带选项的解析器实例
pub fn create_parser_with_options(options: ParserOptions) -> Parser {
    Parser::with_options(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_balanced_parentheses() {
        assert_eq!(match_balanced_parentheses("(hello)"), "(hello)");
        assert_eq!(match_balanced_parentheses("{key: 'value'}"), "{key: 'value'}");
        assert_eq!(match_balanced_parentheses("[1, 2, 3]"), "[1, 2, 3]");
        assert_eq!(match_balanced_parentheses("(nested (inner) content)"), "(nested (inner) content)");
    }

    #[test]
    fn test_fix_string_after_reg_exp() {
        let parser = Parser::new();
        
        assert_eq!(parser.fix_string_after_reg_exp("'hello'", true), Some("hello".to_string()));
        assert_eq!(parser.fix_string_after_reg_exp("\"world\"", true), Some("world".to_string()));
        assert_eq!(parser.fix_string_after_reg_exp("`template`", true), Some("template".to_string()));
        assert_eq!(parser.fix_string_after_reg_exp("", true), None);
        assert_eq!(parser.fix_string_after_reg_exp("''", true), None);
    }

    #[test]
    fn test_parser_set_and_get() {
        let mut parser = Parser::new();
        let options = ParseOptions {
            default_value: Some("Hello".to_string()),
            ..Default::default()
        };
        
        parser.set("greeting", options.clone());
        assert!(parser.translations.contains_key("greeting"));
        assert_eq!(parser.translations.get("greeting").unwrap().default_value, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_simple_function_call() {
        let mut parser = Parser::new();
        let content = r#"t('hello.world')"#;
        
        parser.parse_func_from_string_by_babel(
            content,
            ParserOpts::default(),
            None,
            None,
        );
        
        assert!(parser.translations.contains_key("hello.world"));
    }

    #[test]
    fn test_parse_function_call_with_default_value() {
        let mut parser = Parser::new();
        let content = r#"t('greeting', 'Hello World')"#;
        
        parser.parse_func_from_string_by_babel(
            content,
            ParserOpts::default(),
            None,
            None,
        );
        
        assert!(parser.translations.contains_key("greeting"));
        let options = parser.translations.get("greeting").unwrap();
        assert_eq!(options.default_value, Some("Hello World".to_string()));
    }
}