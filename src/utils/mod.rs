//! Utility functions and helpers

pub mod remove_i18n_key;

pub use remove_i18n_key::*;

use std::fs;
use std::path::Path;

/// Read file content safely
pub fn read_file_safe(filepath: &str) -> Result<String, Box<dyn std::error::Error>> {
    match fs::read_to_string(filepath) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file {}: {}", filepath, e).into()),
    }
}

/// Check if file exists
pub fn file_exists(filepath: &str) -> bool {
    Path::new(filepath).exists()
}

/// Get file extension
pub fn get_file_extension(filepath: &str) -> Option<String> {
    Path::new(filepath)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

/// Check if file is a JavaScript/TypeScript file
pub fn is_js_ts_file(filepath: &str) -> bool {
    match get_file_extension(filepath) {
        Some(ext) => matches!(ext.as_str(), "js" | "jsx" | "ts" | "tsx"),
        None => false,
    }
}

/// Normalize file path
pub fn normalize_path(filepath: &str) -> String {
    filepath.replace('\\', "/")
}

/// Extract directory from file path
pub fn get_directory(filepath: &str) -> String {
    match Path::new(filepath).parent() {
        Some(parent) => parent.to_string_lossy().to_string(),
        None => ".".to_string(),
    }
}

/// Extract filename from file path
pub fn get_filename(filepath: &str) -> String {
    match Path::new(filepath).file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => filepath.to_string(),
    }
}

/// Create directory if it doesn't exist
pub fn ensure_directory(dirpath: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(dirpath).exists() {
        fs::create_dir_all(dirpath)?;
    }
    Ok(())
}

/// Write content to file safely
pub fn write_file_safe(filepath: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(filepath).parent() {
        ensure_directory(&parent.to_string_lossy())?;
    }
    
    fs::write(filepath, content)?;
    Ok(())
}

/// Format file size in human readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Get file size
pub fn get_file_size(filepath: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(filepath)?;
    Ok(metadata.len())
}

/// Check if string contains Chinese characters
pub fn contains_chinese(text: &str) -> bool {
    text.chars().any(|c| '\u{4e00}' <= c && c <= '\u{9fff}')
}

/// Remove whitespace from string
pub fn remove_whitespace(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Escape string for regex
pub fn escape_regex(text: &str) -> String {
    regex::escape(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("test.js"), Some("js".to_string()));
        assert_eq!(get_file_extension("test.tsx"), Some("tsx".to_string()));
        assert_eq!(get_file_extension("test"), None);
    }

    #[test]
    fn test_is_js_ts_file() {
        assert!(is_js_ts_file("app.js"));
        assert!(is_js_ts_file("component.tsx"));
        assert!(!is_js_ts_file("style.css"));
        assert!(!is_js_ts_file("README.md"));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("src\\components\\App.tsx"), "src/components/App.tsx");
        assert_eq!(normalize_path("src/utils/index.js"), "src/utils/index.js");
    }

    #[test]
    fn test_get_directory() {
        assert_eq!(get_directory("src/components/App.tsx"), "src/components");
        assert_eq!(get_directory("index.js"), "");
    }

    #[test]
    fn test_get_filename() {
        assert_eq!(get_filename("src/components/App.tsx"), "App.tsx");
        assert_eq!(get_filename("index.js"), "index.js");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_contains_chinese() {
        assert!(contains_chinese("你好世界"));
        assert!(contains_chinese("Hello 世界"));
        assert!(!contains_chinese("Hello World"));
    }

    #[test]
    fn test_remove_whitespace() {
        assert_eq!(remove_whitespace("Hello World"), "HelloWorld");
        assert_eq!(remove_whitespace("  Hello   World  "), "HelloWorld");
        assert_eq!(remove_whitespace("Hello\nWorld\t"), "HelloWorld");
    }
}