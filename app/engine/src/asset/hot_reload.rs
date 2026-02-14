//! Hot-reload system for asset manifests
//!
//! This module provides file watching and automatic reloading of RON manifest files
//! when they change on disk. This is only available with the `hot-reload` feature.
//!
//! # Example
//!
//! ```rust,ignore
//! use narrative_engine::asset::{AssetRegistry, HotReloadWatcher, ReloadEvent};
//!
//! let mut registry = AssetRegistry::new("assets");
//! registry.load_all_manifests()?;
//!
//! let (watcher, rx) = HotReloadWatcher::new("assets/manifests")?;
//! watcher.start()?;
//!
//! // In your game loop:
//! while let Ok(event) = rx.try_recv() {
//!     match event {
//!         ReloadEvent::Characters => registry.characters.reload()?,
//!         ReloadEvent::Backgrounds => registry.backgrounds.load_manifest("manifests/backgrounds.ron")?,
//!         // ...
//!     }
//! }
//! ```

use crossbeam_channel::{Receiver, Sender, unbounded};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Errors that can occur during hot-reload operations
#[derive(Debug, Error)]
pub enum HotReloadError {
    #[error("Failed to create file watcher: {0}")]
    CreateFailed(#[from] notify::Error),

    #[error("Failed to watch path: {0}")]
    WatchFailed(String),

    #[error("Invalid manifest path: {0}")]
    InvalidPath(PathBuf),
}

/// Events emitted when a manifest file is modified
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadEvent {
    /// Character manifest was modified
    Characters,
    /// Background manifest was modified
    Backgrounds,
    /// BGM manifest was modified
    Bgm,
    /// Sound effect manifest was modified
    SoundEffects,
    /// UI theme manifest was modified
    UiThemes,
}

impl ReloadEvent {
    /// Get the expected filename for this reload event
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Characters => "characters.ron",
            Self::Backgrounds => "backgrounds.ron",
            Self::Bgm => "bgm.ron",
            Self::SoundEffects => "se.ron",
            Self::UiThemes => "ui_themes.ron",
        }
    }

    /// Try to determine reload event from a file path
    pub fn from_path(path: &Path) -> Option<Self> {
        let filename = path.file_name()?.to_str()?;
        match filename {
            "characters.ron" => Some(Self::Characters),
            "backgrounds.ron" => Some(Self::Backgrounds),
            "bgm.ron" => Some(Self::Bgm),
            "se.ron" => Some(Self::SoundEffects),
            "ui_themes.ron" => Some(Self::UiThemes),
            _ => None,
        }
    }
}

/// Debounce tracker to prevent reload storms
///
/// When a file is saved, editors often trigger multiple file system events
/// in quick succession. This tracker ensures we only reload once per file
/// within a given time window.
struct DebounceTracker {
    last_events: HashMap<PathBuf, SystemTime>,
    duration: Duration,
}

impl DebounceTracker {
    fn new(duration: Duration) -> Self {
        Self {
            last_events: HashMap::new(),
            duration,
        }
    }

    fn should_reload(&self, path: &Path) -> bool {
        if let Some(last_time) = self.last_events.get(path) {
            if let Ok(elapsed) = SystemTime::now().duration_since(*last_time) {
                return elapsed >= self.duration;
            }
        }
        true
    }

    fn mark_reloaded(&mut self, path: PathBuf) {
        self.last_events.insert(path, SystemTime::now());
    }
}

/// Hot-reload watcher for asset manifests
///
/// This watcher monitors a directory for changes to RON manifest files
/// and emits reload events when they are modified.
pub struct HotReloadWatcher {
    _watcher: RecommendedWatcher,
    path: PathBuf,
}

impl HotReloadWatcher {
    /// Create a new hot-reload watcher
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the manifests directory to watch
    ///
    /// # Returns
    ///
    /// Returns a tuple of (watcher, receiver). The receiver will emit
    /// ReloadEvent values when manifest files are modified.
    pub fn new(path: impl AsRef<Path>) -> Result<(Self, Receiver<ReloadEvent>), HotReloadError> {
        let path = path.as_ref().to_path_buf();
        let (tx, rx): (Sender<ReloadEvent>, Receiver<ReloadEvent>) = unbounded();

        // Create debounce tracker (500ms window) - wrapped in Arc<Mutex> for thread safety
        let debouncer = Arc::new(Mutex::new(DebounceTracker::new(Duration::from_millis(500))));
        let debouncer_clone = Arc::clone(&debouncer);

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // Only process Modify and Create events
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            // Check debounce
                            let should_reload = {
                                match debouncer_clone.lock() {
                                    Ok(debouncer) => debouncer.should_reload(&path),
                                    Err(e) => {
                                        tracing::error!("Debouncer mutex poisoned: {}", e);
                                        continue;
                                    }
                                }
                            };

                            if !should_reload {
                                continue;
                            }

                            // Check if it's a RON file
                            if path.extension().and_then(|s| s.to_str()) != Some("ron") {
                                continue;
                            }

                            // Determine which manifest was modified
                            if let Some(reload_event) = ReloadEvent::from_path(&path) {
                                tracing::info!("🔥 Detected change in {}", path.display());

                                // Mark as reloaded
                                match debouncer_clone.lock() {
                                    Ok(mut debouncer) => {
                                        debouncer.mark_reloaded(path.clone());
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to mark debounce: {}", e);
                                    }
                                }

                                let _ = tx.send(reload_event);
                            }
                        }
                    }
                }
            },
            Config::default(),
        )?;

        Ok((
            Self {
                _watcher: watcher,
                path,
            },
            rx,
        ))
    }

    /// Start watching the directory
    pub fn start(&mut self) -> Result<(), HotReloadError> {
        self._watcher
            .watch(&self.path, RecursiveMode::NonRecursive)
            .map_err(|e| HotReloadError::WatchFailed(e.to_string()))?;

        tracing::info!("🔥 Hot-reload watcher started for: {}", self.path.display());
        Ok(())
    }

    /// Get the path being watched
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reload_event_from_path() {
        assert_eq!(
            ReloadEvent::from_path(Path::new("characters.ron")),
            Some(ReloadEvent::Characters)
        );
        assert_eq!(
            ReloadEvent::from_path(Path::new("manifests/backgrounds.ron")),
            Some(ReloadEvent::Backgrounds)
        );
        assert_eq!(
            ReloadEvent::from_path(Path::new("bgm.ron")),
            Some(ReloadEvent::Bgm)
        );
        assert_eq!(
            ReloadEvent::from_path(Path::new("se.ron")),
            Some(ReloadEvent::SoundEffects)
        );
        assert_eq!(
            ReloadEvent::from_path(Path::new("ui_themes.ron")),
            Some(ReloadEvent::UiThemes)
        );
        assert_eq!(ReloadEvent::from_path(Path::new("other.ron")), None);
    }

    #[test]
    fn test_debounce_tracker() {
        let mut tracker = DebounceTracker::new(Duration::from_millis(100));
        let path = PathBuf::from("test.ron");

        // First check should always return true
        assert!(tracker.should_reload(&path));

        // Mark as reloaded
        tracker.mark_reloaded(path.clone());

        // Immediate check should return false
        assert!(!tracker.should_reload(&path));

        // Wait for debounce duration
        std::thread::sleep(Duration::from_millis(150));

        // Now should return true again
        assert!(tracker.should_reload(&path));
    }

    #[test]
    fn test_reload_event_filename() {
        assert_eq!(ReloadEvent::Characters.filename(), "characters.ron");
        assert_eq!(ReloadEvent::Backgrounds.filename(), "backgrounds.ron");
        assert_eq!(ReloadEvent::Bgm.filename(), "bgm.ron");
        assert_eq!(ReloadEvent::SoundEffects.filename(), "se.ron");
        assert_eq!(ReloadEvent::UiThemes.filename(), "ui_themes.ron");
    }
}
