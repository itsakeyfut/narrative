# Integration Tests - Narrative Engine

This directory contains integration tests for the engine.

## Purpose

- Scenario-based feature testing
- Regression testing
- Automated testing in CI/CD pipeline

## Test Files

### animation.rs

**Purpose:** Verify animation functionality

**Test Content:**
- Basic operation of each animation type
- Verify intensity levels (small/medium/large)
- Verify custom intensity operation
- Verify timing modes

**Scenario File:** `assets/scenarios/tests/animation.toml`

**How to Run:**
```bash
# Currently run as an Example
cargo run --example animation

# Will be run as integration test in the future
cargo test --test animation
```

---

### cg.rs

**Purpose:** Verify CG display functionality

**Scenario File:** `assets/scenarios/tests/cg.toml`

**How to Run:**
```bash
# Currently run as an Example
cargo run --example cg

# Will be run as integration test in the future
cargo test --test cg
```

---

## Test Implementation Guidelines

When adding new integration tests:

1. **Create Test File**
   - Create `app/game/tests/<feature>_test.rs`
   - Use Rust's integration test framework

2. **Prepare Scenario**
   - Create corresponding scenario in `assets/scenarios/tests/`
   - Focus test cases on specific features

3. **Implement Test**
   - Load and execute scenario
   - Verify expected results
   - Cover error cases

4. **Update Documentation**
   - Add information to this README
   - Also update `assets/scenarios/tests/README.md`

## Current Status

Currently, these test files are run with `cargo run --example`,
but will be migrated to integration tests that can be run with `cargo test` in the future.

## Related Links

- [Test Scenarios](../../../assets/scenarios/tests/)
- [Example Files](../examples/)
- [Examples README](../examples/README.md)
