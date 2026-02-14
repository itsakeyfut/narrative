# narrative-tools

Development tools for visual novel creation

## Overview

This crate provides multiple command-line tools to support the development of Narrative Novel Engine. It includes tools for scenario validation, asset conversion, performance testing, and more to streamline the development workflow.

## Provided Tools

### scenario-validator

A tool to validate the syntax and semantics of scenario files.

**Features:**
- TOML syntax checking
- Detection of non-existent character ID references
- Detection of transitions to non-existent scenes
- Detection of circular references
- Warnings for unreachable scenes
- Asset file existence verification
- Condition expression syntax error detection

**Usage:**
```bash
# Validate a single file
cargo run --bin scenario-validator -- assets/scenarios/chapter_01.toml

# Validate an entire directory
cargo run --bin scenario-validator -- assets/scenarios/

# Detailed validation
cargo run --bin scenario-validator -- --verbose assets/scenarios/chapter_01.toml
```

### asset-converter

A tool for converting and optimizing asset files.

**Usage:**
```bash
cargo run --bin asset-converter -- [OPTIONS]
```

### scenario-editor

Scenario editor (in development).

**Usage:**
```bash
cargo run --bin scenario-editor
```

### perf-test

Performance test runner (for Issue #111).

**Features:**
- Runs test scenarios with 200+ dialogue lines
- FPS overlay display
- Frame time measurement
- Monitoring of layout/paint/GPU transfer times
- Memory leak checking

**Acceptance Criteria:**
- FPS: Stable 60 FPS in all states
- P95 frame time: < 16.67ms
- Layout time: < 2ms
- Paint time: < 3ms
- GPU transfer time: < 11ms
- No frame drops during typewriter effect
- No memory leaks after 30 minutes of operation

**Usage:**
```bash
# Run in development mode
cargo run -p narrative-tools --bin perf-test

# Run in release mode (recommended for accurate measurements)
cargo run -p narrative-tools --bin perf-test --release
```

**Test Procedure:**
1. Check the FPS overlay (top-left)
2. Click dialogue to test typewriter effect
3. Monitor metrics in each game state:
   - Idle state
   - Dialogue display
   - Typewriter effect
   - Choice menu
   - Long dialogue (100+ lines)
4. Run for at least 30 minutes to check memory stability
5. Press ESC to exit

## Usage as a Library

This crate can also be used as a library and called directly from the editor.

```rust
use narrative_tools::scenario_validator::{self, ValidationConfig};

// Validate scenario file
let config = ValidationConfig::default();
let result = scenario_validator::validate_file("scenario.toml", &config)?;

if result.has_errors() {
    eprintln!("Validation failed: {:?}", result.errors);
}

// Validate entire directory
let results = scenario_validator::validate_directory("assets/scenarios", &config)?;
for result in results {
    println!("{}: {}", result.file_path.display(),
             if result.success { "✅" } else { "❌" });
}
```

See `examples/validate_from_code.rs` for usage examples.

## Dependencies

- **narrative-engine**: Core engine
- **narrative-game**: Application components (for perf-test)
- **narrative-gui**: GUI framework (for perf-test)
- **clap**: Command-line argument parsing
- **walkdir**: Directory traversal
- **indicatif**: Progress bar display

## Developer Information

### Adding New Tools

1. Create a new Rust file in `src/bin/`
2. Add a `[[bin]]` section to `Cargo.toml`
3. Add dependencies as needed

### Project Structure

```
app/tools/
├── Cargo.toml
├── README.md
└── src/
    └── bin/
        ├── scenario_validator.rs
        ├── asset_converter.rs
        ├── scenario_editor.rs
        └── perf_test.rs
```

## Related Documentation

- [Scenario Format Specification](../../docs/scenario-format.md)
- [Asset Management](../../docs/assets-management.md)
- [Performance Testing](../../docs/performance-testing.md)
