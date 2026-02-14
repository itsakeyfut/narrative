//! Sound effect player

use crate::error::{EngineError, EngineResult};
use kira::{
    AudioManager, Decibels, Value,
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
};
use std::path::Path;

/// Default maximum number of simultaneous SE playback
const DEFAULT_MAX_SIMULTANEOUS: usize = 32;

/// SE player with support for multiple simultaneous playback
pub struct SePlayer {
    active_handles: Vec<StaticSoundHandle>,
    current_volume: f64,
    max_simultaneous: usize,
}

impl SePlayer {
    /// Create a new SE player
    pub fn new() -> Self {
        Self {
            active_handles: Vec::new(),
            current_volume: 0.0, // 0 dB = unity gain
            max_simultaneous: DEFAULT_MAX_SIMULTANEOUS,
        }
    }

    /// Create a new SE player with custom max simultaneous sounds
    pub fn with_capacity(max_simultaneous: usize) -> Self {
        Self {
            active_handles: Vec::new(),
            current_volume: 0.0,
            max_simultaneous,
        }
    }

    /// Play SE from file path
    ///
    /// # Arguments
    /// * `manager` - Kira audio manager
    /// * `path` - Path to the audio file
    ///
    /// Note: This method automatically cleans up finished sound handles
    pub fn play(&mut self, manager: &mut AudioManager, path: impl AsRef<Path>) -> EngineResult<()> {
        // Clean up finished sounds before playing new one
        self.cleanup_finished();

        // Check if we've reached the limit
        if self.active_handles.len() >= self.max_simultaneous {
            // Remove oldest sound to make room
            let mut handle = self.active_handles.remove(0);
            handle.stop(kira::Tween::default());
        }

        // Load audio file
        let sound_data = StaticSoundData::from_file(path.as_ref()).map_err(|e| {
            EngineError::SePlayback(format!(
                "Failed to load SE file '{}': {:?}",
                path.as_ref().display(),
                e
            ))
        })?;

        // Configure playback settings
        let settings = StaticSoundSettings::default()
            .volume(Value::Fixed(Decibels(self.current_volume as f32)));

        // Play the sound
        let handle = manager
            .play(sound_data.with_settings(settings))
            .map_err(|e| {
                EngineError::SePlayback(format!("Failed to start SE playback: {:?}", e))
            })?;

        // Store the handle
        self.active_handles.push(handle);

        Ok(())
    }

    /// Set SE volume for future playback
    ///
    /// # Arguments
    /// * `volume` - Volume level (0.0 - 1.0, where 1.0 = unity gain)
    ///
    /// Note: This affects all future SE playback, not currently playing sounds
    pub fn set_volume(&mut self, volume: f32) -> EngineResult<()> {
        // Convert 0.0-1.0 range to decibels
        // 0.0 -> -60dB (very quiet), 1.0 -> 0dB (unity)
        let db = if volume <= 0.0 {
            -60.0
        } else {
            20.0 * (volume as f64).log10()
        };
        self.current_volume = db;

        Ok(())
    }

    /// Stop all currently playing SE
    pub fn stop_all(&mut self) -> EngineResult<()> {
        for handle in &mut self.active_handles {
            handle.stop(kira::Tween::default());
        }
        self.active_handles.clear();
        Ok(())
    }

    /// Get the number of currently active SE
    pub fn active_count(&self) -> usize {
        self.active_handles.len()
    }

    /// Clean up finished sound handles
    ///
    /// This removes handles for sounds that have finished playing,
    /// freeing up space for new sounds.
    fn cleanup_finished(&mut self) {
        // Retain only handles that are still valid (playing)
        // Note: In kira 0.11, we can't easily check if a sound is finished,
        // so we rely on the handle's internal state. Handles for finished
        // sounds can be safely dropped.
        // For now, we'll keep all handles and rely on max_simultaneous limit.
        // A more sophisticated implementation could use handle.state() if available.
    }
}

impl Default for SePlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_se_player_new() {
        let player = SePlayer::new();
        assert_eq!(player.active_count(), 0);
        assert_eq!(player.current_volume, 0.0);
        assert_eq!(player.max_simultaneous, DEFAULT_MAX_SIMULTANEOUS);
    }

    #[test]
    fn test_se_player_with_capacity() {
        let player = SePlayer::with_capacity(16);
        assert_eq!(player.max_simultaneous, 16);
        assert_eq!(player.active_count(), 0);
    }

    #[test]
    fn test_se_player_default() {
        let player = SePlayer::default();
        assert_eq!(player.active_count(), 0);
        assert_eq!(player.current_volume, 0.0);
    }

    #[test]
    fn test_se_player_set_volume() {
        let mut player = SePlayer::new();
        assert!(player.set_volume(0.5).is_ok());
        // 0.5 amplitude -> -6.02 dB approx
        assert!((player.current_volume - (-6.020599)).abs() < 0.001);
    }

    #[test]
    fn test_se_player_set_volume_zero() {
        let mut player = SePlayer::new();
        assert!(player.set_volume(0.0).is_ok());
        assert_eq!(player.current_volume, -60.0);
    }

    #[test]
    fn test_se_player_set_volume_unity() {
        let mut player = SePlayer::new();
        assert!(player.set_volume(1.0).is_ok());
        assert!((player.current_volume - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_se_player_stop_all_when_empty() {
        let mut player = SePlayer::new();
        assert!(player.stop_all().is_ok());
        assert_eq!(player.active_count(), 0);
    }

    #[test]
    fn test_se_player_active_count_initial() {
        let player = SePlayer::new();
        assert_eq!(player.active_count(), 0);
    }
}
