# Tools Usage Guide

This document explains how to use the narrative-tools crate.

## 📋 Dual Usage

narrative-tools can be used as both a **library** and **CLI tool**.

---

## 🖥️ Use as CLI (For Scenario Writers & Developers)

### scenario-validator

CLI tool for validating scenario files.

#### Basic Usage

```bash
# Run from workspace root
cd /path/to/narrative

# Show help
cargo run -p narrative-tools --bin scenario-validator -- --help

# Validate single file
cargo run -p narrative-tools --bin scenario-validator -- assets/scenarios/chapter_01.toml

# Validate entire directory (default)
cargo run -p narrative-tools --bin scenario-validator

# Explicitly specify directory
cargo run -p narrative-tools --bin scenario-validator -- assets/scenarios/

# Strict mode (show more warnings)
cargo run -p narrative-tools --bin scenario-validator -- --strict assets/scenarios/

# Skip asset checking
cargo run -p narrative-tools --bin scenario-validator -- --no-assets chapter_01.toml
```

#### Build and Run Directly

```bash
# Release build
cargo build --release -p narrative-tools --bin scenario-validator

# Run directly (faster)
./target/release/scenario-validator.exe assets/scenarios/chapter_01.toml

# Or development build
cargo build -p narrative-tools --bin scenario-validator
./target/debug/scenario-validator.exe --help
```

#### Alias Setup (Optional)

Setting up an alias is convenient for frequent use.

**Bash/Zsh (.bashrc / .zshrc)**
```bash
alias validate-scenario='cargo run -p narrative-tools --bin scenario-validator --'
```

Usage:
```bash
validate-scenario assets/scenarios/chapter_01.toml
validate-scenario --strict assets/scenarios/
```

**PowerShell (Microsoft.PowerShell_profile.ps1)**
```powershell
function Validate-Scenario { cargo run -p narrative-tools --bin scenario-validator -- $args }
Set-Alias validate Validate-Scenario
```

Usage:
```powershell
validate assets/scenarios/chapter_01.toml
validate --strict assets/scenarios/
```

---

## 📚 Use as Library (From Editor or Program)

### Add Dependency to Cargo.toml

```toml
[dependencies]
narrative-tools = { path = "../tools" }
```

### Code Examples

#### Basic Usage

```rust
use narrative_tools::scenario_validator::{self, ValidationConfig};

fn main() -> anyhow::Result<()> {
    // Validate with default configuration
    let config = ValidationConfig::default();
    let result = scenario_validator::validate_file(
        "assets/scenarios/chapter_01.toml",
        &config
    )?;

    // Check results
    if result.has_errors() {
        eprintln!("❌ Validation failed!");
        for error in &result.errors {
            eprintln!("  ERROR: {}", error);
        }
    } else {
        println!("✅ Validation passed!");
    }

    Ok(())
}
```

#### Usage in Editor

```rust
use narrative_tools::scenario_validator::{self, ValidationConfig};

struct ScenarioEditor {
    current_file: PathBuf,
    validation_results: Option<ValidationResult>,
}

impl ScenarioEditor {
    /// When validation button is clicked
    fn on_validate_clicked(&mut self) {
        let config = ValidationConfig {
            strict_mode: true,
            check_assets: true,
        };

        match scenario_validator::validate_file(&self.current_file, &config) {
            Ok(result) => {
                self.validation_results = Some(result.clone());

                if result.has_errors() {
                    self.show_error_dialog(&result.errors);
                } else if result.has_warnings() {
                    self.show_warning_dialog(&result.warnings);
                } else {
                    self.show_success_message();
                }
            }
            Err(e) => {
                self.show_error_dialog(&[format!("Validation failed: {}", e)]);
            }
        }
    }

    /// Validate entire project
    fn validate_all_scenarios(&mut self) {
        let config = ValidationConfig::default();

        match scenario_validator::validate_directory("assets/scenarios", &config) {
            Ok(results) => {
                let total = results.len();
                let passed = results.iter().filter(|r| r.success).count();
                let failed = total - passed;

                self.show_validation_summary(total, passed, failed, results);
            }
            Err(e) => {
                self.show_error_dialog(&[format!("Failed to validate directory: {}", e)]);
            }
        }
    }

    // UI display methods (example implementation)
    fn show_error_dialog(&self, errors: &[String]) {
        println!("❌ Errors:");
        for error in errors {
            println!("  - {}", error);
        }
    }

    fn show_warning_dialog(&self, warnings: &[String]) {
        println!("⚠️  Warnings:");
        for warning in warnings {
            println!("  - {}", warning);
        }
    }

    fn show_success_message(&self) {
        println!("✅ Validation successful!");
    }

    fn show_validation_summary(
        &self,
        total: usize,
        passed: usize,
        failed: usize,
        results: Vec<ValidationResult>,
    ) {
        println!("📊 Validation Summary:");
        println!("  Total: {}", total);
        println!("  Passed: {}", passed);
        println!("  Failed: {}", failed);

        for result in results {
            if result.has_errors() {
                println!("\n❌ {}", result.file_path.display());
                for error in &result.errors {
                    println!("    - {}", error);
                }
            }
        }
    }
}
```

#### Real-time Validation

```rust
use narrative_tools::scenario_validator::{self, ValidationConfig};
use std::time::Duration;

struct RealtimeValidator {
    last_validation: Option<std::time::Instant>,
    debounce_duration: Duration,
}

impl RealtimeValidator {
    fn new() -> Self {
        Self {
            last_validation: None,
            debounce_duration: Duration::from_millis(500),
        }
    }

    /// Called when file changes
    fn on_file_changed(&mut self, file_path: &Path) {
        let now = std::time::Instant::now();

        // Debounce processing
        if let Some(last) = self.last_validation {
            if now.duration_since(last) < self.debounce_duration {
                return; // Not enough time since last validation
            }
        }

        self.last_validation = Some(now);

        // Run validation in background
        self.validate_in_background(file_path);
    }

    fn validate_in_background(&self, file_path: &Path) {
        let path = file_path.to_owned();

        std::thread::spawn(move || {
            let config = ValidationConfig::default();
            match scenario_validator::validate_file(&path, &config) {
                Ok(result) => {
                    // Send result to UI (using channels, etc.)
                    println!("Background validation: {} errors", result.errors.len());
                }
                Err(e) => {
                    eprintln!("Background validation failed: {}", e);
                }
            }
        });
    }
}
```

---

## 🔧 API Reference

### ValidationConfig

```rust
pub struct ValidationConfig {
    /// Strict mode (show more warnings)
    pub strict_mode: bool,
    /// Check asset file existence
    pub check_assets: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            check_assets: true,
        }
    }
}
```

### ValidationResult

```rust
pub struct ValidationResult {
    /// Path to validated file
    pub file_path: PathBuf,
    /// List of error messages
    pub errors: Vec<String>,
    /// List of warning messages
    pub warnings: Vec<String>,
    /// Whether validation succeeded (no errors)
    pub success: bool,
}

impl ValidationResult {
    /// Check if there are errors
    pub fn has_errors(&self) -> bool;

    /// Check if there are warnings
    pub fn has_warnings(&self) -> bool;
}
```

### Functions

```rust
/// Validate single file
pub fn validate_file(
    file_path: impl AsRef<Path>,
    config: &ValidationConfig
) -> Result<ValidationResult>

/// Validate all TOML files in directory
pub fn validate_directory(
    dir_path: impl AsRef<Path>,
    config: &ValidationConfig
) -> Result<Vec<ValidationResult>>
```

---

## 📝 Execution Examples

### Success Example

```bash
$ cargo run -p narrative-tools --bin scenario-validator -- assets/scenarios/chapter_01.toml

🔍 Validating scenario files...
📋 Configuration:
   - Strict mode: false
   - Check assets: true

✅ assets/scenarios/chapter_01.toml

📊 Validation Summary:
   - Files processed: 1
   - Files passed: 1
   - Total errors: 0
   - Total warnings: 0

✅ All scenario files validated successfully!
```

### Error Example

```bash
$ cargo run -p narrative-tools --bin scenario-validator -- broken.toml

🔍 Validating scenario files...
📋 Configuration:
   - Strict mode: false
   - Check assets: true

❌ broken.toml
   ERROR: Chapter ID cannot be empty
   ERROR: Scene 'intro': Choice references non-existent scene 'missing_scene'

📊 Validation Summary:
   - Files processed: 1
   - Files passed: 0
   - Total errors: 2
   - Total warnings: 0
```

---

## 📚 Related Documentation

- [README.md](./README.md) - Crate overview
- [examples/validate_from_code.rs](./examples/validate_from_code.rs) - Code examples
- [../../docs/scenario-format.md](../../docs/scenario-format.md) - Scenario format specification
