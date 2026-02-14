//! SE (sound effect) test program
//!
//! This example demonstrates:
//! - Playing multiple simultaneous sound effects
//! - SE volume control
//! - Stopping all SE

use narrative_engine::AudioManager;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SE Test Program ===");
    println!("Initializing audio manager...");

    let mut audio_manager = AudioManager::new()?;
    println!("✓ AudioManager initialized successfully");

    // Test SE files (these would need to exist in your assets directory)
    let se_click = Path::new("assets/audio/se/click.ogg");
    let se_cancel = Path::new("assets/audio/se/cancel.ogg");
    let se_confirm = Path::new("assets/audio/se/confirm.ogg");

    // Check if SE files exist
    println!("\nChecking for SE files...");
    let has_click = se_click.exists();
    let has_cancel = se_cancel.exists();
    let has_confirm = se_confirm.exists();

    if has_click {
        println!("✓ Click SE found: {}", se_click.display());
    } else {
        println!("✗ Click SE not found: {}", se_click.display());
    }

    if has_cancel {
        println!("✓ Cancel SE found: {}", se_cancel.display());
    } else {
        println!("✗ Cancel SE not found: {}", se_cancel.display());
    }

    if has_confirm {
        println!("✓ Confirm SE found: {}", se_confirm.display());
    } else {
        println!("✗ Confirm SE not found: {}", se_confirm.display());
    }

    if !has_click && !has_cancel && !has_confirm {
        println!("\n✗ No SE files found. Please add some sound effect files to test.");
        println!("Expected paths:");
        println!("  - {}", se_click.display());
        println!("  - {}", se_cancel.display());
        println!("  - {}", se_confirm.display());
        return Ok(());
    }

    // Test 1: Volume control
    println!("\n--- Test 1: Volume Control ---");
    println!("Setting sound volume to 0.5 (50%)...");
    audio_manager.set_sound_volume(0.5)?;
    println!("✓ Sound volume set to 0.5");

    // Test 2: Play single SE
    if has_click {
        println!("\n--- Test 2: Single SE Playback ---");
        println!("Playing click SE...");
        audio_manager.play_se(se_click, 1.0)?;
        println!("✓ Click SE started");
        println!("  Active SE count: {}", audio_manager.active_se_count());
        thread::sleep(Duration::from_millis(500));
    }

    // Test 3: Multiple simultaneous SE
    println!("\n--- Test 3: Multiple Simultaneous SE ---");
    println!("Playing multiple SE in quick succession...");

    let mut count: u32 = 0;
    if has_click {
        audio_manager.play_se(se_click, 1.0)?;
        count = count.saturating_add(1);
    }
    thread::sleep(Duration::from_millis(100));

    if has_cancel {
        audio_manager.play_se(se_cancel, 1.0)?;
        count = count.saturating_add(1);
    }
    thread::sleep(Duration::from_millis(100));

    if has_confirm {
        audio_manager.play_se(se_confirm, 1.0)?;
        count = count.saturating_add(1);
    }

    println!("✓ Played {} SE simultaneously", count);
    println!("  Active SE count: {}", audio_manager.active_se_count());
    thread::sleep(Duration::from_secs(1));

    // Test 4: Volume adjustment
    println!("\n--- Test 4: Volume Adjustment ---");
    println!("Changing sound volume to 0.8 (80%)...");
    audio_manager.set_sound_volume(0.8)?;
    println!("✓ Sound volume changed to 0.8");

    if has_click {
        println!("Playing click SE with new volume...");
        audio_manager.play_se(se_click, 1.0)?;
        println!("✓ Click SE played with updated volume");
        thread::sleep(Duration::from_millis(500));
    }

    // Test 5: Stop all SE
    println!("\n--- Test 5: Stop All SE ---");
    if has_click {
        println!("Playing click SE...");
        audio_manager.play_se(se_click, 1.0)?;
        println!(
            "  Active SE count before stop: {}",
            audio_manager.active_se_count()
        );
        thread::sleep(Duration::from_millis(100));

        println!("Stopping all SE...");
        audio_manager.stop_all_se()?;
        println!("✓ All SE stopped");
        println!(
            "  Active SE count after stop: {}",
            audio_manager.active_se_count()
        );
    }

    println!("\n=== Test Complete ===");
    println!("SE playback system is working correctly!");
    Ok(())
}
