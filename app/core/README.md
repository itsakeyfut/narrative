# narrative-core

**Responsibility**: Type definitions and data schemas

## Overview

A crate that provides core type definitions for the Narrative Novel Engine. It defines pure data structures with minimal external dependencies.

## Responsibilities

### ✅ What this crate provides

- **Scenario types**: `Scenario`, `Scene`, `ScenarioCommand`, `Dialogue`, `Choice`
- **Character types**: `CharacterDef`, `Expression`, `CharacterPosition`
- **Configuration types**: `GameConfig`, `UserSettings`, `GraphicsConfig`, `AudioConfig`
- **Primitive types**: `Color`, `Point`, `Rect`, `Size`, `AssetRef`
- **ID types**: `SceneId`, `CharacterId`, `AssetRef`, `FlagId`, `VariableId`
- **Error types**: `EngineError`, `ScenarioError`, `ConfigError`
- **Conditions & variables**: `Condition`, `VariableOperation`, `Variable`
- **Asset metadata**: `BackgroundManifest`, `CharacterManifest`, `BgmManifest`

### ❌ What this crate excludes

- ✗ Library dependencies like wgpu/kira
- ✗ Execution logic (runtime, renderer, etc.)
- ✗ Window management, input handling
- ✗ File I/O (loading logic)

## Dependencies

```toml
[dependencies]
serde       # Serialization
toml        # TOML format
ron         # RON format
thiserror   # Error definitions
```

**No external crate dependencies** - Lightweight and reusable

## Design Principles

1. **Pure type definitions**: No business logic
2. **Lightweight**: Minimal dependencies for fast builds
3. **Reusable**: Shared across tools, editor, and engine
4. **Serializable**: All types support `serde`

## Usage Example

```rust
use narrative_core::{Scenario, Scene, ScenarioCommand, Dialogue, Speaker};

// Build scenario
let mut scenario = Scenario::new(metadata, "scene_01");
let mut scene = Scene::new("scene_01", "Opening");

scene.add_command(ScenarioCommand::Dialogue {
    dialogue: Dialogue::character("alice", "Hello!"),
});

scenario.add_scene("scene_01", scene);
```

## Relationship with Other Crates

```
tools   → core  (scenario validation)
engine  → core  (use type definitions)
gui     → core  (use config types)
game    → core  (use type definitions)
editor  → core  (scenario editing)
```
