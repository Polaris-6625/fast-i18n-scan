//! Scanning module for i18n keys and translations

pub mod config;
pub mod js_config;
pub mod hash_key;
pub mod nodes_to_string;
pub mod parse_func_from_string_by_babel;
pub mod zh_linter;
pub mod slp;

// Re-export submodules
pub use config::*;
pub use hash_key::*;
pub use nodes_to_string::*;
pub use parse_func_from_string_by_babel::*;
pub use zh_linter::*;
pub use slp::*;

use std::collections::HashMap;
use std::fs;

pub use config::ScanConfig;
pub use hash_key::{hash_key, hash_key_simple};
pub use zh_linter::{verify_code, get_result, get_hard_code_suggestions, get_no_string_concatenations};

/// Scan result structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanResult {
    pub keys: Vec<String>,
    pub translations: HashMap<String, String>,
    pub errors: Vec<ScanError>,
    pub warnings: Vec<ScanWarning>,
    pub stats: ScanStats,
}

/// Scan error
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanError {
    pub filepath: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub error_type: ErrorType,
}

/// Scan warning
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanWarning {
    pub filepath: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub warning_type: WarningType,
}

/// Error types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ErrorType {
    ParseError,
    InvalidKey,
    DuplicateKey,
    MissingTranslation,
    HardCodedText,
}

/// Warning types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WarningType {
    UnusedKey,
    StringConcatenation,
    HardCodedDomain,
}

/// Scan statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanStats {
    pub files_scanned: usize,
    pub keys_found: usize,
    pub errors_count: usize,
    pub warnings_count: usize,
    pub processing_time_ms: u64,
}

/// Main scanner structure
pub struct Scanner {
    config: ScanConfig,
    linter: zh_linter::ZhLinter,
}

impl Scanner {
    /// Create a new scanner with default configuration
    pub fn new() -> Self {
        Self {
            config: ScanConfig::default(),
            linter: zh_linter::ZhLinter::new(),
        }
    }

    /// Create a scanner with custom configuration
    pub fn with_config(config: ScanConfig) -> Self {
        Self {
            config,
            linter: zh_linter::ZhLinter::new(),
        }
    }

    /// Check if scanner is ready
    pub fn is_ready(&self) -> bool {
        true
    }

    /// Scan multiple files
    pub fn scan_files(&mut self, files: &[String]) -> Result<ScanResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut keys = Vec::new();
        let mut translations = HashMap::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Clear previous results
        zh_linter::clear_results();

        for filepath in files {
            match self.scan_single_file(filepath) {
                Ok(file_result) => {
                    keys.extend(file_result.keys);
                    translations.extend(file_result.translations);
                    errors.extend(file_result.errors);
                    warnings.extend(file_result.warnings);
                }
                Err(e) => {
                    errors.push(ScanError {
                        filepath: filepath.clone(),
                        line: 1,
                        column: 1,
                        message: e.to_string(),
                        error_type: ErrorType::ParseError,
                    });
                }
            }
        }

        // Collect linter results
        let lint_results = get_result();
        for result in lint_results {
            errors.push(ScanError {
                filepath: result.filepath,
                line: result.loc.start.line,
                column: result.loc.start.column,
                message: format!("Hard-coded Chinese text found: {}", result.value),
                error_type: ErrorType::HardCodedText,
            });
        }

        let hard_code_suggestions = get_hard_code_suggestions();
        for suggestion in hard_code_suggestions {
            warnings.push(ScanWarning {
                filepath: suggestion.filepath,
                line: suggestion.loc.start.line,
                column: suggestion.loc.start.column,
                message: format!("Hard-coded domain found: {}", suggestion.value),
                warning_type: WarningType::HardCodedDomain,
            });
        }

        let concatenations = get_no_string_concatenations();
        for concat in concatenations {
            warnings.push(ScanWarning {
                filepath: concat.filepath,
                line: concat.loc.start.line,
                column: concat.loc.start.column,
                message: format!("String concatenation found: {}", concat.value),
                warning_type: WarningType::StringConcatenation,
            });
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        let unique_keys: Vec<String> = keys.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
        let keys_count = unique_keys.len();
        let errors_count = errors.len();
        let warnings_count = warnings.len();

        Ok(ScanResult {
            keys: unique_keys,
            translations,
            errors,
            warnings,
            stats: ScanStats {
                files_scanned: files.len(),
                keys_found: keys_count,
                errors_count,
                warnings_count,
                processing_time_ms: processing_time,
            },
        })
    }

    /// Scan a single file
    fn scan_single_file(&mut self, filepath: &str) -> Result<ScanResult, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(filepath)?;
        
        // Run linter
        self.linter.verify(&content, filepath);

        // Parse functions from content
        let mut parser = parse_func_from_string_by_babel::Parser::new();
        parser.parse_func_from_string_by_babel(
            &content,
            parse_func_from_string_by_babel::ParserOpts::default(),
            None,
            None,
        );

        // Extract keys and translations
        let mut keys = Vec::new();
        let mut translations = HashMap::new();

        for (key, options) in &parser.translations {
            keys.push(key.clone());
            if let Some(default_value) = &options.default_value {
                translations.insert(key.clone(), default_value.clone());
            }
        }

        let keys_count = keys.len();
        Ok(ScanResult {
            keys,
            translations,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ScanStats {
                files_scanned: 1,
                keys_found: keys_count,
                errors_count: 0,
                warnings_count: 0,
                processing_time_ms: 0,
            },
        })
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ScanConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ScanConfig) {
        self.config = config;
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let scanner = Scanner::new();
        assert!(scanner.is_ready());
    }

    #[test]
    fn test_scanner_with_config() {
        let config = ScanConfig::default();
        let scanner = Scanner::with_config(config);
        assert!(scanner.is_ready());
    }

    #[test]
    fn test_scan_empty_files() {
        let mut scanner = Scanner::new();
        let result = scanner.scan_files(&[]).unwrap();
        assert_eq!(result.stats.files_scanned, 0);
        assert_eq!(result.keys.len(), 0);
    }
}