use clap::{Arg, Command};
use fast_i18n_scan::{scan_files, get_default_config};
use std::process;
use glob::glob;

// Function to expand brace patterns like *.{js,jsx,ts,tsx}
fn expand_braces(pattern: &str) -> Vec<String> {
    if let Some(start) = pattern.find('{') {
        if let Some(end) = pattern.find('}') {
            if start < end {
                let prefix = &pattern[..start];
                let suffix = &pattern[end + 1..];
                let options = &pattern[start + 1..end];
                
                return options
                    .split(',')
                    .map(|opt| format!("{}{}{}", prefix, opt.trim(), suffix))
                    .collect();
            }
        }
    }
    
    // No braces found, return original pattern
    vec![pattern.to_string()]
}

fn main() {
    let matches = Command::new("fast-i18n-scan")
        .version(fast_i18n_scan::VERSION)
        .about("Fast i18n scanning tool for JavaScript/TypeScript projects")
        .arg(
            Arg::new("files")
                .help("Files to scan")
                .required(true)
                .num_args(1..)
                .value_name("FILE"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file for results")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .help("Output format")
                .value_name("FORMAT")
                .default_value("json"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let file_patterns: Vec<String> = matches
        .get_many::<String>("files")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let verbose = matches.get_flag("verbose");

    // Expand glob patterns to actual file paths
    let mut files = Vec::new();
    for pattern in &file_patterns {
        if verbose {
            println!("Processing pattern: '{}'", pattern);
        }
        
        // First check if it's a direct file path
        let path = std::path::Path::new(pattern);
        if path.is_file() {
            files.push(pattern.clone());
            if verbose {
                println!("  Found direct file: {}", pattern);
            }
            continue;
        }
        
        // Handle brace expansion manually (e.g., *.{js,jsx,ts,tsx})
        let expanded_patterns = expand_braces(pattern);
        
        for expanded_pattern in expanded_patterns {
            if verbose {
                println!("  Trying expanded pattern: '{}'", expanded_pattern);
            }
            
            // Try glob expansion
            match glob(&expanded_pattern) {
                Ok(paths) => {
                    let mut pattern_matches = 0;
                    for entry in paths {
                        match entry {
                            Ok(path) => {
                                if path.is_file() {
                                    // Accept all files that match the pattern
                                    files.push(path.to_string_lossy().to_string());
                                    pattern_matches += 1;
                                    if verbose {
                                        println!("    Found file: {}", path.display());
                                    }
                                }
                            }
                            Err(e) => {
                                if verbose {
                                    eprintln!("    Error processing path in pattern '{}': {}", expanded_pattern, e);
                                }
                            }
                        }
                    }
                    if verbose {
                        println!("    Pattern '{}' matched {} files", expanded_pattern, pattern_matches);
                    }
                }
                Err(e) => {
                    if verbose {
                        eprintln!("  Invalid glob pattern '{}': {}", expanded_pattern, e);
                    }
                }
            }
        }
    }

    if files.is_empty() {
        eprintln!("No files found matching the specified patterns: {:?}", file_patterns);
        eprintln!("Make sure:");
        eprintln!("  1. The patterns are correct");
        eprintln!("  2. The files exist in the current directory");
        eprintln!("  3. Try using quotes around the pattern to prevent shell expansion");
        eprintln!("  4. Use --verbose flag to see debug information");
        
        // Always show current directory contents for debugging
        eprintln!("\nCurrent directory contents:");
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let prefix = if file_type.is_dir() { "  [DIR]  " } else { "  [FILE] " };
                    eprintln!("{}{}", prefix, entry.file_name().to_string_lossy());
                }
            }
        }
        
        // Also try to show some example patterns
        eprintln!("\nExample patterns:");
        eprintln!("  fast-i18n-scan \"src/**/*.js\"");
        eprintln!("  fast-i18n-scan \"src/**/*.{{js,jsx,ts,tsx}}\"");
        eprintln!("  fast-i18n-scan \"**/*.js\" \"**/*.ts\"");
        
        process::exit(1);
    }

    if verbose {
        println!("Found {} files matching patterns: {:?}", files.len(), file_patterns);
        println!("Using configuration: {:?}", get_default_config());
    }

    match scan_files(&files) {
        Ok(result) => {
            if verbose {
                println!("Scan completed successfully!");
                println!("Files scanned: {}", result.stats.files_scanned);
                println!("Keys found: {}", result.stats.keys_found);
                println!("Errors: {}", result.stats.errors_count);
                println!("Warnings: {}", result.stats.warnings_count);
                println!("Processing time: {}ms", result.stats.processing_time_ms);
            }

            // Output results
            let output_format = matches.get_one::<String>("format").unwrap();
            match output_format.as_str() {
                "json" => {
                    let json_output = serde_json::to_string_pretty(&result).unwrap_or_else(|e| {
                        eprintln!("Failed to serialize results: {}", e);
                        process::exit(1);
                    });

                    if let Some(output_file) = matches.get_one::<String>("output") {
                        if let Err(e) = std::fs::write(output_file, json_output) {
                            eprintln!("Failed to write output file: {}", e);
                            process::exit(1);
                        }
                        if verbose {
                            println!("Results written to: {}", output_file);
                        }
                    } else {
                        println!("{}", json_output);
                    }
                }
                _ => {
                    eprintln!("Unsupported output format: {}", output_format);
                    process::exit(1);
                }
            }

            // Exit with error code if there are errors
            if result.stats.errors_count > 0 {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Scan failed: {}", e);
            process::exit(1);
        }
    }
}