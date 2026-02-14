# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Run applications
cargo run -p narrative-editor     # Run editor (workspace default: narrative-game)
cargo run -p narrative-game       # Run game
cargo run -p narrative-game --features dev  # Run game with debug overlay/tools

# Using just (recommended)
just run                          # Run editor (default just command)
just run-game                     # Run game
just dev                          # Build and run game in debug mode with RUST_LOG=debug
just dev-full                     # Build and run game with dev features + debug logging
just release                      # Build and run game in release mode
just new                          # Update local main branch (git pull)

# Quality checks (run before committing)
cargo fmt                         # Format code
cargo clippy -- -D warnings       # Lint check
cargo test                        # Run all tests
cargo build                       # Build workspace

# Quick check (format + clippy)
just check                        # Run fmt --check && clippy

# Testing
just test                         # Run all tests
just unit-test [crate] [test]     # Run unit tests (optionally filter by crate/test name)
just integration-test [crate] [test]  # Run integration tests (optionally filter)
just test-seq                     # Run tests sequentially (saves memory)

# Examples:
#   just unit-test                      # All unit tests
#   just unit-test narrative-engine     # Unit tests for narrative-engine
#   just unit-test narrative-engine test_save_load  # Specific test

# Development tools (in app/tools)
cargo run --bin scenario-validator -- assets/scenarios/chapter_01.toml  # Validate scenario
cargo run --bin asset-converter   # Convert assets
cargo run --bin scenario-editor   # Interactive scenario editor
cargo run --bin perf-test --release  # Run performance test (or: just perf-test)
```

### Environment Variables

```bash
# Debug logging (filters out verbose wgpu/naga logs)
RUST_LOG=debug,wgpu=warn,wgpu_hal=warn,naga=warn,cosmic_text=warn cargo run -p narrative-game

# Full debug logging
RUST_LOG=debug cargo run -p narrative-editor
```

## Architecture Overview

wgpu-based visual novel engine with a 6-crate workspace:

```
app/
├── core/     # Shared types (scenarios, characters, saves, errors)
├── engine/   # Core VN library - dialogue, characters, scenes, save/load, audio
├── gui/      # Custom wgpu-based GUI framework (GPUI-inspired)
├── game/     # Game player application (GUI binary)
├── editor/   # Visual novel editor (placeholder for Phase 5)
└── tools/    # CLI utilities (scenario-validator, asset-converter)
```

**Key engine modules** (`app/engine/src/`):
- `app/` - Application lifecycle (GameLoop, EngineConfig)
- `render/` - wgpu rendering (Renderer, quad, text)
- `audio/` - Audio playback (AudioManager)
- `text/` - Text rendering and layout
- `asset/` - Asset loading and management
- `save/` - Save/load system
- `runtime/` - Runtime state and execution
- `ui/` - UI components

**GUI framework** (`app/gui/src/`):
- `framework/` - Core framework (App, Window, Element, Renderer)
  - wgpu-based rendering with batching optimization
  - Taffy flexbox layout engine
  - Reactive system (Signal/Effect)
- `components/` - Reusable UI components (Button, Card, Dropdown, Sidebar)
- `theme/` - Color palette and styling system

**Application flow:** `Window creation` → `wgpu setup` → `Game loop` → `Render frame`

## Feature Flags

**narrative-game:**
- `dev` - Enables `narrative-engine/debug` feature (debug overlay, hot-reload, dev tools)

**narrative-editor:**
- `hot-reload` - File watching and hot-reload support (optional dependencies: notify, crossbeam-channel)

**narrative-engine:**
- `debug` - Debug overlay, performance metrics, development tools

## Data Formats

- **Scenarios:** TOML format in `assets/scenarios/` (see `docs/scenario-format.md`)
- **Game config:** `assets/config/game.toml`
- **Design docs:** `docs/design/` (philosophy, architecture, features, roadmap)

## Language Requirements

All project documentation, code comments, commit messages, and PR descriptions should be in **English** as this is an open-source project with an international audience.

## Coding Guidelines

- Use `thiserror` for error handling - avoid `.unwrap()` in production code
- Avoid direct index access - use `.get()` or `.first()`
- Use saturating arithmetic (`saturating_add`, etc.)
- Target 60 FPS stable performance

### Test Code Organization

Follow the test splitting guidelines based on file size:

- **< 2000 lines**: Keep tests in the same file with `#[cfg(test)] mod tests`
- **≥ 2000 lines**: Split into `foo.rs` (implementation) and `foo_tests.rs` (tests)
- **Visibility**: Use `pub(super)` for internal functions that need testing (never `pub` just for tests)
- **Naming**: Implementation `foo.rs`, tests `foo_tests.rs`
- **Declaration**: `#[cfg(test)] mod foo_tests;` in mod.rs

See `docs/testing-guidelines.md` for detailed rules and examples.

## Git Conventions

**Branch naming:**
- `feat/<description>` - New features
- `fix/<description>` - Bug fixes
- `refactor/<description>` - Refactoring
- `docs/<description>` - Documentation
- `chore/<description>` - Maintenance

**Commit format:** `<type>(<scope>): <description in English>`

Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `perf`

Scopes: `engine`, `core`, `game`, `gui`, `editor`, `tools`, `dialogue`, `character`, `scene`, `save`, `choice`, `flag`, `audio`, `ui`, `render`, `asset`
