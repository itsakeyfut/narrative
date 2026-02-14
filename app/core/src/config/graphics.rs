use serde::{Deserialize, Serialize};

/// Graphics configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicsConfig {
    /// Window width
    #[serde(default = "default_width")]
    pub width: u32,
    /// Window height
    #[serde(default = "default_height")]
    pub height: u32,
    /// Fullscreen mode
    #[serde(default)]
    pub fullscreen: bool,
    /// VSync enabled
    #[serde(default = "default_true")]
    pub vsync: bool,
    /// Target frame rate (0 = unlimited)
    #[serde(default = "default_fps")]
    pub target_fps: u32,
    /// MSAA sample count (1, 2, 4, 8)
    #[serde(default = "default_msaa")]
    pub msaa_samples: u32,
}

impl GraphicsConfig {
    /// Create a new graphics config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a graphics config for a specific resolution
    pub fn with_resolution(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Get the aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Check if the resolution is valid
    pub fn is_valid_resolution(&self) -> bool {
        self.width >= 640 && self.height >= 480
    }

    /// Get the frame time in seconds (for target FPS)
    pub fn target_frame_time(&self) -> Option<f32> {
        if self.target_fps > 0 {
            Some(1.0 / self.target_fps as f32)
        } else {
            None
        }
    }
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            fullscreen: false,
            vsync: default_true(),
            target_fps: default_fps(),
            msaa_samples: default_msaa(),
        }
    }
}

fn default_width() -> u32 {
    1280
}

fn default_height() -> u32 {
    720
}

fn default_true() -> bool {
    true
}

fn default_fps() -> u32 {
    60
}

fn default_msaa() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_config_new() {
        let config = GraphicsConfig::new();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert!(!config.fullscreen);
        assert!(config.vsync);
        assert_eq!(config.target_fps, 60);
        assert_eq!(config.msaa_samples, 1);
    }

    #[test]
    fn test_graphics_config_default() {
        let config = GraphicsConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
    }

    #[test]
    fn test_graphics_config_with_resolution() {
        let config = GraphicsConfig::with_resolution(1920, 1080);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert!(config.vsync);
    }

    #[test]
    fn test_graphics_config_aspect_ratio() {
        let config = GraphicsConfig::with_resolution(1920, 1080);
        assert!((config.aspect_ratio() - 1.7778).abs() < 0.001);
    }

    #[test]
    fn test_graphics_config_aspect_ratio_4_3() {
        let config = GraphicsConfig::with_resolution(800, 600);
        assert!((config.aspect_ratio() - 1.3333).abs() < 0.001);
    }

    #[test]
    fn test_graphics_config_is_valid_resolution_valid() {
        let config = GraphicsConfig::with_resolution(1280, 720);
        assert!(config.is_valid_resolution());

        let config = GraphicsConfig::with_resolution(640, 480);
        assert!(config.is_valid_resolution());
    }

    #[test]
    fn test_graphics_config_is_valid_resolution_invalid() {
        let config = GraphicsConfig::with_resolution(320, 240);
        assert!(!config.is_valid_resolution());

        let config = GraphicsConfig::with_resolution(800, 400);
        assert!(!config.is_valid_resolution());
    }

    #[test]
    fn test_graphics_config_target_frame_time() {
        let config = GraphicsConfig::new(); // 60 FPS
        let frame_time = config.target_frame_time().unwrap();
        assert!((frame_time - 0.01667).abs() < 0.0001);
    }

    #[test]
    fn test_graphics_config_target_frame_time_30fps() {
        let mut config = GraphicsConfig::new();
        config.target_fps = 30;
        let frame_time = config.target_frame_time().unwrap();
        assert!((frame_time - 0.03333).abs() < 0.0001);
    }

    #[test]
    fn test_graphics_config_target_frame_time_unlimited() {
        let mut config = GraphicsConfig::new();
        config.target_fps = 0;
        assert_eq!(config.target_frame_time(), None);
    }

    #[test]
    fn test_graphics_config_serialization() {
        let config = GraphicsConfig::new();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: GraphicsConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_graphics_config_fullscreen() {
        let mut config = GraphicsConfig::new();
        config.fullscreen = true;
        assert!(config.fullscreen);
    }

    #[test]
    fn test_graphics_config_vsync_disabled() {
        let mut config = GraphicsConfig::new();
        config.vsync = false;
        assert!(!config.vsync);
    }

    #[test]
    fn test_graphics_config_msaa() {
        let mut config = GraphicsConfig::new();
        config.msaa_samples = 4;
        assert_eq!(config.msaa_samples, 4);
    }
}
