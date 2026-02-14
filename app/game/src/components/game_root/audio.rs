//! Audio management for GameRootElement

use super::element::GameRootElement;

impl GameRootElement {
    /// Start title screen BGM playback
    ///
    /// This should be called when entering the main menu.
    pub(super) fn start_title_bgm(&mut self) {
        // Reset BGM started flag when returning to title
        self.bgm_started = false;

        // Use title BGM path from config, or default to "assets/audio/music/title.ogg"
        let title_bgm_path = self
            .config
            .ui
            .title_bgm
            .as_deref()
            .unwrap_or("assets/audio/music/title.ogg");

        let mut audio = self.audio_manager.lock().unwrap_or_else(|e| {
            tracing::warn!("AudioManager mutex poisoned, recovering: {}", e);
            e.into_inner()
        });

        // Stop any currently playing BGM first
        if audio.is_bgm_playing()
            && let Err(e) = audio.stop_bgm(Some(0.5))
        {
            tracing::warn!("Failed to stop previous BGM: {}", e);
        }

        // Play title BGM with looping, fade-in, normal volume
        match audio.play_bgm(title_bgm_path, true, Some(1.0), 1.0) {
            Ok(_) => {
                tracing::info!("Title BGM playback started: {}", title_bgm_path);
            }
            Err(e) => {
                // Don't log error for missing title BGM - it's optional
                tracing::debug!("Title BGM not available (optional): {}", e);
            }
        }
    }

    /// Start BGM playback
    ///
    /// This should be called when the game starts (transitions to InGame state).
    /// Currently uses a hardcoded BGM path - in the future this should be read
    /// from the scenario configuration.
    pub(super) fn start_bgm(&mut self) {
        if self.bgm_started {
            return; // BGM already started
        }

        // FIXME: BGM path should be read from scenario configuration
        // Currently hardcoded to match assets/scenarios/chapter_01.toml
        let bgm_path = "assets/audio/music/dailylife/schooldays.ogg";

        let mut audio = self.audio_manager.lock().unwrap_or_else(|e| {
            tracing::warn!("AudioManager mutex poisoned, recovering: {}", e);
            e.into_inner()
        });

        // Stop any title BGM that might be playing
        if audio.is_bgm_playing()
            && let Err(e) = audio.stop_bgm(Some(0.5))
        {
            tracing::warn!("Failed to stop previous BGM: {}", e);
        }

        // Play BGM with looping, no fade-in, normal volume
        match audio.play_bgm(bgm_path, true, None, 1.0) {
            Ok(_) => {
                tracing::info!("BGM playback started: {}", bgm_path);
                self.bgm_started = true;
            }
            Err(e) => {
                tracing::error!("Failed to start BGM playback: {}", e);
            }
        }
    }
}
