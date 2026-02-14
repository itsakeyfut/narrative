//! Narrative GUI Library
//!
//! This crate provides a custom wgpu-based graphical user interface framework for
//! the Narrative Novel game engine editor.
//!
//! # Architecture
//!
//! The GUI uses a custom rendering framework inspired by Zed's GPUI:
//! - **framework**: Core abstractions (App, Window, Element, Renderer)
//! - **components**: Reusable UI widgets (buttons, cards, panels)
//! - **theme**: Shared color palette and styling constants
//!
//! # Key Features
//!
//! - GPU-first rendering via wgpu
//! - Taffy-based flexbox layout
//! - Reactive system with signals and effects
//! - Component-based architecture for reusability
//! - Performance monitoring and metrics

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod components;
pub mod framework;
pub mod theme;

// Re-export framework types
pub use framework::{
    Alignment, App, AppContext, AppMenu, Bounds, Color, Container, Element, ElementId,
    FlexDirection, FrameworkError, FrameworkResult, InputEvent, MenuEventHandler, MenuId, Point,
    PresentMode, Renderer, Size, Text, Window, WindowContext, WindowOperation, WindowOptions,
};

use thiserror::Error;

/// GUI-specific error types
#[derive(Error, Debug)]
pub enum GuiError {
    /// Failed to initialize the GUI
    #[error("GUI initialization failed: {0}")]
    InitializationFailed(String),

    /// Failed to load a resource
    #[error("Failed to load resource: {0}")]
    ResourceLoadFailed(String),

    /// Invalid UI state
    #[error("Invalid UI state: {0}")]
    InvalidState(String),

    /// Invalid ID format
    #[error("Invalid ID: {0}")]
    InvalidId(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Framework error
    #[error("Framework error: {0}")]
    Framework(#[from] FrameworkError),
}

/// Result type for GUI operations
pub type GuiResult<T> = Result<T, GuiError>;

/// GUI application configuration
#[derive(Debug, Clone)]
pub struct GuiConfig {
    /// Window title
    pub title: String,

    /// Window width
    pub width: u32,

    /// Window height
    pub height: u32,

    /// Whether to maximize window on startup
    pub maximize_on_startup: bool,

    /// Enable dark theme
    pub dark_theme: bool,

    /// Show FPS overlay
    pub show_fps_overlay: bool,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            title: "Narrative Novel Editor".to_string(),
            width: 1600,
            height: 900,
            maximize_on_startup: true,
            dark_theme: true,
            show_fps_overlay: false,
        }
    }
}

impl From<GuiConfig> for WindowOptions {
    fn from(config: GuiConfig) -> Self {
        WindowOptions {
            title: config.title,
            width: config.width,
            height: config.height,
            resizable: true,
            decorations: true,
            present_mode: PresentMode::VSync,
            target_fps: 60,
            show_fps_overlay: config.show_fps_overlay,
        }
    }
}
