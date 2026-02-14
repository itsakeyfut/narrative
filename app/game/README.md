# narrative-game

**Responsibility**: Game application executable

## Overview

A GUI application that runs games created with the Narrative Novel Engine. Provides a graphical experience using `narrative-gui`.

## Responsibilities

### ✅ What this crate provides

#### 1. Game UI
- Title screen
- Dialogue box
- Choice menu
- Save/load screen
- Settings screen
- CG gallery
- Backlog

#### 2. Game Logic
- `GameRootElement`: Main game component
- State management
- Input handling
- Rendering integration

#### 3. Application Integration
- Integration with `narrative-engine`
- Integration with `narrative-gui`
- Configuration loading and saving

### ❌ What this crate excludes

- ✗ Engine implementation (`narrative-engine`'s responsibility)
- ✗ GUI framework (`narrative-gui`'s responsibility)
- ✗ Editor features (`narrative-editor`'s responsibility)

## Dependencies

```toml
[dependencies]
narrative-core    # Type definitions
narrative-engine  # Game engine
narrative-gui     # GUI framework

# Others
anyhow, tracing, image, taffy
```

## Directory Structure

```
src/
├── main.rs              # Application entry point
└── components/         # Game UI components
    ├── game_root/      # Main game component
    ├── dialogue_box.rs
    ├── choice_menu.rs
    ├── backlog.rs
    ├── cg_gallery.rs
    └── ...
```

## Usage

### Run the game
```bash
cargo run
# or
cargo run --bin narrative-game
```

### Run in debug mode
```bash
cargo run --features dev
```

## Relationship with Other Crates

```
game → engine  (game execution)
game → gui     (UI building)
game → core    (type definitions)
```

## Notes

- This crate is **for game execution only**
- Use `narrative-editor` for game creation features
