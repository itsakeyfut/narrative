//! Animation Tests
//!
//! This test runs a simple animation test scenario.
//! It loads the animation.toml scenario which provides quick verification
//! of all animation types and timing modes.
//!
//! Run with: cargo run --example animation

use narrative_core::config::UserSettings;
use narrative_engine::EngineConfig;
use narrative_game::components::GameRootElement;
use narrative_gui::framework::{App, PresentMode, WindowOptions};

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
        )
        .init();

    tracing::info!("Starting Animation Test Example");

    // Load user settings
    let (width, height) = match UserSettings::load("assets/config/settings.ron") {
        Ok(settings) => settings.display.resolution,
        Err(_) => (1280, 720),
    };

    // Create and run GUI application
    App::new(WindowOptions {
        title: "Animation Test - Narrative Engine".to_string(),
        width,
        height,
        resizable: false,
        present_mode: PresentMode::VSync,
        target_fps: 60,
        show_fps_overlay: true,
        ..Default::default()
    })
    .with_root(move || {
        let mut config = EngineConfig::default();
        config.window.title = "Animation Test".to_string();
        config.window.width = width;
        config.window.height = height;

        // Load the test scenario
        config.start_scenario =
            std::path::PathBuf::from("assets/scenarios/tests/animation.toml");

        tracing::info!(
            "Loading test scenario: {}",
            config.start_scenario.display()
        );

        Box::new(GameRootElement::new(config))
    })
    .on_window_created(|window| {
        match window.load_default_assets() {
            Ok((bg_id, char_id)) => {
                if let Some(root) = window.root_element_mut()
                    && let Some(game_root) = root.as_any_mut().downcast_mut::<GameRootElement>()
                {
                    game_root.set_texture_ids(bg_id, char_id);
                    tracing::info!("Assets loaded successfully");
                }
            }
            Err(e) => {
                tracing::error!("Failed to load assets: {}", e);
            }
        }
    })
    .run()?;

    Ok(())
}
