//! Audio manager

use super::{BgmPlayer, SePlayer, VoicePlayer};
use crate::app::AudioConfig;
use crate::error::{EngineError, EngineResult};
use kira::AudioManager as KiraAudioManager;

/// Central audio manager
pub struct AudioManager {
    kira_manager: Option<KiraAudioManager>,
    bgm: BgmPlayer,
    se: SePlayer,
    voice: VoicePlayer,
    config: AudioConfig,
}

impl AudioManager {
    /// Create a new audio manager with kira integration
    pub fn new() -> EngineResult<Self> {
        Self::with_config(AudioConfig::default())
    }

    /// Create a new audio manager with the given configuration
    pub fn with_config(config: AudioConfig) -> EngineResult<Self> {
        let kira_manager = KiraAudioManager::new(Default::default())
            .map_err(|e| EngineError::AudioInit(format!("Failed to initialize kira: {:?}", e)))?;

        let mut manager = Self {
            kira_manager: Some(kira_manager),
            bgm: BgmPlayer::new(),
            se: SePlayer::new(),
            voice: VoicePlayer::new(),
            config: config.clone(),
        };

        // Apply initial volumes from config
        manager.apply_volumes()?;

        Ok(manager)
    }

    /// Create a disabled audio manager (audio operations will return errors)
    ///
    /// This is useful when audio initialization fails but the application
    /// should continue running without audio.
    pub fn disabled() -> Self {
        Self {
            kira_manager: None,
            bgm: BgmPlayer::new(),
            se: SePlayer::new(),
            voice: VoicePlayer::new(),
            config: AudioConfig::default(),
        }
    }

    /// Get kira audio manager reference
    ///
    /// This is currently unused as audio players receive the manager directly,
    /// but is kept for potential future use or alternative implementations.
    #[allow(dead_code)]
    pub(super) fn kira(&mut self) -> EngineResult<&mut KiraAudioManager> {
        self.kira_manager
            .as_mut()
            .ok_or_else(|| EngineError::AudioInit("Audio is disabled".to_string()))
    }

    /// Get BGM player
    pub fn bgm(&mut self) -> &mut BgmPlayer {
        &mut self.bgm
    }

    /// Get SE player
    pub fn se(&mut self) -> &mut SePlayer {
        &mut self.se
    }

    /// Get voice player
    pub fn voice(&mut self) -> &mut VoicePlayer {
        &mut self.voice
    }

    /// Play BGM with direct access to both player and manager
    ///
    /// # Arguments
    /// * `path` - Path to the audio file
    /// * `loop_enabled` - Whether to loop the BGM
    /// * `fade_in_duration` - Optional fade-in duration in seconds
    /// * `volume_multiplier` - Volume multiplier for this playback (1.0 = use config volume)
    pub fn play_bgm(
        &mut self,
        path: impl AsRef<std::path::Path>,
        loop_enabled: bool,
        fade_in_duration: Option<f64>,
        volume_multiplier: f32,
    ) -> EngineResult<()> {
        let kira = self.kira_manager.as_mut().ok_or_else(|| {
            EngineError::AudioInit("Audio is disabled - cannot play BGM".to_string())
        })?;

        // Calculate effective volume (config volume * multiplier)
        let effective_volume = self.config.effective_music_volume() * volume_multiplier;

        // Set the volume before playing
        self.bgm.set_volume(effective_volume, None)?;

        self.bgm.play(kira, path, loop_enabled, fade_in_duration)
    }

    /// Stop BGM playback
    ///
    /// # Arguments
    /// * `fade_out_duration` - Optional fade-out duration in seconds
    pub fn stop_bgm(&mut self, fade_out_duration: Option<f64>) -> EngineResult<()> {
        self.bgm.stop(fade_out_duration)
    }

    /// Pause BGM playback
    pub fn pause_bgm(&mut self, fade_out_duration: Option<f64>) -> EngineResult<()> {
        self.bgm.pause(fade_out_duration)
    }

    /// Resume BGM playback
    pub fn resume_bgm(&mut self, fade_in_duration: Option<f64>) -> EngineResult<()> {
        self.bgm.resume(fade_in_duration)
    }

    /// Check if BGM is currently playing
    pub fn is_bgm_playing(&self) -> bool {
        self.bgm.is_playing()
    }

    /// Play SE with direct access to both player and manager
    ///
    /// # Arguments
    /// * `path` - Path to the audio file
    /// * `volume_multiplier` - Volume multiplier for this playback (1.0 = use config volume)
    pub fn play_se(
        &mut self,
        path: impl AsRef<std::path::Path>,
        volume_multiplier: f32,
    ) -> EngineResult<()> {
        let kira = self.kira_manager.as_mut().ok_or_else(|| {
            EngineError::AudioInit("Audio is disabled - cannot play SE".to_string())
        })?;

        // Calculate effective volume (config volume * multiplier)
        let effective_volume = self.config.effective_sound_volume() * volume_multiplier;

        // Set the volume before playing
        self.se.set_volume(effective_volume)?;

        self.se.play(kira, path)
    }

    /// Stop all currently playing SE
    pub fn stop_all_se(&mut self) -> EngineResult<()> {
        self.se.stop_all()
    }

    /// Get the number of currently active SE
    pub fn active_se_count(&self) -> usize {
        self.se.active_count()
    }

    /// Get the current audio configuration
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// Update the audio configuration and apply volumes
    pub fn update_config(&mut self, config: AudioConfig) -> EngineResult<()> {
        self.config = config;
        self.apply_volumes()
    }

    /// Apply volumes from config to all players
    fn apply_volumes(&mut self) -> EngineResult<()> {
        self.bgm
            .set_volume(self.config.effective_music_volume(), None)?;
        self.se.set_volume(self.config.effective_sound_volume())?;
        // Voice player volumes will be applied when voice playback is implemented
        Ok(())
    }

    /// Set master volume (0.0-1.0) and apply to all categories
    pub fn set_master_volume(&mut self, volume: f32) -> EngineResult<()> {
        self.config.set_master_volume(volume);
        self.apply_volumes()
    }

    /// Set music volume (0.0-1.0) and apply
    pub fn set_music_volume(&mut self, volume: f32) -> EngineResult<()> {
        self.config.set_music_volume(volume);
        self.bgm
            .set_volume(self.config.effective_music_volume(), None)
    }

    /// Set sound effects volume (0.0-1.0) and apply
    pub fn set_sound_volume(&mut self, volume: f32) -> EngineResult<()> {
        self.config.set_sound_volume(volume);
        self.se.set_volume(self.config.effective_sound_volume())
    }

    /// Set voice volume (0.0-1.0)
    pub fn set_voice_volume(&mut self, volume: f32) -> EngineResult<()> {
        self.config.set_voice_volume(volume);
        // Voice player volumes will be applied when voice playback is implemented
        Ok(())
    }

    /// Toggle mute (enable/disable all audio)
    pub fn toggle_mute(&mut self) -> EngineResult<()> {
        self.config.toggle_mute();
        self.apply_volumes()
    }

    /// Set mute state
    pub fn set_mute(&mut self, muted: bool) -> EngineResult<()> {
        self.config.set_mute(muted);
        self.apply_volumes()
    }

    /// Check if audio is muted
    pub fn is_muted(&self) -> bool {
        self.config.is_muted()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_manager_new() {
        let manager = AudioManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_audio_manager_bgm_player_access() {
        let mut manager = AudioManager::new().unwrap();
        let bgm = manager.bgm();
        assert!(!bgm.is_playing());
    }

    #[test]
    fn test_audio_manager_is_bgm_playing_initial() {
        let manager = AudioManager::new().unwrap();
        assert!(!manager.is_bgm_playing());
    }

    #[test]
    fn test_audio_manager_set_music_volume() {
        let mut manager = AudioManager::new().unwrap();
        assert!(manager.set_music_volume(0.5).is_ok());
        assert_eq!(manager.config().music_volume, 0.5);
    }

    #[test]
    fn test_audio_manager_stop_bgm_when_not_playing() {
        let mut manager = AudioManager::new().unwrap();
        assert!(manager.stop_bgm(None).is_ok());
    }

    #[test]
    fn test_audio_manager_pause_bgm_when_not_playing() {
        let mut manager = AudioManager::new().unwrap();
        assert!(manager.pause_bgm(None).is_ok());
    }

    #[test]
    fn test_audio_manager_resume_bgm_when_not_playing() {
        let mut manager = AudioManager::new().unwrap();
        assert!(manager.resume_bgm(None).is_ok());
    }

    #[test]
    fn test_audio_manager_se_player_access() {
        let mut manager = AudioManager::new().unwrap();
        let se = manager.se();
        assert_eq!(se.active_count(), 0);
    }

    #[test]
    fn test_audio_manager_set_sound_volume() {
        let mut manager = AudioManager::new().unwrap();
        assert!(manager.set_sound_volume(0.5).is_ok());
        assert_eq!(manager.config().sound_volume, 0.5);
    }

    #[test]
    fn test_audio_manager_stop_all_se_when_empty() {
        let mut manager = AudioManager::new().unwrap();
        assert!(manager.stop_all_se().is_ok());
    }

    #[test]
    fn test_audio_manager_active_se_count_initial() {
        let manager = AudioManager::new().unwrap();
        assert_eq!(manager.active_se_count(), 0);
    }
}
