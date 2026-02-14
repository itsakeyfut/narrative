//! Simple audio test program

use narrative_engine::AudioManager;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Audio Test Program ===");
    println!("Initializing audio manager...");

    let mut audio_manager = AudioManager::new()?;
    println!("✓ AudioManager initialized successfully");

    let bgm_path = Path::new("assets/audio/music/dailylife/schooldays.ogg");

    if !bgm_path.exists() {
        eprintln!("✗ BGM file not found: {}", bgm_path.display());
        return Err("BGM file not found".into());
    }

    println!("✓ BGM file exists: {}", bgm_path.display());

    println!("Playing BGM without fade-in...");
    audio_manager.play_bgm(bgm_path, true, None, 1.0)?;
    println!("✓ BGM playback started");
    println!("  Is playing: {}", audio_manager.is_bgm_playing());

    println!("\nPlaying for 10 seconds...");
    println!("If you can hear the music, the audio system is working correctly.");
    println!("Press Ctrl+C to stop early.\n");

    for i in 1..=10 {
        thread::sleep(Duration::from_secs(1));
        println!(
            "  {} seconds elapsed - Is playing: {}",
            i,
            audio_manager.is_bgm_playing()
        );
    }

    println!("\nStopping BGM...");
    audio_manager.stop_bgm(None)?;
    println!("✓ BGM stopped");

    println!("\n=== Test Complete ===");
    Ok(())
}
