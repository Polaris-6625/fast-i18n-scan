use colored::*;
use regex::Regex;

/// 错误回调函数类型
pub type OnErrorCallback = Box<dyn Fn() + Send + Sync>;

/// AST 节点类型枚举
#[derive(Debug, Clone)]
pub enum AstNode {
    JSXText {
        value: String,
    },
    StringLiteral {
        value: String,
    },
    JSXExpressionContainer {
        expression: Box<AstNode>,
    },
    NumericLiteral {
        value: f64,
    },
    ObjectExpression {
        properties: Vec<ObjectProperty>,
    },
    TemplateLiteral {
        quasis: Vec<TemplateElement>,
    },
    JSXElement {
        children: Vec<AstNode>,
    },
    Other,
}

/// 对象属性
#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub key: ObjectKey,
    pub value: Box<AstNode>,
}

/// 对象键
#[derive(Debug, Clone)]
pub struct ObjectKey {
    pub name: String,
}

/// 模板字面量元素
#[derive(Debug, Clone)]
pub struct TemplateElement {
    pub value: TemplateElementValue,
}

/// 模板字面量值
#[derive(Debug, Clone)]
pub struct TemplateElementValue {
    pub raw: String,
}

/// 位置信息
#[derive(Debug, Clone)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

impl AstNode {
    /// 获取节点的位置信息
    pub fn get_location(&self) -> Location {
        // 默认位置，实际使用时应该从解析器获取真实位置
        Location { line: 1, column: 1 }
    }

    /// 检查是否为 JSXText 节点
    pub fn is_jsx_text(&self) -> bool {
        matches!(self, AstNode::JSXText { .. })
    }

    /// 检查是否为 StringLiteral 节点
    pub fn is_string_literal(&self) -> bool {
        matches!(self, AstNode::StringLiteral { .. })
    }

    /// 检查是否为 JSXExpressionContainer 节点
    pub fn is_jsx_expression_container(&self) -> bool {
        matches!(self, AstNode::JSXExpressionContainer { .. })
    }

    /// 检查是否为 NumericLiteral 节点
    pub fn is_numeric_literal(&self) -> bool {
        matches!(self, AstNode::NumericLiteral { .. })
    }

    /// 检查是否为 ObjectExpression 节点
    pub fn is_object_expression(&self) -> bool {
        matches!(self, AstNode::ObjectExpression { .. })
    }

    /// 检查是否为 TemplateLiteral 节点
    pub fn is_template_literal(&self) -> bool {
        matches!(self, AstNode::TemplateLiteral { .. })
    }

    /// 获取节点的值（如果是文本或字符串字面量）
    pub fn get_value(&self) -> Option<&String> {
        match self {
            AstNode::JSXText { value } | AstNode::StringLiteral { value } => Some(value),
            _ => None,
        }
    }

    /// 获取表达式（如果是 JSXExpressionContainer）
    pub fn get_expression(&self) -> Option<&AstNode> {
        match self {
            AstNode::JSXExpressionContainer { expression } => Some(expression),
            _ => None,
        }
    }

    /// 获取子节点（如果是 JSXElement）
    pub fn get_children(&self) -> Option<&Vec<AstNode>> {
        match self {
            AstNode::JSXElement { children } => Some(children),
            _ => None,
        }
    }

    /// 获取对象属性（如果是 ObjectExpression）
    pub fn get_properties(&self) -> Option<&Vec<ObjectProperty>> {
        match self {
            AstNode::ObjectExpression { properties } => Some(properties),
            _ => None,
        }
    }

    /// 获取模板字面量的 quasis（如果是 TemplateLiteral）
    pub fn get_quasis(&self) -> Option<&Vec<TemplateElement>> {
        match self {
            AstNode::TemplateLiteral { quasis } => Some(quasis),
            _ => None,
        }
    }
}

/// 将 AST 节点数组转换为字符串
pub fn nodes_to_string(
    nodes: &[AstNode],
    filepath: &str,
    on_error: Option<&OnErrorCallback>,
) -> String {
    let mut memo = String::new();
    let mut node_index = 0;

    for node in nodes {
        if node.is_jsx_text() || node.is_string_literal() {
            if let Some(value) = node.get_value() {
                let processed_value = process_text_value(value);
                if !processed_value.is_empty() {
                    memo.push_str(&processed_value);
                }
            }
        } else if node.is_jsx_expression_container() {
            if let Some(expression) = node.get_expression() {
                if expression.is_numeric_literal() {
                    // Numeric literal is ignored in react-i18next
                    memo.push_str("");
                } else if expression.is_string_literal() {
                    if let Some(value) = expression.get_value() {
                        memo.push_str(value);
                    }
                } else if expression.is_object_expression() {
                    if let Some(properties) = expression.get_properties() {
                        if !properties.is_empty() {
                            memo.push_str(&format!("{{{{{}}}}}", properties[0].key.name));
                        }
                    }
                } else if expression.is_template_literal() {
                    if let Some(quasis) = expression.get_quasis() {
                        for quasi in quasis {
                            memo.push_str(&quasi.value.raw);
                        }
                    }
                } else {
                    if let Some(error_callback) = on_error {
                        let location = node.get_location();
                        let error_fn = create_error_handler(filepath, location.line, location.column);
                        error_callback();
                        error_fn();
                    }
                }
            }
        } else if let Some(children) = node.get_children() {
            let child_string = nodes_to_string(children, filepath, on_error);
            memo.push_str(&format!("<{}>{}</{}>", node_index, child_string, node_index));
        }

        node_index += 1;
    }

    memo
}

/// 处理文本值，移除换行符和多余空格
fn process_text_value(value: &str) -> String {
    // remove leading spaces containing a leading newline character
    let re1 = Regex::new(r"^[\r\n]+\s*").unwrap();
    let step1 = re1.replace_all(value, "");

    // remove trailing spaces containing a leading newline character
    let re2 = Regex::new(r"[\r\n]+\s*$").unwrap();
    let step2 = re2.replace_all(&step1, "");

    // replace spaces containing a leading newline character with a single space character
    let re3 = Regex::new(r"[\r\n]+\s*").unwrap();
    let result = re3.replace_all(&step2, " ");

    result.trim().to_string()
}

/// 创建错误处理函数
fn create_error_handler(filepath: &str, line: u32, column: u32) -> Box<dyn Fn()> {
    let filepath = filepath.to_string();
    Box::new(move || {
        eprintln!();
        eprintln!("{}", format!("{}:{}:{}", filepath, line, column).yellow());
        eprintln!(
            "{}",
            "Unsupported JSX expression. Only static values or {{interpolation}} blocks are supported.".red()
        );
    })
}

/// 简化版本的 nodes_to_string，不需要错误回调
pub fn nodes_to_string_simple(nodes: &[AstNode], filepath: &str) -> String {
    nodes_to_string(nodes, filepath, None)
}

/// 创建默认错误回调
pub fn create_default_error_callback(filepath: &str, line: u32, column: u32) -> OnErrorCallback {
    let filepath = filepath.to_string();
    Box::new(move || {
        eprintln!();
        eprintln!("{}", format!("{}:{}:{}", filepath, line, column).yellow());
        eprintln!(
            "{}",
            "Unsupported JSX expression. Only static values or {{interpolation}} blocks are supported.".red()
        );
    })
}

// 便捷构造函数
impl AstNode {
    pub fn jsx_text(value: String) -> Self {
        AstNode::JSXText { value }
    }

    pub fn string_literal(value: String) -> Self {
        AstNode::StringLiteral { value }
    }

    pub fn jsx_expression_container(expression: AstNode) -> Self {
        AstNode::JSXExpressionContainer {
            expression: Box::new(expression),
        }
    }

    pub fn numeric_literal(value: f64) -> Self {
        AstNode::NumericLiteral { value }
    }

    pub fn object_expression(properties: Vec<ObjectProperty>) -> Self {
        AstNode::ObjectExpression { properties }
    }

    pub fn template_literal(quasis: Vec<TemplateElement>) -> Self {
        AstNode::TemplateLiteral { quasis }
    }

    pub fn jsx_element(children: Vec<AstNode>) -> Self {
        AstNode::JSXElement { children }
    }
}

impl ObjectProperty {
    pub fn new(key_name: String, value: AstNode) -> Self {
        ObjectProperty {
            key: ObjectKey { name: key_name },
            value: Box::new(value),
        }
    }
}

impl TemplateElement {
    pub fn new(raw: String) -> Self {
        TemplateElement {
            value: TemplateElementValue { raw },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsx_text_processing() {
        let nodes = vec![AstNode::jsx_text("  \n  Hello World  \n  ".to_string())];
        let result = nodes_to_string_simple(&nodes, "test.tsx");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_string_literal() {
        let nodes = vec![AstNode::string_literal("Hello".to_string())];
        let result = nodes_to_string_simple(&nodes, "test.tsx");
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_numeric_literal_ignored() {
        let expression = AstNode::numeric_literal(42.0);
        let nodes = vec![AstNode::jsx_expression_container(expression)];
        let result = nodes_to_string_simple(&nodes, "test.tsx");
        assert_eq!(result, "");
    }

    #[test]
    fn test_object_expression_interpolation() {
        let property = ObjectProperty::new("name".to_string(), AstNode::string_literal("value".to_string()));
        let expression = AstNode::object_expression(vec![property]);
        let nodes = vec![AstNode::jsx_expression_container(expression)];
        let result = nodes_to_string_simple(&nodes, "test.tsx");
        assert_eq!(result, "{{name}}");
    }

    #[test]
    fn test_template_literal() {
        let quasis = vec![
            TemplateElement::new("Hello ".to_string()),
            TemplateElement::new("World".to_string()),
        ];
        let expression = AstNode::template_literal(quasis);
        let nodes = vec![AstNode::jsx_expression_container(expression)];
        let result = nodes_to_string_simple(&nodes, "test.tsx");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_jsx_element_with_children() {
        let children = vec![AstNode::jsx_text("Child content".to_string())];
        let nodes = vec![AstNode::jsx_element(children)];
        let result = nodes_to_string_simple(&nodes, "test.tsx");
        assert_eq!(result, "<0>Child content</0>");
    }

    #[test]
    fn test_complex_structure() {
        let nodes = vec![
            AstNode::jsx_text("Hello ".to_string()),
            AstNode::jsx_expression_container(AstNode::object_expression(vec![
                ObjectProperty::new("name".to_string(), AstNode::string_literal("value".to_string()))
            ])),
            AstNode::jsx_text(" World".to_string()),
        ];
        let result = nodes_to_string_simple(&nodes, "test.tsx");
        assert_eq!(result, "Hello{{name}}World");
    }

    #[test]
    fn test_process_text_value() {
        assert_eq!(process_text_value("  \n  Hello World  \n  "), "Hello World");
        assert_eq!(process_text_value("Hello\n  World"), "Hello World");
        assert_eq!(process_text_value("\n\nHello\n\n"), "Hello");
    }
}