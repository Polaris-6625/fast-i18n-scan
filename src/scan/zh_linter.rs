use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

/// 位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

/// 位置范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub start: Position,
    pub end: Position,
}

/// Lint 结果项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub filepath: String,
    pub loc: Location,
    pub value: String,
}

/// ESLint 配置
#[derive(Debug, Clone)]
pub struct EslintConfig {
    pub env: HashMap<String, bool>,
    pub parser: String,
    pub parser_options: ParserOptions,
    pub rules: HashMap<String, String>,
}

/// 解析器选项
#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub ecma_version: u32,
    pub source_type: String,
    pub ecma_features: EcmaFeatures,
}

/// ECMAScript 特性
#[derive(Debug, Clone)]
pub struct EcmaFeatures {
    pub jsx: bool,
    pub legacy_decorators: bool,
}

/// 消息 ID 枚举
#[derive(Debug, Clone, PartialEq)]
pub enum MessageId {
    BareZhInJs,
    BareZhInJsx,
    BareZhInTemplate,
    ForbiddenHardCodeOfDomain,
    NoStringConcatenation,
}

impl MessageId {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "bareZhInJs" => Some(MessageId::BareZhInJs),
            "bareZhInJsx" => Some(MessageId::BareZhInJsx),
            "bareZhInTemplate" => Some(MessageId::BareZhInTemplate),
            "forbiddenHardCodeOfDomain" => Some(MessageId::ForbiddenHardCodeOfDomain),
            "noStringConcatenation" => Some(MessageId::NoStringConcatenation),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MessageId::BareZhInJs => "bareZhInJs",
            MessageId::BareZhInJsx => "bareZhInJsx",
            MessageId::BareZhInTemplate => "bareZhInTemplate",
            MessageId::ForbiddenHardCodeOfDomain => "forbiddenHardCodeOfDomain",
            MessageId::NoStringConcatenation => "noStringConcatenation",
        }
    }
}

/// ESLint 验证结果
#[derive(Debug, Clone)]
pub struct EslintMessage {
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub message_id: MessageId,
}

lazy_static! {
    static ref RESULT: Mutex<Vec<LintResult>> = Mutex::new(Vec::new());
    static ref HARD_CODE_SUGGESTIONS: Mutex<Vec<LintResult>> = Mutex::new(Vec::new());
    static ref NO_STRING_CONCATENATIONS: Mutex<Vec<LintResult>> = Mutex::new(Vec::new());
}

/// 中文 Linter
pub struct ZhLinter {
    zh_pattern: Regex,
    jsx_pattern: Regex,
    template_pattern: Regex,
    domain_pattern: Regex,
    concat_pattern: Regex,
}

impl Default for ZhLinter {
    fn default() -> Self {
        Self::new()
    }
}

impl ZhLinter {
    pub fn new() -> Self {
        Self {
            // 匹配中文字符的正则表达式
            zh_pattern: Regex::new(r"[\u4e00-\u9fff]+").unwrap(),
            // 匹配 JSX 中的中文
            jsx_pattern: Regex::new(r">\s*[\u4e00-\u9fff]+\s*<").unwrap(),
            // 匹配模板字符串中的中文
            template_pattern: Regex::new(r"`[^`]*[\u4e00-\u9fff]+[^`]*`").unwrap(),
            // 匹配硬编码域名
            domain_pattern: Regex::new(r"https?://[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            // 匹配字符串拼接
            concat_pattern: Regex::new(r#"['"`][^'"`]*['"`]\s*\+\s*['"`][^'"`]*['"`]"#).unwrap(),
        }
    }

    /// 获取 ESLint 配置
    pub fn get_config(&self, filepath: &str) -> EslintConfig {
        let is_ts = filepath.ends_with(".ts") || filepath.ends_with(".tsx");
        
        let mut env = HashMap::new();
        env.insert("browser".to_string(), true);
        env.insert("es6".to_string(), true);

        let mut rules = HashMap::new();
        rules.insert("@tencent/tea-i18n/no-bare-zh-in-js".to_string(), "error".to_string());
        rules.insert("@tencent/tea-i18n/no-bare-zh-in-jsx".to_string(), "error".to_string());
        rules.insert("@tencent/tea-i18n/no-hard-code-of-domain".to_string(), "error".to_string());
        rules.insert("@tencent/tea-i18n/no-string-concat".to_string(), "error".to_string());

        EslintConfig {
            env,
            parser: if is_ts { "ts-parser".to_string() } else { "js-eslint".to_string() },
            parser_options: ParserOptions {
                ecma_version: 6,
                source_type: "module".to_string(),
                ecma_features: EcmaFeatures {
                    jsx: true,
                    legacy_decorators: true,
                },
            },
            rules,
        }
    }

    /// 从代码行中获取值
    pub fn get_value(&self, lines: &[String], loc: &Location, message_id: &MessageId) -> String {
        let start = &loc.start;
        let end = &loc.end;
        let delta = end.line - start.line;

        let mut value = String::new();
        
        for (index, line) in lines.iter().enumerate() {
            let line_num = (index + 1) as u32;
            if line_num >= start.line && line_num <= end.line {
                let line_content = if index == 0 {
                    if delta == 0 {
                        // 单行
                        line.chars()
                            .skip((start.column - 1) as usize)
                            .take((end.column - start.column) as usize)
                            .collect::<String>()
                            .trim()
                            .to_string()
                    } else {
                        // 多行的第一行
                        line.chars()
                            .skip((start.column - 1) as usize)
                            .collect::<String>()
                            .trim()
                            .to_string()
                    }
                } else if line_num == end.line {
                    // 多行的最后一行
                    line.chars()
                        .take((end.column - 1) as usize)
                        .collect::<String>()
                        .trim()
                        .to_string()
                } else {
                    // 多行的中间行
                    line.trim().to_string()
                };
                
                value.push_str(&line_content);
            }
        }

        // 根据消息类型处理值
        match message_id {
            MessageId::BareZhInJsx => {
                // 移除 JSX 标签，只保留内容
                let jsx_regex = Regex::new(r#"^<("[^"]*"|'[^']*'|[^'">])*>(.+)(<\/[\w\-\.]*>)$"#).unwrap();
                if let Some(captures) = jsx_regex.captures(&value) {
                    captures.get(2).map_or(value.clone(), |m| m.as_str().to_string())
                } else {
                    value
                }
            }
            _ => {
                // 移除引号
                let quote_regex = Regex::new(r#"^['"`](.+)['"`]$"#).unwrap();
                if let Some(captures) = quote_regex.captures(&value) {
                    captures.get(1).map_or(value.clone(), |m| m.as_str().to_string())
                } else {
                    value
                }
            }
        }
    }

    /// 简化的 ESLint 验证（模拟 ESLint 行为）
    pub fn mock_eslint_verify(&self, content: &str, _config: &EslintConfig) -> Vec<EslintMessage> {
        let mut messages = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // 检查 JavaScript 中的中文
        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = (line_idx + 1) as u32;
            
            // 检查字符串中的中文
            if let Some(mat) = self.zh_pattern.find(line) {
                // 检查是否在字符串字面量中
                if self.is_in_string_literal(line, mat.start()) {
                    messages.push(EslintMessage {
                        line: line_num,
                        column: (mat.start() + 1) as u32,
                        end_line: line_num,
                        end_column: (mat.end() + 1) as u32,
                        message_id: MessageId::BareZhInJs,
                    });
                }
            }

            // 检查 JSX 中的中文
            if let Some(mat) = self.jsx_pattern.find(line) {
                messages.push(EslintMessage {
                    line: line_num,
                    column: (mat.start() + 1) as u32,
                    end_line: line_num,
                    end_column: (mat.end() + 1) as u32,
                    message_id: MessageId::BareZhInJsx,
                });
            }

            // 检查模板字符串中的中文
            if let Some(mat) = self.template_pattern.find(line) {
                messages.push(EslintMessage {
                    line: line_num,
                    column: (mat.start() + 1) as u32,
                    end_line: line_num,
                    end_column: (mat.end() + 1) as u32,
                    message_id: MessageId::BareZhInTemplate,
                });
            }

            // 检查硬编码域名
            if let Some(mat) = self.domain_pattern.find(line) {
                messages.push(EslintMessage {
                    line: line_num,
                    column: (mat.start() + 1) as u32,
                    end_line: line_num,
                    end_column: (mat.end() + 1) as u32,
                    message_id: MessageId::ForbiddenHardCodeOfDomain,
                });
            }

            // 检查字符串拼接
            if let Some(mat) = self.concat_pattern.find(line) {
                messages.push(EslintMessage {
                    line: line_num,
                    column: (mat.start() + 1) as u32,
                    end_line: line_num,
                    end_column: (mat.end() + 1) as u32,
                    message_id: MessageId::NoStringConcatenation,
                });
            }
        }

        messages
    }

    /// 检查位置是否在字符串字面量中
    fn is_in_string_literal(&self, line: &str, pos: usize) -> bool {
        // 确保 pos 在字符边界上
        let safe_pos = line.char_indices()
            .find(|(i, _)| *i >= pos)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        
        let before = &line[..safe_pos];
        let single_quotes = before.matches('\'').count();
        let double_quotes = before.matches('"').count();
        let backticks = before.matches('`').count();

        // 简单检查：如果前面有奇数个引号，说明在字符串中
        (single_quotes % 2 == 1) || (double_quotes % 2 == 1) || (backticks % 2 == 1)
    }

    /// 验证代码
    pub fn verify(&self, content: &str, filepath: &str) {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let config = self.get_config(filepath);
        let messages = self.mock_eslint_verify(content, &config);

        let filtered_messages: Vec<_> = messages
            .into_iter()
            .filter(|msg| {
                matches!(
                    msg.message_id,
                    MessageId::BareZhInJs
                        | MessageId::BareZhInJsx
                        | MessageId::BareZhInTemplate
                        | MessageId::ForbiddenHardCodeOfDomain
                        | MessageId::NoStringConcatenation
                )
            })
            .collect();

        let mut result = RESULT.lock().unwrap();
        let mut hard_code_suggestions = HARD_CODE_SUGGESTIONS.lock().unwrap();
        let mut no_string_concatenations = NO_STRING_CONCATENATIONS.lock().unwrap();

        for (index, msg) in filtered_messages.iter().enumerate() {
            let loc = Location {
                start: Position {
                    line: msg.line,
                    column: msg.column,
                },
                end: Position {
                    line: msg.end_line,
                    column: msg.end_column,
                },
            };

            // 检查重复
            if index > 0 && !result.is_empty() {
                let last = &result[result.len() - 1];
                if last.loc.start.line == msg.line
                    && last.loc.start.column == msg.column
                    && last.loc.end.line == msg.end_line
                    && last.loc.end.column == msg.end_column
                {
                    continue;
                }
            }

            let value = self.get_value(&lines, &loc, &msg.message_id);
            let lint_result = LintResult {
                filepath: filepath.to_string(),
                loc,
                value,
            };

            match msg.message_id {
                MessageId::ForbiddenHardCodeOfDomain => {
                    hard_code_suggestions.push(lint_result);
                }
                MessageId::NoStringConcatenation => {
                    no_string_concatenations.push(lint_result);
                }
                _ => {
                    result.push(lint_result);
                }
            }
        }
    }
}

/// 获取验证结果
pub fn get_result() -> Vec<LintResult> {
    RESULT.lock().unwrap().clone()
}

/// 获取硬编码建议
pub fn get_hard_code_suggestions() -> Vec<LintResult> {
    HARD_CODE_SUGGESTIONS.lock().unwrap().clone()
}

/// 获取字符串拼接问题
pub fn get_no_string_concatenations() -> Vec<LintResult> {
    NO_STRING_CONCATENATIONS.lock().unwrap().clone()
}

/// 清空所有结果
pub fn clear_results() {
    RESULT.lock().unwrap().clear();
    HARD_CODE_SUGGESTIONS.lock().unwrap().clear();
    NO_STRING_CONCATENATIONS.lock().unwrap().clear();
}

/// 创建 Linter 实例
pub fn create_linter() -> ZhLinter {
    ZhLinter::new()
}

/// 验证代码的便捷函数
pub fn verify_code(content: &str, filepath: &str) {
    let linter = ZhLinter::new();
    linter.verify(content, filepath);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zh_pattern() {
        let linter = ZhLinter::new();
        assert!(linter.zh_pattern.is_match("这是中文"));
        assert!(!linter.zh_pattern.is_match("This is English"));
    }

    #[test]
    fn test_get_config() {
        let linter = ZhLinter::new();
        let config_ts = linter.get_config("test.ts");
        let config_js = linter.get_config("test.js");
        
        assert_eq!(config_ts.parser, "ts-parser");
        assert_eq!(config_js.parser, "js-eslint");
    }

    #[test]
    fn test_get_value() {
        let linter = ZhLinter::new();
        let lines = vec!["const msg = '你好世界';".to_string()];
        let loc = Location {
            start: Position { line: 1, column: 13 },
            end: Position { line: 1, column: 18 },
        };
        
        let value = linter.get_value(&lines, &loc, &MessageId::BareZhInJs);
        assert_eq!(value, "'你好世界");
    }

    #[test]
    fn test_verify() {
        clear_results();
        let linter = ZhLinter::new();
        let content = r#"
const message = "你好世界";
const jsx = <div>中文内容</div>;
const url = "https://example.com";
const concat = "hello" + "world";
        "#;
        
        linter.verify(content, "test.tsx");
        
        let results = get_result();
        let hard_codes = get_hard_code_suggestions();
        let concatenations = get_no_string_concatenations();
        
        assert!(!results.is_empty());
        assert!(!hard_codes.is_empty());
        assert!(!concatenations.is_empty());
    }

    #[test]
    fn test_is_in_string_literal() {
        let linter = ZhLinter::new();
        assert!(linter.is_in_string_literal("const msg = '你好", 14));
        assert!(!linter.is_in_string_literal("const msg = '你好'; // 你好", 20));
    }

    #[test]
    fn test_message_id_conversion() {
        assert_eq!(MessageId::from_str("bareZhInJs"), Some(MessageId::BareZhInJs));
        assert_eq!(MessageId::BareZhInJs.as_str(), "bareZhInJs");
        assert_eq!(MessageId::from_str("unknown"), None);
    }
}