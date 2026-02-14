# Asset System

This module provides a unified asset management system for the Narrative Novel Engine.

## Architecture

**All assets are loaded through `AssetLoader`** - a centralized hub for managing:

1. **Manifests** (via AssetRegistry) - Asset definitions from RON files
2. **Scenarios** - TOML scenario files
3. **Textures** - Image files (PNG, etc.)
4. **Audio** - BGM and SE files
5. **Hot-reload** - Development-time automatic reloading (optional)

## Quick Start

```rust
use narrative_engine::asset::AssetLoader;

// Create unified asset loader
let mut loader = AssetLoader::new("assets");

// Load all manifests
loader.load_manifests()?;

// Load a scenario
let scenario = loader.load_scenario("scenarios/chapter_01.toml")?;

// Access asset definitions
let bg = loader.background("bg.school.classroom");
let bgm = loader.bgm("bgm.dailylife.school");
let character = loader.character("akane")?;

// Get statistics
let stats = loader.stats();
println!("Loaded {} scenarios", stats.scenarios);
println!("Loaded {} backgrounds", stats.backgrounds);
```

## Components

### AssetLoader (Unified Hub)

**All asset loading goes through AssetLoader.** This prevents confusion and ensures consistent asset management.

```rust
use narrative_engine::asset::AssetLoader;

let mut loader = AssetLoader::new("assets");

// 1. Load manifests
loader.load_manifests()?;

// 2. Load scenarios
let scenario = loader.load_scenario("scenarios/chapter_01.toml")?;
let cached = loader.get_scenario("chapter_01");

// 3. Access asset definitions
let bg = loader.background("bg.school.classroom");
let bgm = loader.bgm("bgm.dailylife.school");
let se = loader.sound_effect("se.ui.click");
let character = loader.character("akane")?;
let theme = loader.ui_theme("light");

// 4. Load textures (with automatic caching)
let texture = loader.load_texture(&asset_ref)?;

// 5. Get statistics
let stats = loader.stats();
```

### AssetRegistry (Internal)

AssetRegistry is used internally by AssetLoader. You don't need to use it directly.

Manifest files loaded:
- `assets/manifests/characters.ron`
- `assets/manifests/backgrounds.ron`
- `assets/manifests/bgm.ron`
- `assets/manifests/se.ron`
- `assets/manifests/ui_themes.ron`

### Hot-reload (Debug only)

Automatically reloads manifest files when they change on disk. Only available with the `hot-reload` feature flag.

```rust
#[cfg(feature = "hot-reload")]
use narrative_engine::asset::{HotReloadWatcher, ReloadEvent};

#[cfg(feature = "hot-reload")]
fn setup_hot_reload(loader: &mut AssetLoader) -> Result<(), Box<dyn std::error::Error>> {
    let (mut watcher, rx) = HotReloadWatcher::new("assets/manifests")?;
    watcher.start()?;

    // In your game loop:
    while let Ok(event) = rx.try_recv() {
        let registry = loader.registry_mut();
        match event {
            ReloadEvent::Backgrounds => {
                registry.backgrounds.load_manifest("manifests/backgrounds.ron")?;
                tracing::info!("🔥 Reloaded backgrounds");
            }
            ReloadEvent::Bgm => {
                registry.bgm.load_manifest("manifests/bgm.ron")?;
                tracing::info!("🔥 Reloaded BGM");
            }
            ReloadEvent::SoundEffects => {
                registry.se.load_manifest("manifests/se.ron")?;
                tracing::info!("🔥 Reloaded sound effects");
            }
            ReloadEvent::UiThemes => {
                registry.ui_themes.load_manifest("manifests/ui_themes.ron")?;
                tracing::info!("🔥 Reloaded UI themes");
            }
            ReloadEvent::Characters => {
                registry.characters.load_from_manifest("manifests/characters.ron")
                    .map_err(|e| format!("{}", e))?;
                tracing::info!("🔥 Reloaded characters");
            }
        }
    }

    Ok(())
}
```

## Features

### hot-reload

Enable hot-reload functionality for development:

```toml
[dependencies]
narrative-engine = { path = "../engine", features = ["hot-reload"] }
```

Or in your game config:

```toml
[development]
hot_reload = true
```

## Usage Example

Complete example with hot-reload:

```rust
use narrative_engine::asset::AssetLoader;
#[cfg(feature = "hot-reload")]
use narrative_engine::asset::{HotReloadWatcher, ReloadEvent};

struct Game {
    assets: AssetLoader,
    #[cfg(feature = "hot-reload")]
    hot_reload: Option<(HotReloadWatcher, crossbeam_channel::Receiver<ReloadEvent>)>,
}

impl Game {
    fn new(asset_path: &str, enable_hot_reload: bool) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize unified asset loader
        let mut assets = AssetLoader::new(asset_path);
        assets.load_manifests()?;

        // Initialize hot-reload if enabled
        #[cfg(feature = "hot-reload")]
        let hot_reload = if enable_hot_reload {
            let (mut watcher, rx) = HotReloadWatcher::new(format!("{}/manifests", asset_path))?;
            watcher.start()?;
            Some((watcher, rx))
        } else {
            None
        };

        Ok(Self {
            assets,
            #[cfg(feature = "hot-reload")]
            hot_reload,
        })
    }

    fn update(&mut self) {
        #[cfg(feature = "hot-reload")]
        if let Some((_, rx)) = &self.hot_reload {
            while let Ok(event) = rx.try_recv() {
                if let Err(e) = self.handle_reload_event(event) {
                    tracing::error!("Failed to reload asset: {}", e);
                }
            }
        }
    }

    #[cfg(feature = "hot-reload")]
    fn handle_reload_event(&mut self, event: ReloadEvent) -> Result<(), Box<dyn std::error::Error>> {
        let registry = self.assets.registry_mut();
        match event {
            ReloadEvent::Backgrounds => {
                registry.backgrounds.load_manifest("manifests/backgrounds.ron")?;
                tracing::info!("🔥 Reloaded backgrounds");
            }
            ReloadEvent::Bgm => {
                registry.bgm.load_manifest("manifests/bgm.ron")?;
                tracing::info!("🔥 Reloaded BGM");
            }
            ReloadEvent::SoundEffects => {
                registry.se.load_manifest("manifests/se.ron")?;
                tracing::info!("🔥 Reloaded sound effects");
            }
            ReloadEvent::UiThemes => {
                registry.ui_themes.load_manifest("manifests/ui_themes.ron")?;
                tracing::info!("🔥 Reloaded UI themes");
            }
            ReloadEvent::Characters => {
                registry.characters.load_from_manifest("manifests/characters.ron")
                    .map_err(|e| format!("{}", e))?;
                tracing::info!("🔥 Reloaded characters");
            }
        }
        Ok(())
    }
}
```

## Asset Manifest Format

See `/assets/README.md` and `/assets/MANIFEST_GUIDE.md` for detailed information on creating and managing asset manifests.

## Performance

- **Manifest loading**: Manifests are loaded once at startup (or when hot-reloaded)
- **Texture caching**: Textures are cached with LRU eviction
- **Hot-reload**: Uses debouncing (500ms) to prevent reload storms
- **Memory**: Asset definitions are small (just metadata), actual assets are loaded on-demand

## Error Handling

All operations return `Result` types:
- `EngineResult<T>` for engine operations
- `HotReloadError` for hot-reload operations

Errors are gracefully handled - if a hot-reload fails, the previous valid definitions remain in use.

## Statistics

Get statistics about loaded assets:

```rust
let stats = registry.stats();
println!("Loaded {} characters", stats.characters);
println!("Loaded {} backgrounds", stats.backgrounds);
println!("Loaded {} BGM tracks", stats.bgm_tracks);
println!("Loaded {} sound effects", stats.sound_effects);
println!("Loaded {} UI themes", stats.ui_themes);
println!("Total assets: {}", stats.total());
```
