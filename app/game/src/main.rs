//! Narrative Novel Engine - Main Application
//!
//! This is the main entry point for the Narrative Novel Engine application.

use narrative_core::config::UserSettings;
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

    tracing::info!("Starting Narrative Novel Engine");

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
            (1280, 720) // Default to 720p
        }
    };

    // Create and run GUI application
    // AudioManager is now initialized inside GameRootElement
    App::new(WindowOptions {
        title: "Narrative Novel Engine".to_string(),
        width,
        height,
        resizable: false, // Disable window resizing to maintain aspect ratio and layout
        present_mode: PresentMode::VSync,
        target_fps: 60,
        show_fps_overlay: cfg!(debug_assertions),
        ..Default::default()
    })
    .with_root(move || {
        // Create engine configuration with user-selected resolution
        let mut config = EngineConfig::default();
        config.window.title = "Narrative Novel Engine".to_string();
        config.window.width = width;
        config.window.height = height;

        // Create root element
        Box::new(GameRootElement::new(config))
    })
    .on_window_created(|window| {
        // Load default game assets after window creation
        match window.load_default_assets() {
            Ok((bg_id, char_id)) => {
                // Set texture IDs in GameRootElement
                if let Some(root) = window.root_element_mut()
                    && let Some(game_root) = root.as_any_mut().downcast_mut::<GameRootElement>()
                {
                    game_root.set_texture_ids(bg_id, char_id);
                    tracing::info!("Successfully set texture IDs in GameRootElement");
                }
            }
            Err(e) => {
                tracing::error!("Failed to load default assets: {}", e);
                // Continue anyway - game can run without textures
            }
        }
    })
    .run()?;

    Ok(())
}
