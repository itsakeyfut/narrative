//! Animation Demo Example
//!
//! This example demonstrates the character emotion animation system.
//! It loads the animations.toml scenario from assets/scenarios/examples/
//! which showcases:
//! - Shake animation (surprise, anger)
//! - Jump animation (joy, excitement)
//! - Tremble animation (fear, anxiety)
//!
//! Run with: cargo run --example animation

use narrative_engine::EngineConfig;
use narrative_game::components::GameRootElement;
use narrative_gui::framework::{App, PresentMode, WindowOptions};

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Animation Demo");

    // Create and run GUI application
    App::new(WindowOptions {
        title: "Animation Demo - Narrative Engine".to_string(),
        width: 1280,
        height: 720,
        present_mode: PresentMode::VSync,
        target_fps: 60,
        show_fps_overlay: cfg!(debug_assertions),
        ..Default::default()
    })
    .with_root(|| {
        // Create engine configuration
        let mut config = EngineConfig::default();
        config.window.title = "Animation Demo".to_string();
        config.window.width = 1280;
        config.window.height = 720;

        // Create root element with animation demo scenario
        Box::new(GameRootElement::with_scenario(
            config,
            "assets/scenarios/examples/animations.toml",
        ))
    })
    .on_window_created(|window| {
        // Load default game assets after window creation
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
