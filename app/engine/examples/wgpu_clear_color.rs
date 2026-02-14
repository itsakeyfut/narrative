//! Example: wgpu clear color
//!
//! This example demonstrates the basic wgpu setup by displaying a window
//! with a clear color (dark blue).
//!
//! Usage:
//!   cargo run --example wgpu_clear_color

use narrative_engine::{EngineConfig, GameLoop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine configuration with custom window settings
    let mut config = EngineConfig::default();
    config.window.title = "wgpu Clear Color Test".to_string();
    config.window.width = 1280;
    config.window.height = 720;

    // Create and run game loop
    let game_loop = GameLoop::with_config(config);
    game_loop.run()?;

    Ok(())
}
