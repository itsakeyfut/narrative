//! Voice player

use narrative_core::AudioId;

/// Voice player
pub struct VoicePlayer {
    // Will be populated with kira in Phase 3.3
}

impl VoicePlayer {
    /// Create a new voice player (stub implementation)
    pub fn new() -> Self {
        Self {}
    }

    /// Play voice
    pub fn play(&mut self, _audio_id: &AudioId) {
        // TODO: Phase 3.3 - voice support
    }

    /// Stop voice
    pub fn stop(&mut self) {
        // TODO: Phase 3.3 - voice support
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&mut self, _volume: f32) {
        // TODO: Phase 3.3 - voice support
    }
}

impl Default for VoicePlayer {
    fn default() -> Self {
        Self::new()
    }
}
