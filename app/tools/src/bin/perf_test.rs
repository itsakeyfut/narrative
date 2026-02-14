//! Performance Test Binary
//!
//! Issue #111: Performance testing runner
//!
//! This binary runs the application with performance monitoring enabled.
//! It loads the performance test scenario (200+ dialogue entries) and enables
//! detailed FPS overlay for manual validation.
//!
//! Manual testing procedure:
//! 1. Run: `cargo run --bin perf-test`
//! 2. Observe FPS overlay (top-left corner)
//! 3. Click through dialogue to test typewriter effect
//! 4. Monitor metrics during different game states:
//!    - Idle state
//!    - Dialogue display
//!    - Typewriter effect
//!    - Choice menu
//!    - Long dialogues (100+ lines)
//! 5. Run for at least 30 minutes to check memory stability
//!
//! Acceptance criteria (from Issue #111):
//! - FPS: stable 60 FPS in all states
//! - P95 frame time: < 16.67ms
//! - Layout time: < 2ms
//! - Paint time: < 3ms
//! - GPU submit: < 11ms
//! - No frame drops during typewriter effect
//! - No memory leaks after 30 minutes
//!
//! Usage:
//!   cargo run --bin perf-test                 # Run performance test
//!   cargo run --bin perf-test --release       # Run in release mode for accurate metrics

use anyhow::Result;
use narrative_engine::EngineConfig;
use narrative_game::components::GameRootElement;
use narrative_gui::framework::{App, PresentMode, WindowOptions};
use tracing::info;

fn main() -> Result<()> {
    // Initialize logging with detailed output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Print test instructions
    println!("{}", "=".repeat(80));
    println!("PERFORMANCE TEST - Issue #111");
    println!("{}", "=".repeat(80));
    println!();
    println!("This test validates the following acceptance criteria:");
    println!("  1. Stable 60 FPS in all game states");
    println!("  2. P95 frame time < 16.67ms");
    println!("  3. Layout time < 2ms");
    println!("  4. Paint time < 3ms");
    println!("  5. GPU submit time < 11ms");
    println!("  6. No frame drops during typewriter effect");
    println!("  7. No memory leaks after 30 minutes");
    println!();
    println!("INSTRUCTIONS:");
    println!("  - FPS overlay is displayed in the top-left corner");
    println!("  - Click through dialogue to test different game states");
    println!("  - Monitor metrics during typewriter effect");
    println!("  - Press ESC to exit");
    println!("  - For memory leak testing, run for at least 30 minutes");
    println!();
    println!("METRICS TO MONITOR:");
    println!("  FPS       - Should stay at ~60");
    println!("  Frame     - Average frame time (should be < 16.67ms)");
    println!("  Layout    - Layout computation time (should be < 2ms)");
    println!("  Paint     - Paint phase time (should be < 3ms)");
    println!("  Draws     - Number of draw calls per frame");
    println!();
    println!("{}", "=".repeat(80));
    println!();

    info!("Starting Performance Test");
    info!("Loading performance_test.toml scenario with 200+ dialogue entries");

    // Create and run GUI application with performance monitoring enabled
    App::new(WindowOptions {
        title: "Performance Test - Issue #111 [CLOSE WITH ESC]".to_string(),
        width: 1280,
        height: 720,
        present_mode: PresentMode::VSync,
        target_fps: 60,
        show_fps_overlay: true, // Always show FPS overlay for performance testing
        ..Default::default()
    })
    .with_root(|| {
        // Create engine configuration
        let mut config = EngineConfig::default();
        config.window.title = "Performance Test".to_string();

        // Create root element with performance test scenario (200+ dialogue entries)
        Box::new(GameRootElement::with_scenario(
            config,
            "assets/scenarios/performance_test.toml",
        ))
    })
    .on_window_created(|window| {
        // Load default game assets
        match window.load_default_assets() {
            Ok((bg_id, char_id)) => {
                // Set texture IDs in GameRootElement
                if let Some(root) = window.root_element_mut()
                    && let Some(game_root) = root.as_any_mut().downcast_mut::<GameRootElement>()
                {
                    game_root.set_texture_ids(bg_id, char_id);
                    info!("Assets loaded successfully");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load assets: {}", e);
            }
        }

        info!("Performance test running. Monitor FPS overlay for metrics.");
        info!("Press ESC to exit when testing is complete.");
    })
    .run()?;

    info!("Performance test completed");
    println!("\nPerformance test session ended.");
    println!("Review the FPS metrics observed during the test to validate acceptance criteria.");

    Ok(())
}
