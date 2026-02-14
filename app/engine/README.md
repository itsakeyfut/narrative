# narrative-engine

**Responsibility**: Visual novel game engine core

## Overview

The execution engine for Narrative Novel Engine. Provides all features needed to run a game, including scenario execution, rendering, audio playback, and input handling.

## Responsibilities

### ✅ What this crate provides

#### 1. Runtime
- Scenario execution engine (`ScenarioRuntime`)
- State machine (`AppState`, `InGameState`)
- Flag & variable management (`FlagStore`, `VariableStore`)
- Read history (`ReadHistory`)

#### 2. Rendering
- wgpu-based 2D renderer (`Renderer`)
- Sprite rendering (`SpritePipeline`)
- Batch processing (`RenderBatch`)
- Render commands (`RenderCommand`)
- Transition effects

#### 3. Text
- cosmic-text based text rendering
- Typewriter effect (`TypewriterEffect`)
- Glyph cache (`GlyphCache`)
- Text layout (`TextLayout`)

#### 4. Audio
- BGM playback (`BgmPlayer`)
- Sound effect playback (`SePlayer`)
- Voice playback (`VoicePlayer`)
- Audio management (`AudioManager`)

#### 5. Input
- Keyboard & mouse input (`InputHandler`)
- Input state management (`InputState`)

#### 6. Save/Load
- Save data management (`SaveManager`)
- Save data structure (`SaveData`)
- Thumbnail generation

#### 7. Asset
- Asset loading (`AssetLoader`)
- Texture cache (`TextureCache`)
- Hot-reload support

#### 8. App
- Game loop (`GameLoop`)
- Engine configuration (`EngineConfig`)

### ❌ What this crate excludes

- ✗ Type definitions (`narrative-core`'s responsibility)
- ✗ GUI framework (`narrative-gui`'s responsibility)
- ✗ Game application (`narrative-game`'s responsibility)

## Dependencies

```toml
[dependencies]
narrative-core  # Type definitions

# GPU rendering
wgpu, bytemuck, winit, pollster

# Text rendering
cosmic-text, swash

# Audio
kira

# Others
glam, image, serde, toml, tokio, lru, tracing
```

## Module Structure

```
src/
├── runtime/       Scenario execution engine
│   └── executor/  Detailed command execution
├── render/        wgpu 2D renderer
├── text/          Text rendering
├── audio/         Audio playback
├── input/         Input handling
├── save/          Save/load
├── asset/         Asset management
├── ui/            UI components
└── app/           Game loop & configuration
```

## Design Principles

1. **VN-optimized**: No ECS, state machine based
2. **Stable 60FPS**: Performance-focused
3. **Modular**: Each module operates independently
4. **Testable**: Each feature is unit-testable

## Usage Example

```rust
use narrative_engine::{EngineConfig, GameLoop};

fn main() -> Result<()> {
    let config = EngineConfig::load("assets/config/game.toml")?;
    let mut game_loop = GameLoop::new(config)?;
    game_loop.run()?;
    Ok(())
}
```

## Relationship with Other Crates

```
game   → engine  (game execution)
editor → engine  (preview display)
```

## Performance Goals

- **FPS**: Stable 60 FPS operation
- **Startup time**: < 3 seconds
- **Memory usage**: < 500MB

