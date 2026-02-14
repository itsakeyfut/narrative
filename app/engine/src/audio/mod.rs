//! Audio module
//!
//! This module provides audio playback using kira, including BGM, SE, and voice.

mod bgm;
mod manager;
mod se;
mod voice;

pub use bgm::BgmPlayer;
pub use manager::AudioManager;
pub use se::SePlayer;
pub use voice::VoicePlayer;
