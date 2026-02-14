use serde::{Deserialize, Serialize};

/// Audio configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub master_volume: f32,
    /// BGM volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub bgm_volume: f32,
    /// Sound effect volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub se_volume: f32,
    /// Voice volume (0.0 - 1.0)
    #[serde(default = "default_volume")]
    pub voice_volume: f32,
    /// Enable audio
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl AudioConfig {
    /// Create a new audio config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the effective BGM volume (master * bgm)
    pub fn effective_bgm_volume(&self) -> f32 {
        if self.enabled {
            self.master_volume * self.bgm_volume
        } else {
            0.0
        }
    }

    /// Get the effective SE volume (master * se)
    pub fn effective_se_volume(&self) -> f32 {
        if self.enabled {
            self.master_volume * self.se_volume
        } else {
            0.0
        }
    }

    /// Get the effective voice volume (master * voice)
    pub fn effective_voice_volume(&self) -> f32 {
        if self.enabled {
            self.master_volume * self.voice_volume
        } else {
            0.0
        }
    }

    /// Set master volume (clamped to 0.0-1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set BGM volume (clamped to 0.0-1.0)
    pub fn set_bgm_volume(&mut self, volume: f32) {
        self.bgm_volume = volume.clamp(0.0, 1.0);
    }

    /// Set SE volume (clamped to 0.0-1.0)
    pub fn set_se_volume(&mut self, volume: f32) {
        self.se_volume = volume.clamp(0.0, 1.0);
    }

    /// Set voice volume (clamped to 0.0-1.0)
    pub fn set_voice_volume(&mut self, volume: f32) {
        self.voice_volume = volume.clamp(0.0, 1.0);
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: default_volume(),
            bgm_volume: default_volume(),
            se_volume: default_volume(),
            voice_volume: default_volume(),
            enabled: default_true(),
        }
    }
}

fn default_volume() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config_new() {
        let config = AudioConfig::new();
        assert_eq!(config.master_volume, 1.0);
        assert_eq!(config.bgm_volume, 1.0);
        assert_eq!(config.se_volume, 1.0);
        assert_eq!(config.voice_volume, 1.0);
        assert!(config.enabled);
    }

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.master_volume, 1.0);
        assert_eq!(config.bgm_volume, 1.0);
        assert_eq!(config.se_volume, 1.0);
        assert_eq!(config.voice_volume, 1.0);
        assert!(config.enabled);
    }

    #[test]
    fn test_audio_config_effective_bgm_volume() {
        let mut config = AudioConfig::new();
        config.master_volume = 0.8;
        config.bgm_volume = 0.5;
        assert_eq!(config.effective_bgm_volume(), 0.4);
    }

    #[test]
    fn test_audio_config_effective_bgm_volume_disabled() {
        let mut config = AudioConfig::new();
        config.master_volume = 0.8;
        config.bgm_volume = 0.5;
        config.enabled = false;
        assert_eq!(config.effective_bgm_volume(), 0.0);
    }

    #[test]
    fn test_audio_config_effective_se_volume() {
        let mut config = AudioConfig::new();
        config.master_volume = 0.7;
        config.se_volume = 0.6;
        assert!((config.effective_se_volume() - 0.42).abs() < 0.001);
    }

    #[test]
    fn test_audio_config_effective_se_volume_disabled() {
        let mut config = AudioConfig::new();
        config.enabled = false;
        assert_eq!(config.effective_se_volume(), 0.0);
    }

    #[test]
    fn test_audio_config_effective_voice_volume() {
        let mut config = AudioConfig::new();
        config.master_volume = 0.9;
        config.voice_volume = 0.8;
        assert!((config.effective_voice_volume() - 0.72).abs() < 0.001);
    }

    #[test]
    fn test_audio_config_effective_voice_volume_disabled() {
        let mut config = AudioConfig::new();
        config.enabled = false;
        assert_eq!(config.effective_voice_volume(), 0.0);
    }

    #[test]
    fn test_audio_config_set_master_volume() {
        let mut config = AudioConfig::new();
        config.set_master_volume(0.75);
        assert_eq!(config.master_volume, 0.75);
    }

    #[test]
    fn test_audio_config_set_master_volume_clamping_low() {
        let mut config = AudioConfig::new();
        config.set_master_volume(-0.5);
        assert_eq!(config.master_volume, 0.0);
    }

    #[test]
    fn test_audio_config_set_master_volume_clamping_high() {
        let mut config = AudioConfig::new();
        config.set_master_volume(1.5);
        assert_eq!(config.master_volume, 1.0);
    }

    #[test]
    fn test_audio_config_set_bgm_volume() {
        let mut config = AudioConfig::new();
        config.set_bgm_volume(0.6);
        assert_eq!(config.bgm_volume, 0.6);
    }

    #[test]
    fn test_audio_config_set_bgm_volume_clamping() {
        let mut config = AudioConfig::new();
        config.set_bgm_volume(2.0);
        assert_eq!(config.bgm_volume, 1.0);
    }

    #[test]
    fn test_audio_config_set_se_volume() {
        let mut config = AudioConfig::new();
        config.set_se_volume(0.4);
        assert_eq!(config.se_volume, 0.4);
    }

    #[test]
    fn test_audio_config_set_voice_volume() {
        let mut config = AudioConfig::new();
        config.set_voice_volume(0.85);
        assert_eq!(config.voice_volume, 0.85);
    }

    #[test]
    fn test_audio_config_serialization() {
        let config = AudioConfig::new();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: AudioConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_audio_config_all_volumes() {
        let mut config = AudioConfig::new();
        config.set_master_volume(0.8);
        config.set_bgm_volume(0.7);
        config.set_se_volume(0.6);
        config.set_voice_volume(0.5);

        assert_eq!(config.master_volume, 0.8);
        assert_eq!(config.bgm_volume, 0.7);
        assert_eq!(config.se_volume, 0.6);
        assert_eq!(config.voice_volume, 0.5);
    }
}
