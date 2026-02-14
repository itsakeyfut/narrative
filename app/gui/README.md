# narrative-gui

A custom wgpu-based GUI framework for the Narrative Novel Engine.

## Overview

This crate implements a custom GUI framework inspired by Zed's GPUI. It provides GPU-first rendering optimized for the visual novel game engine editor.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  narrative-gui                       │
├─────────────────────────────────────────────────────┤
│  framework/    Core abstractions                     │
│  ├── App, Window, Element                           │
│  ├── Renderer (wgpu)                                │
│  ├── Layout (taffy flexbox)                         │
│  └── Reactive (Signal/Effect)                       │
├─────────────────────────────────────────────────────┤
│  components/   Reusable UI components                │
│  └── Button, Card, Icon, Sidebar, etc.              │
├─────────────────────────────────────────────────────┤
│  theme/        Color palette & styles                │
└─────────────────────────────────────────────────────┘
```

**Key modules** (`src/framework/`):
- `app.rs` - Application lifecycle management
- `window.rs` - Window management (winit integration)
- `element.rs` - UI element trait and basic components (Container, Text)
- `renderer/` - wgpu-based rendering (quad, text, batch)
- `layout.rs` - Taffy-based flexbox layout engine
- `reactive.rs` - Reactive system (Signal/Effect)
- `metrics.rs` - Performance measurement (FPS tracking)
- `menu.rs` - Native menu bar integration (muda)

## Key Features

- **GPU-first** - Direct rendering via wgpu
- **Flexbox layout** - Flexible layouts via Taffy
- **Reactive system** - Fine-grained state management with Signal/Effect
- **Performance optimizations**
  - Batch rendering (minimize draw calls)
  - LRU glyph cache (text rendering)
  - Dirty tracking (optimized redrawing via change detection)
- **Component-based** - Reusable UI parts

## Usage Example

```rust
use narrative_gui::{App, GuiConfig, WindowOptions};

// Configuration
let config = GuiConfig::default();
let window_options: WindowOptions = config.into();

// Create and run application
let app = App::new(window_options)
    .with_root(|| Box::new(MyRootElement::new()));

app.run()?;
```

## Provided Features

### Framework (`framework`)

Core framework:

- `App` - Application lifecycle
- `Window` - Window management (winit)
- `Element` - UI element trait
- `Renderer` - wgpu renderer (quad, text)
- `Container` / `Text` - Basic elements
- Layout engine (taffy flexbox)
- Reactive system (Signal/Effect)

### Components (`components`)

UI components:

- `Button` - Clickable button
- `Card` - Card layout
- `Icon` - Icon display
- `Sidebar` - Sidebar
- `Dropdown` - Dropdown menu

### Theme (`theme`)

Theme system:

- Dark theme support
- Color palette
- Style constants (font sizes, layout, typography)

## GuiConfig

```rust
pub struct GuiConfig {
    pub title: String,                    // "Narrative Novel Editor"
    pub width: u32,                       // 1600
    pub height: u32,                      // 900
    pub maximize_on_startup: bool,        // true
    pub dark_theme: bool,                 // true
    pub show_fps_overlay: bool,           // false
}
```

## Future Enhancements

- Animation system
- Improved drag & drop
- Customizable keyboard shortcuts
- Theme customization
- Improved high-DPI support
- Editor-specific components (node editor, scenario preview, etc.)
