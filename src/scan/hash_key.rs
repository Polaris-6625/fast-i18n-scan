use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use colored::*;

// 全局 key 映射表，使用 Mutex 保证线程安全
lazy_static! {
    static ref KEY_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// 错误回调函数类型
pub type OnErrorCallback = Box<dyn Fn(&str) + Send + Sync>;

/// 从句子计算哈希一个 key 值，该算法需要和 scanner 保持一致
pub fn hash_key(
    value: &str,
    context: Option<&str>,
    on_error: Option<&OnErrorCallback>,
) -> String {
    // 移除所有空白字符并计算哈希
    let cleaned_value = value.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    let hash = hash_string(&cleaned_value);
    
    // 转换为 36 进制并格式化为 7 位
    let hash_str = format!("{:0>7}", radix_36(hash));
    let key = format!("k_{}", hash_str);
    
    // 构建完整的 key（包含 context）
    let _full_key = if let Some(ctx) = context {
        format!("{}_{}", key, ctx)
    } else {
        key.clone()
    };
    
    // 检查是否存在冲突
    let mut key_map = KEY_MAP.lock().unwrap();
    if let Some(existed_key) = key_map.get(value) {
        // 相同的原始值已经存在，检查是否生成了相同的 key
        if existed_key != &key {
            // 这种情况下是哈希冲突，相同的文本生成了不同的 key（理论上不应该发生）
            if let Some(error_callback) = on_error {
                error_callback(&format_error_message(existed_key, &key));
            }
        }
        // 如果是相同的值和相同的 key，不需要重复插入
    } else {
        key_map.insert(value.to_string(), key.clone());
    }
    
    key
}

/// 简化版本的 hash_key，不需要错误回调
pub fn hash_key_simple(value: &str, context: Option<&str>) -> String {
    hash_key(value, context, None)
}

/// 清空 key 映射表（主要用于测试）
pub fn clear_key_map() {
    let mut key_map = KEY_MAP.lock().unwrap();
    key_map.clear();
}

/// 获取当前 key 映射表的大小
pub fn get_key_map_size() -> usize {
    let key_map = KEY_MAP.lock().unwrap();
    key_map.len()
}

/// 简单的字符串哈希函数，模拟 hash-string 库的行为
fn hash_string(s: &str) -> u32 {
    let mut hash: u32 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    hash
}

/// 将数字转换为 36 进制字符串
fn radix_36(mut num: u32) -> String {
    if num == 0 {
        return "0".to_string();
    }
    
    let mut result = String::new();
    let chars = "0123456789abcdefghijklmnopqrstuvwxyz";
    
    while num > 0 {
        let remainder = (num % 36) as usize;
        result.insert(0, chars.chars().nth(remainder).unwrap());
        num /= 36;
    }
    
    result
}

/// 格式化错误消息
fn format_error_message(existed_value: &str, new_value: &str) -> String {
    format!(
        "\n{}\n{}\n    \"{}\"\n    \"{}\"",
        "Same sentence in different forms found:".red(),
        "",
        existed_value,
        new_value
    )
}

/// 带文件路径的错误回调函数
pub fn create_file_error_callback() -> OnErrorCallback {
    Box::new(|filepath: &str| {
        eprintln!();
        eprintln!("{}", filepath.yellow());
        // 错误信息已经在 format_error_message 中格式化
        eprintln!("{}", filepath);
    })
}

/// 默认错误回调函数
pub fn default_error_callback(filepath: &str) {
    eprintln!();
    eprintln!("{}", filepath.yellow());
    eprintln!("{}", "Same sentence in different forms found:".red());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_key_basic() {
        clear_key_map();
        
        let key1 = hash_key_simple("Hello World", None);
        let key2 = hash_key_simple("Hello World", None);
        
        assert_eq!(key1, key2);
        assert!(key1.starts_with("k_"));
        assert_eq!(key1.len(), 9); // "k_" + 7 characters
        
        clear_key_map(); // 清理状态
    }

    #[test]
    fn test_hash_key_with_context() {
        clear_key_map();
        
        let key1 = hash_key_simple("Hello", Some("context1"));
        let key2 = hash_key_simple("Hello", Some("context2"));
        
        // 相同的文本在不同 context 下应该生成相同的 key
        assert_eq!(key1, key2);
        
        clear_key_map(); // 清理状态
    }

    #[test]
    fn test_hash_key_whitespace_removal() {
        clear_key_map();
        
        let key1 = hash_key_simple("Hello World", None);
        let key2 = hash_key_simple("Hello   World", None);
        let key3 = hash_key_simple("HelloWorld", None);
        
        // 空白字符应该被移除，所以这些应该生成相同的 key
        assert_eq!(key1, key2);
        assert_eq!(key2, key3);
        
        clear_key_map(); // 清理状态
    }

    #[test]
    fn test_radix_36() {
        assert_eq!(radix_36(0), "0");
        assert_eq!(radix_36(35), "z");
        assert_eq!(radix_36(36), "10");
    }

    #[test]
    fn test_hash_string() {
        let hash1 = hash_string("test");
        let hash2 = hash_string("test");
        let hash3 = hash_string("different");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_key_map_operations() {
        // 测试基本的 key 生成功能
        let key1 = hash_key_simple("test_value_1", None);
        let key2 = hash_key_simple("test_value_2", None);
        let key3 = hash_key_simple("test_value_1", None); // 重复值
        
        // 验证 key 格式
        assert!(key1.starts_with("k_"));
        assert!(key2.starts_with("k_"));
        assert!(key3.starts_with("k_"));
        
        // 验证相同输入生成相同的 key
        assert_eq!(key1, key3);
        
        // 验证不同输入生成不同的 key
        assert_ne!(key1, key2);
        
        // 验证 key 长度
        assert_eq!(key1.len(), 9); // "k_" + 7 characters
        assert_eq!(key2.len(), 9);
        assert_eq!(key3.len(), 9);
    }
}