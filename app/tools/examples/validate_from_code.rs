//! Example: Using scenario validator from code
//!
//! This demonstrates how the editor can use the validation library.

use narrative_tools::scenario_validator::{self, ValidationConfig};

fn main() -> anyhow::Result<()> {
    println!("📝 Example: Validating scenarios from code\n");

    // Example 1: Validate a single file
    println!("1. Validating single file:");
    let config = ValidationConfig::default();

    match scenario_validator::validate_file("assets/scenarios/chapter_01.toml", &config) {
        Ok(result) => {
            println!("   File: {}", result.file_path.display());
            println!("   Success: {}", result.success);
            println!("   Errors: {}", result.errors.len());
            println!("   Warnings: {}", result.warnings.len());

            if result.has_errors() {
                println!("\n   ❌ Errors found:");
                for error in &result.errors {
                    println!("      - {}", error);
                }
            }

            if result.has_warnings() {
                println!("\n   ⚠️  Warnings:");
                for warning in &result.warnings {
                    println!("      - {}", warning);
                }
            }
        }
        Err(e) => {
            eprintln!("   ❌ Validation failed: {}", e);
        }
    }

    println!("\n2. Validating directory:");
    match scenario_validator::validate_directory("assets/scenarios", &config) {
        Ok(results) => {
            println!("   Found {} scenario files", results.len());

            let passed = results.iter().filter(|r| r.success).count();
            let failed = results.len() - passed;

            println!("   ✅ Passed: {}", passed);
            println!("   ❌ Failed: {}", failed);

            for result in &results {
                if result.has_errors() {
                    println!("\n   {} - FAILED", result.file_path.display());
                    for error in &result.errors {
                        println!("      ERROR: {}", error);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("   ❌ Directory validation failed: {}", e);
        }
    }

    println!("\n3. Strict mode validation:");
    let strict_config = ValidationConfig {
        strict_mode: true,
        check_assets: true,
    };

    match scenario_validator::validate_file("assets/scenarios/chapter_01.toml", &strict_config) {
        Ok(result) => {
            println!("   Warnings in strict mode: {}", result.warnings.len());
        }
        Err(e) => {
            eprintln!("   ❌ Strict validation failed: {}", e);
        }
    }

    Ok(())
}
