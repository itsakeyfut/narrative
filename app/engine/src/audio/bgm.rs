//! BGM player

use crate::error::{EngineError, EngineResult};
use kira::{
    AudioManager, Decibels, Tween, Value,
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
};
use std::{path::Path, time::Duration};

/// BGM player with kira integration
pub struct BgmPlayer {
    current_handle: Option<StaticSoundHandle>,
    current_volume: f64,
}

impl BgmPlayer {
    /// Create a new BGM player
    pub fn new() -> Self {
        Self {
            current_handle: None,
            current_volume: 0.0, // 0 dB = unity gain
        }
    }

    /// Play BGM from file path
    ///
    /// # Arguments
    /// * `manager` - Kira audio manager
    /// * `path` - Path to the audio file
    /// * `loop_enabled` - Whether to loop the BGM
    /// * `fade_in_duration` - Optional fade-in duration in seconds
    pub fn play(
        &mut self,
        manager: &mut AudioManager,
        path: impl AsRef<Path>,
        loop_enabled: bool,
        fade_in_duration: Option<f64>,
    ) -> EngineResult<()> {
        // Stop current BGM if playing
        if self.current_handle.is_some() {
            self.stop(None)?;
        }

        // Load audio file
        let sound_data = StaticSoundData::from_file(path.as_ref()).map_err(|e| {
            EngineError::BgmPlayback(format!(
                "Failed to load BGM file '{}': {:?}",
                path.as_ref().display(),
                e
            ))
        })?;

        // Configure playback settings
        let mut settings = StaticSoundSettings::default();

        // Enable looping if requested
        if loop_enabled {
            settings = settings.loop_region(..);
        }

        // Set volume with fade-in if requested
        if let Some(duration) = fade_in_duration {
            // Use fade_in_tween for volume fade-in
            settings.fade_in_tween = Some(Tween {
                start_time: kira::StartTime::Immediate,
                duration: Duration::from_secs_f64(duration),
                easing: kira::Easing::Linear,
            });
            // Set target volume (fade_in_tween will fade from 0 to this)
            settings.volume = Value::Fixed(Decibels(self.current_volume as f32));
        } else {
            // No fade-in, use current volume directly
            settings.volume = Value::Fixed(Decibels(self.current_volume as f32));
        }

        // Play the sound
        let handle = manager
            .play(sound_data.with_settings(settings))
            .map_err(|e| {
                EngineError::BgmPlayback(format!("Failed to start BGM playback: {:?}", e))
            })?;

        self.current_handle = Some(handle);
        Ok(())
    }

    /// Stop BGM playback
    ///
    /// # Arguments
    /// * `fade_out_duration` - Optional fade-out duration in seconds
    pub fn stop(&mut self, fade_out_duration: Option<f64>) -> EngineResult<()> {
        if let Some(mut handle) = self.current_handle.take() {
            if let Some(duration) = fade_out_duration {
                // Fade out then stop
                let tween = Tween {
                    start_time: kira::StartTime::Immediate,
                    duration: Duration::from_secs_f64(duration),
                    easing: kira::Easing::Linear,
                };
                handle.set_volume(Decibels(-60.0), tween);
                // Note: In kira 0.11, we can't wait for the tween to complete easily
                // The handle will be dropped and sound will stop when it goes out of scope
            } else {
                // Immediate stop
                handle.stop(Tween::default());
            }
        }
        Ok(())
    }

    /// Set BGM volume
    ///
    /// # Arguments
    /// * `volume` - Volume level (0.0 - 1.0, where 1.0 = unity gain)
    /// * `tween_duration` - Optional duration for volume change in seconds
    pub fn set_volume(&mut self, volume: f32, tween_duration: Option<f64>) -> EngineResult<()> {
        // Convert 0.0-1.0 range to decibels
        // 0.0 -> -60dB (very quiet), 1.0 -> 0dB (unity)
        let db = if volume <= 0.0 {
            -60.0
        } else {
            20.0 * (volume as f64).log10()
        };
        self.current_volume = db;

        if let Some(handle) = &mut self.current_handle {
            let tween = if let Some(duration) = tween_duration {
                Tween {
                    start_time: kira::StartTime::Immediate,
                    duration: Duration::from_secs_f64(duration),
                    easing: kira::Easing::Linear,
                }
            } else {
                Tween::default()
            };

            handle.set_volume(Decibels(db as f32), tween);
        }

        Ok(())
    }

    /// Check if BGM is currently playing
    pub fn is_playing(&self) -> bool {
        self.current_handle.is_some()
    }

    /// Pause BGM playback
    pub fn pause(&mut self, fade_out_duration: Option<f64>) -> EngineResult<()> {
        if let Some(handle) = &mut self.current_handle {
            let tween = if let Some(duration) = fade_out_duration {
                Tween {
                    start_time: kira::StartTime::Immediate,
                    duration: Duration::from_secs_f64(duration),
                    easing: kira::Easing::Linear,
                }
            } else {
                Tween::default()
            };

            handle.pause(tween);
        }
        Ok(())
    }

    /// Resume BGM playback
    pub fn resume(&mut self, fade_in_duration: Option<f64>) -> EngineResult<()> {
        if let Some(handle) = &mut self.current_handle {
            let tween = if let Some(duration) = fade_in_duration {
                Tween {
                    start_time: kira::StartTime::Immediate,
                    duration: Duration::from_secs_f64(duration),
                    easing: kira::Easing::Linear,
                }
            } else {
                Tween::default()
            };

            handle.resume(tween);
        }
        Ok(())
    }
}

impl Default for BgmPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bgm_player_new() {
        let player = BgmPlayer::new();
        assert!(!player.is_playing());
        assert_eq!(player.current_volume, 0.0);
    }

    #[test]
    fn test_bgm_player_default() {
        let player = BgmPlayer::default();
        assert!(!player.is_playing());
        assert_eq!(player.current_volume, 0.0);
    }

    #[test]
    fn test_bgm_player_is_playing_initial() {
        let player = BgmPlayer::new();
        assert!(!player.is_playing());
    }

    #[test]
    fn test_bgm_player_stop_when_not_playing() {
        let mut player = BgmPlayer::new();
        // Should not error when stopping while nothing is playing
        assert!(player.stop(None).is_ok());
        assert!(player.stop(Some(1.0)).is_ok());
    }

    #[test]
    fn test_bgm_player_set_volume_when_not_playing() {
        let mut player = BgmPlayer::new();
        // Should update volume even when not playing
        assert!(player.set_volume(0.5, None).is_ok());
        // 0.5 amplitude -> -6.02 dB approx
        assert!((player.current_volume - (-6.020599)).abs() < 0.001);
    }

    #[test]
    fn test_bgm_player_set_volume_zero() {
        let mut player = BgmPlayer::new();
        assert!(player.set_volume(0.0, None).is_ok());
        assert_eq!(player.current_volume, -60.0);
    }

    #[test]
    fn test_bgm_player_set_volume_unity() {
        let mut player = BgmPlayer::new();
        assert!(player.set_volume(1.0, None).is_ok());
        assert!((player.current_volume - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_bgm_player_pause_when_not_playing() {
        let mut player = BgmPlayer::new();
        // Should not error when pausing while nothing is playing
        assert!(player.pause(None).is_ok());
    }

    #[test]
    fn test_bgm_player_resume_when_not_playing() {
        let mut player = BgmPlayer::new();
        // Should not error when resuming while nothing is playing
        assert!(player.resume(None).is_ok());
    }
}
