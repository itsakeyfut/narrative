# =============================================================================
# justfile - Daily development workflow
# =============================================================================
#
# Narrative Novel Engine - Development Workflow
#
# =============================================================================

# Update local main branch
new:
    git checkout main && git fetch && git pull origin main

# === Build Commands ===

# Build workspace in debug mode
build:
    cargo build --workspace

# Build game in debug mode (faster compile, more logging)
dev-build:
    cargo build -p narrative-game

# Build game with dev features (debug overlay, hot reload)
dev-build-full:
    cargo build -p narrative-game --features dev

# Build game in release mode (optimized, minimal logging)
release-build:
    cargo build -p narrative-game --release

# Build editor in debug mode
editor-build:
    cargo build -p narrative-editor

# === Run Commands ===

# Run editor (default)
run:
    cargo run -p narrative-editor

# Run game
run-game:
    cargo run -p narrative-game

# Run game in debug mode with debug logging
dev-run:
    RUST_LOG=debug,wgpu=warn,wgpu_hal=warn,naga=warn,cosmic_text=warn cargo run -p narrative-game

# Run game with dev features
dev-run-full:
    RUST_LOG=debug,wgpu=warn,wgpu_hal=warn,naga=warn,cosmic_text=warn cargo run -p narrative-game --features dev

# Run game in release mode with info logging
release-run:
    cargo run -p narrative-game --release

# Run editor in debug mode
editor-run:
    RUST_LOG=debug cargo run -p narrative-editor

# Run performance test
perf-test:
    cargo run --bin perf-test --release

# === Shortcuts ===

# Build and run in debug mode
dev: dev-build dev-run

# Build and run with dev features
dev-full: dev-build-full dev-run-full

# Build and run in release mode
release: release-build release-run

# === Code Quality ===

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --workspace -- -D warnings

# Quick check (format + clippy)
check:
    cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings

# === Testing ===

# Run all tests (unit + integration) for all crates
test:
    cargo test --workspace

# Run unit tests: all crates / specific crate / specific test in crate
# Examples:
#   just unit-test                    # All unit tests
#   just unit-test ff-decode          # All unit tests in ff-decode
#   just unit-test ff-decode test_foo # Specific test in ff-decode
unit-test crate="" test="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{crate}}" ]; then
        cargo test --workspace --lib
    elif [ -z "{{test}}" ]; then
        cargo test -p {{crate}} --lib
    else
        cargo test -p {{crate}} --lib {{test}}
    fi

# Run integration tests: all crates / specific crate / specific test in crate
# Examples:
#   just integration-test                    # All integration tests
#   just integration-test ff-decode          # All integration tests in ff-decode
#   just integration-test ff-decode test_foo # Specific test in ff-decode
integration-test crate="" test="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{crate}}" ]; then
        cargo test --workspace --tests
    elif [ -z "{{test}}" ]; then
        cargo test -p {{crate}} --tests
    else
        cargo test -p {{crate}} --tests {{test}}
    fi

# Run tests sequentially (saves memory)
test-seq:
    cargo test --workspace -- --test-threads=1
