use fast_i18n_scan::{scan_files, Scanner, ScanConfig};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_scan_sample_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.tsx");
    
    let content = r#"
import React from 'react';

const App = () => {
  const message = "你好世界";
  const greeting = t('greeting', '默认值');
  
  return (
    <div>
      <h1>硬编码中文</h1>
      <p>{t('welcome')}</p>
    </div>
  );
};
"#;
    
    fs::write(&file_path, content).unwrap();
    
    let result = scan_files(&[file_path.to_string_lossy().to_string()]).unwrap();
    
    // Should find some keys
    assert!(!result.keys.is_empty());
    
    // Should detect hard-coded Chinese text
    assert!(result.stats.errors_count > 0);
    
    // Should have scanned 1 file
    assert_eq!(result.stats.files_scanned, 1);
}

#[test]
fn test_scanner_with_config() {
    let config = ScanConfig::default()
        .with_languages(vec!["zh".to_string(), "en".to_string()])
        .with_default_language("zh".to_string());
    
    let scanner = Scanner::with_config(config);
    assert!(scanner.is_ready());
    assert_eq!(scanner.get_config().default_lng, "zh");
}

#[test]
fn test_empty_file_list() {
    let result = scan_files(&[]).unwrap();
    assert_eq!(result.stats.files_scanned, 0);
    assert_eq!(result.keys.len(), 0);
    assert_eq!(result.stats.errors_count, 0);
}

#[test]
fn test_nonexistent_file() {
    let result = scan_files(&["nonexistent.tsx".to_string()]);
    assert!(result.is_err() || result.unwrap().stats.errors_count > 0);
}