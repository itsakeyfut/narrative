# Examples - Narrative Engine

This directory contains sample applications for testing each feature of the engine.

## How to Run

```bash
# Run from project root
cargo run --example <example_name>
```

## Available Examples

### transitions

**Purpose**: Comprehensive demo of all transition effects

Runs a dedicated demo scenario showcasing all scene transition effects in sequence.

**Run Command**:
```bash
cargo run --example transitions
```

**Demo Content**:
1. Fade (black fade) - 1.0s
2. FadeWhite (white fade) - 1.0s
3. Crossfade (crossfade) - 1.5s
4. Slide Left (left slide) - 1.0s
5. Slide Right (right slide) - 1.0s
6. Slide Up (up slide) - 1.0s
7. Slide Down (down slide) - 1.0s
8. Wipe Left (left wipe) - 1.0s
9. Wipe Right (right wipe) - 1.0s
10. Wipe Up (up wipe) - 1.0s
11. Wipe Down (down wipe) - 1.0s
12. Dissolve (dissolve) - 2.0s

**Scenario File**: `assets/scenarios/examples/transitions.toml`

---

### chapter_01_transitions

**Purpose**: Test transition effects integrated into main story

Shows how transition effects are applied in actual gameplay through Chapter 01's story.

**Run Command**:
```bash
cargo run --example chapter_01_transitions
```

**Transition Applications**:
- Opening scene: Fade in from black (1.0s)
- Scene transitions: Cross dissolve (0.5s)
- Special scenes: White fade (0.8s)
- Ending: Fade out to black (2.0s)

**Scenario File**: `assets/scenarios/chapter_01.toml`

---

### animation

**Purpose**: Complete demo of character emotion expression animations

Demonstrates all animation types (Shake, Jump, Tremble) and configuration options.

**Run Command**:
```bash
cargo run --example animation
```

**Demo Content**:
1. **Shake Animation**
   - Three preset levels: small, medium, large
   - Custom intensity settings demo
   - Timed mode demo

2. **Jump Animation**
   - Three preset levels: small, medium, large
   - Custom intensity settings demo
   - Timed mode demo

3. **Tremble Animation**
   - Three preset levels: small, medium, large
   - Continuous mode demo

4. **Combined Demo**
   - Combination of expressions and animations

**Scenario File**: `assets/scenarios/examples/animations_examples.toml`

---

### character_movement

**Purpose**: Complete demo of character movement animations

Demonstrates character position movement functionality.

**Run Command**:
```bash
cargo run --example character_movement
```

**Demo Content**:
1. **Movement to Predefined Positions**
   - Left, Center, Right, FarLeft, FarRight
   - Movement animations (0.5~0.8s)

2. **Simultaneous Multi-Character Movement**
   - Non-blocking movement commands
   - Dialogue during movement

3. **Movement to Custom Positions**
   - Fixed (fixed pixel position)
   - Custom (percentage position)

**Scenario File**: `assets/scenarios/examples/character_movement.toml`

---

### conditions

**Purpose**: Complete demo of condition branching system

Demonstrates flag-based conditional branching and complex conditions.

**Run Command**:
```bash
cargo run --example conditions
```

**Demo Content**:
1. **Complex Conditions**
   - AND/OR logical operations
   - Complex conditional expressions

2. **Flag-Based Branching**
   - Choice visibility control
   - Conditional scene transitions

**Scenario File**: `assets/scenarios/examples/conditions.toml`

---

## Development Guidelines

When adding new examples:

1. Create `<feature_name>.rs` file in this directory
2. Add description in doc comment at the top of the file
3. Add information to this README
4. Include command execution examples

**Template**:
```rust
//! <Feature> Test Example
//!
//! This example demonstrates <feature description>
//!
//! Run with: cargo run --example <name>

use narrative_app::components::GameRootElement;
use narrative_engine::EngineConfig;
use narrative_gui::framework::{App, PresentMode, WindowOptions};

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting <Example Name>");

    // Create and run GUI application
    App::new(WindowOptions {
        title: "<Example Title>".to_string(),
        width: 1280,
        height: 720,
        present_mode: PresentMode::VSync,
        target_fps: 60,
        show_fps_overlay: cfg!(debug_assertions),
        ..Default::default()
    })
    .with_root(|| {
        let mut config = EngineConfig::default();
        config.window.title = "<Window Title>".to_string();
        config.window.width = 1280;
        config.window.height = 720;

        Box::new(GameRootElement::with_scenario(
            config,
            "assets/scenarios/<your_scenario>.toml",
        ))
    })
    .on_window_created(|window| {
        match window.load_default_assets() {
            Ok((bg_id, char_id)) => {
                if let Some(root) = window.root_element_mut()
                    && let Some(game_root) = root.as_any_mut().downcast_mut::<GameRootElement>()
                {
                    game_root.set_texture_ids(bg_id, char_id);
                    tracing::info!("Successfully set texture IDs in GameRootElement");
                }
            }
            Err(e) => {
                tracing::error!("Failed to load default assets: {}", e);
            }
        }
    })
    .run()?;

    Ok(())
}
```
