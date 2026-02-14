//! Character Movement Demo Example
//!
//! This example demonstrates the character movement animation system.
//! It loads the character_movement.toml scenario from assets/scenarios/examples/
//! which showcases:
//! - Predefined positions (Left, Center, Right, FarLeft, FarRight)
//! - Fixed pixel positions
//! - Custom percentage positions
//! - Simultaneous character movements
//! - Non-blocking movement commands
//!
//! Run with: cargo run --example character_movement

use narrative_core::config::UserSettings;
use narrative_engine::EngineConfig;
use narrative_game::components::GameRootElement;
use narrative_gui::framework::{App, PresentMode, WindowOptions};

fn main() -> anyhow::Result<()> {
    // Initialize logging with more verbose output for examples
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
        )
        .init();

    tracing::info!("Starting Character Movement Demo Example");

    // Load user settings to get display resolution
    let (width, height) = match UserSettings::load("assets/config/settings.ron") {
        Ok(settings) => {
            tracing::info!(
                "Loaded display settings: resolution = {}x{}, fullscreen = {}",
                settings.display.resolution.0,
                settings.display.resolution.1,
                settings.display.fullscreen
            );
            settings.display.resolution
        }
        Err(e) => {
            tracing::warn!(
                "Could not load user settings, using default resolution 1280x720: {}",
                e
            );
            (1280, 720)
        }
    };

    // Create and run GUI application
    App::new(WindowOptions {
        title: "Character Movement Demo - Narrative Engine".to_string(),
        width,
        height,
        resizable: false,
        present_mode: PresentMode::VSync,
        target_fps: 60,
        show_fps_overlay: true, // Always show FPS in examples
        ..Default::default()
    })
    .with_root(move || {
        // Create engine configuration
        let mut config = EngineConfig::default();
        config.window.title = "Character Movement Demo".to_string();
        config.window.width = width;
        config.window.height = height;

        // Load the example character movement scenario
        config.start_scenario =
            std::path::PathBuf::from("assets/scenarios/examples/character_movement.toml");

        tracing::info!(
            "Loading scenario: {}",
            config.start_scenario.display()
        );

        Box::new(GameRootElement::new(config))
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
