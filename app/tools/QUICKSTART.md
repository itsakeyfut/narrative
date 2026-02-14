# Quick Start: narrative-tools

## 🚀 Two Ways to Use

---

## ⚡ Method 1: Run as CLI Tool

For validating scenario files from the command line.

```bash
# Run from project root
cd /path/to/narrative

# Validate scenario
cargo run -p narrative-tools --bin scenario-validator -- assets/scenarios/chapter_01.toml

# Validate entire directory
cargo run -p narrative-tools --bin scenario-validator

# Show detailed warnings
cargo run -p narrative-tools --bin scenario-validator -- --strict assets/scenarios/
```

### Common Commands

| Command | Description |
|---------|-------------|
| `-- --help` | Show help |
| `-- file.toml` | Validate file |
| `-- dir/` | Validate directory |
| `-- --strict` | Strict mode |
| `-- --no-assets` | Skip asset checking |

---

## 💻 Method 2: Call from Code (Editor Integration)

For using validation features from within a program.

### Step 1: Add Dependency

```toml
# Cargo.toml
[dependencies]
narrative-tools = { path = "../tools" }
```

### Step 2: Use in Code

```rust
use narrative_tools::scenario_validator::{self, ValidationConfig};

fn main() -> anyhow::Result<()> {
    // Validation configuration
    let config = ValidationConfig::default();

    // Validate file
    let result = scenario_validator::validate_file(
        "assets/scenarios/chapter_01.toml",
        &config
    )?;

    // Display results
    if result.has_errors() {
        println!("❌ {} errors found", result.errors.len());
        for error in &result.errors {
            println!("  - {}", error);
        }
    } else {
        println!("✅ Validation passed!");
    }

    Ok(())
}
```

### Common APIs

| API | Description |
|-----|-------------|
| `validate_file(path, config)` | Validate single file |
| `validate_directory(dir, config)` | Validate directory |
| `result.has_errors()` | Check for errors |
| `result.has_warnings()` | Check for warnings |

---

## 📋 Both Are Available!

```
┌─────────────────────────────────────────────┐
│          narrative-tools crate               │
├─────────────────────────────────────────────┤
│                                             │
│  📚 Library Functions                       │
│  └─→ use narrative_tools::* from editor    │
│                                             │
│  🖥️  CLI Tools                             │
│  └─→ cargo run --bin scenario-validator    │
│                                             │
└─────────────────────────────────────────────┘
```

Both use the **same core functionality**, so behavior is identical.

---

## 🔗 Detailed Usage

- **Details**: [USAGE.md](./USAGE.md)
- **Code Examples**: [examples/validate_from_code.rs](./examples/validate_from_code.rs)
- **API Specification**: [README.md](./README.md)
