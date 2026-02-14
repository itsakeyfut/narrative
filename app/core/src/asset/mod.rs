/// Asset management types and definitions
///
/// This module provides types for defining and managing game assets
/// using RON (Rusty Object Notation) manifest files, inspired by
/// the Veloren project's asset organization pattern.
pub mod background;
pub mod bgm;
pub mod se;
pub mod ui_theme;

pub use background::{BackgroundDef, BackgroundManifest, BackgroundMeta};
pub use bgm::{AudioMeta, BgmDef, BgmManifest};
pub use se::{SeDef, SeManifest};
pub use ui_theme::{UiThemeDef, UiThemeManifest};
