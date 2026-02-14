//! Scenario Validator CLI
//!
//! Command-line interface for scenario file validation.

use anyhow::Result;
use narrative_tools::scenario_validator::{self, ValidationConfig};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut config = ValidationConfig::default();
    let mut paths_to_validate = Vec::new();

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--strict" | "-s" => config.strict_mode = true,
            "--no-assets" => config.check_assets = false,
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            path if !path.starts_with("--") => {
                paths_to_validate.push(PathBuf::from(path));
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Default to validating scenarios directory
    if paths_to_validate.is_empty() {
        paths_to_validate.push(PathBuf::from("assets/scenarios"));
    }

    println!("🔍 Validating scenario files...");
    println!("📋 Configuration:");
    println!("   - Strict mode: {}", config.strict_mode);
    println!("   - Check assets: {}", config.check_assets);
    println!();

    let mut all_results = Vec::new();
    let mut total_errors = 0;
    let mut total_warnings = 0;

    for path in &paths_to_validate {
        if path.is_dir() {
            // Validate directory
            match scenario_validator::validate_directory(path, &config) {
                Ok(results) => {
                    for result in results {
                        total_errors += result.errors.len();
                        total_warnings += result.warnings.len();
                        print_validation_result(&result);
                        all_results.push(result);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to validate directory {}: {}", path.display(), e);
                }
            }
        } else {
            // Validate single file
            match scenario_validator::validate_file(path, &config) {
                Ok(result) => {
                    total_errors += result.errors.len();
                    total_warnings += result.warnings.len();
                    print_validation_result(&result);
                    all_results.push(result);
                }
                Err(e) => {
                    eprintln!("❌ Failed to validate file {}: {}", path.display(), e);
                }
            }
        }
    }

    // Summary
    println!("📊 Validation Summary:");
    println!("   - Files processed: {}", all_results.len());
    println!(
        "   - Files passed: {}",
        all_results.iter().filter(|r| r.success).count()
    );
    println!("   - Total errors: {}", total_errors);
    println!("   - Total warnings: {}", total_warnings);

    if total_errors > 0 {
        std::process::exit(1);
    }

    println!("✅ All scenario files validated successfully!");
    Ok(())
}

fn print_help() {
    println!("Scenario Validator");
    println!("Validate visual novel scenario TOML files");
    println!();
    println!("USAGE:");
    println!("    scenario-validator [OPTIONS] [FILES_OR_DIRS...]");
    println!();
    println!("OPTIONS:");
    println!("    -s, --strict        Enable strict validation mode");
    println!("        --no-assets     Skip asset file validation");
    println!("    -h, --help          Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    scenario-validator                          # Validate assets/scenarios/");
    println!("    scenario-validator chapter_01.toml          # Validate specific file");
    println!("    scenario-validator --strict scenarios/      # Strict validation of directory");
}

fn print_validation_result(result: &narrative_tools::scenario_validator::ValidationResult) {
    if result.success {
        println!("✅ {}", result.file_path.display());
    } else {
        println!("❌ {}", result.file_path.display());
    }

    for error in &result.errors {
        println!("   ERROR: {}", error);
    }

    for warning in &result.warnings {
        println!("   WARNING: {}", warning);
    }

    if !result.errors.is_empty() || !result.warnings.is_empty() {
        println!();
    }
}
