use fast_i18n_scan::{scan_files, Scanner, ScanConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Quick scan
    println!("=== Quick Scan Example ===");
    let files = vec!["examples/sample.tsx".to_string()];
    
    match scan_files(&files) {
        Ok(result) => {
            println!("Scan completed!");
            println!("Files scanned: {}", result.stats.files_scanned);
            println!("Keys found: {}", result.stats.keys_found);
            println!("Errors: {}", result.stats.errors_count);
            println!("Warnings: {}", result.stats.warnings_count);
            
            if !result.keys.is_empty() {
                println!("\nFound keys:");
                for key in &result.keys {
                    println!("  - {}", key);
                }
            }
            
            if !result.errors.is_empty() {
                println!("\nErrors:");
                for error in &result.errors {
                    println!("  {}:{}:{} - {}", error.filepath, error.line, error.column, error.message);
                }
            }
        }
        Err(e) => {
            eprintln!("Scan failed: {}", e);
        }
    }

    // Example 2: Custom configuration
    println!("\n=== Custom Configuration Example ===");
    let config = ScanConfig::default()
        .with_languages(vec!["zh".to_string(), "en".to_string()])
        .with_default_language("zh".to_string());

    let mut scanner = Scanner::with_config(config);
    
    match scanner.scan_files(&files) {
        Ok(result) => {
            println!("Custom scan completed!");
            println!("Processing time: {}ms", result.stats.processing_time_ms);
        }
        Err(e) => {
            eprintln!("Custom scan failed: {}", e);
        }
    }

    Ok(())
}