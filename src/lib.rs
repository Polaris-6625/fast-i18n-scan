//! Fast i18n Scan - Rust Implementation
//! 
//! A fast and efficient internationalization scanning library for JavaScript/TypeScript projects.
//! This is a Rust port of the original JavaScript i18n scanning tools.

#[cfg(all(feature = "napi", not(target_family = "wasm")))]
use napi_derive::napi;

pub mod scan;
pub mod utils;

// Re-export commonly used items
pub use scan::*;
pub use utils::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration for i18n scanning
pub fn get_default_config() -> scan::config::ScanConfig {
    scan::config::get_default_config()
}

/// Create a new scanner instance
pub fn create_scanner() -> scan::Scanner {
    scan::Scanner::new()
}

/// Scan files for i18n keys
pub fn scan_files(files: &[String]) -> Result<scan::ScanResult, Box<dyn std::error::Error>> {
    let mut scanner = create_scanner();
    scanner.scan_files(files)
}

/// Quick scan function for single file
pub fn scan_file(filepath: &str) -> Result<scan::ScanResult, Box<dyn std::error::Error>> {
    scan_files(&[filepath.to_string()])
}

// NAPI exports for Node.js
#[cfg(feature = "napi")]
#[napi]
pub fn get_version() -> String {
    VERSION.to_string()
}

#[cfg(feature = "napi")]
#[napi(object)]
pub struct JsScanResult {
    pub keys: Vec<String>,
    pub translations: std::collections::HashMap<String, String>,
    pub errors: Vec<JsScanError>,
    pub warnings: Vec<JsScanWarning>,
    pub stats: JsScanStats,
}

#[cfg(feature = "napi")]
#[napi(object)]
pub struct JsScanError {
    pub filepath: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub error_type: String,
}

#[cfg(feature = "napi")]
#[napi(object)]
pub struct JsScanWarning {
    pub filepath: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub warning_type: String,
}

#[cfg(feature = "napi")]
#[napi(object)]
pub struct JsScanStats {
    pub files_scanned: u32,
    pub keys_found: u32,
    pub errors_count: u32,
    pub warnings_count: u32,
    pub processing_time_ms: u32,
}

#[cfg(feature = "napi")]
impl From<scan::ScanResult> for JsScanResult {
    fn from(result: scan::ScanResult) -> Self {
        Self {
            keys: result.keys,
            translations: result.translations,
            errors: result.errors.into_iter().map(|e| JsScanError {
                filepath: e.filepath,
                line: e.line,
                column: e.column,
                message: e.message,
                error_type: format!("{:?}", e.error_type),
            }).collect(),
            warnings: result.warnings.into_iter().map(|w| JsScanWarning {
                filepath: w.filepath,
                line: w.line,
                column: w.column,
                message: w.message,
                warning_type: format!("{:?}", w.warning_type),
            }).collect(),
            stats: JsScanStats {
                files_scanned: result.stats.files_scanned as u32,
                keys_found: result.stats.keys_found as u32,
                errors_count: result.stats.errors_count as u32,
                warnings_count: result.stats.warnings_count as u32,
                processing_time_ms: result.stats.processing_time_ms as u32,
            },
        }
    }
}

#[cfg(feature = "napi")]
#[napi]
pub fn scan_files_js(files: Vec<String>) -> napi::Result<JsScanResult> {
    match scan_files(&files) {
        Ok(result) => Ok(result.into()),
        Err(e) => Err(napi::Error::from_reason(e.to_string())),
    }
}

#[cfg(feature = "napi")]
#[napi]
pub fn scan_file_js(filepath: String) -> napi::Result<JsScanResult> {
    match scan_file(&filepath) {
        Ok(result) => Ok(result.into()),
        Err(e) => Err(napi::Error::from_reason(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = get_default_config();
        assert_eq!(config.default_lng, "zh");
    }

    #[test]
    fn test_create_scanner() {
        let scanner = create_scanner();
        assert!(scanner.is_ready());
    }
}