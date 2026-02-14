//! Narrative Tools Library
//!
//! Development tools for visual novel creation.
//! This library provides functionality that can be used both from CLI and from the editor.
//!
//! ## Modules
//!
//! - `scenario_validator` - Scenario file validation
//! - `asset_optimizer` - Asset optimization utilities
//! - `perf_analyzer` - Performance analysis tools
//!
//! ## Usage from Editor
//!
//! ```no_run
//! use narrative_tools::scenario_validator::{self, ValidationConfig};
//!
//! # fn main() -> anyhow::Result<()> {
//! let config = ValidationConfig::default();
//! let result = scenario_validator::validate_file("assets/scenarios/chapter_01.toml", &config)?;
//! if result.has_errors() {
//!     println!("Validation failed: {:?}", result.errors);
//! }
//! # Ok(())
//! # }
//! ```

pub mod scenario_validator;

// Re-export commonly used types
pub use scenario_validator::{
    ValidationConfig, ValidationResult, validate_directory, validate_file,
};
