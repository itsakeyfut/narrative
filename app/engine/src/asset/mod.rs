//! Asset management module
//!
//! This module provides asset loading and caching.

mod cache;
mod handle;
mod loader;
mod registry;

#[cfg(feature = "hot-reload")]
mod hot_reload;

pub use cache::TextureCache;
pub use handle::TextureHandle;
pub use loader::{AssetLoader, AssetStats};
pub use registry::{
    AssetRegistry, BackgroundRegistry, BgmRegistry, RegistryStats, SeRegistry, UiThemeRegistry,
};

#[cfg(feature = "hot-reload")]
pub use hot_reload::{HotReloadWatcher, ReloadEvent};
