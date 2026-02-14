//! Font management with cosmic-text integration

use crate::error::{EngineError, EngineResult};
use cosmic_text::{Attrs, Family, FontSystem, Metrics, fontdb};
use std::path::Path;

/// Font manager wrapping cosmic-text's FontSystem
pub struct FontManager {
    font_system: FontSystem,
}

impl FontManager {
    /// Create a new font manager
    pub fn new() -> EngineResult<Self> {
        let mut font_system = FontSystem::new();

        // Load system fonts as fallback
        font_system.db_mut().load_system_fonts();

        Ok(Self { font_system })
    }

    /// Load a Japanese font from file
    pub fn load_japanese_font<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let path = path.as_ref();

        // Read font file
        let font_data = std::fs::read(path).map_err(|e| {
            EngineError::FontLoad(format!(
                "Failed to read font file '{}': {}",
                path.display(),
                e
            ))
        })?;

        // Load font into database
        self.font_system.db_mut().load_font_data(font_data);

        Ok(())
    }

    /// Load a font from memory
    pub fn load_font_data(&mut self, data: Vec<u8>) -> EngineResult<()> {
        self.font_system.db_mut().load_font_data(data);
        Ok(())
    }

    /// Get mutable reference to the font system
    pub fn font_system_mut(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }

    /// Get reference to the font system
    pub fn font_system(&self) -> &FontSystem {
        &self.font_system
    }

    /// Get font database
    pub fn font_db(&self) -> &fontdb::Database {
        self.font_system.db()
    }

    /// Get mutable font database
    pub fn font_db_mut(&mut self) -> &mut fontdb::Database {
        self.font_system.db_mut()
    }

    /// Check if a font family is available
    pub fn has_font_family(&self, family: &str) -> bool {
        self.font_system
            .db()
            .faces()
            .any(|face| face.families.iter().any(|(name, _)| name == family))
    }

    /// List all available font families
    pub fn list_font_families(&self) -> Vec<String> {
        let mut families = std::collections::HashSet::new();

        for face in self.font_system.db().faces() {
            for (family, _) in &face.families {
                families.insert(family.clone());
            }
        }

        let mut families: Vec<String> = families.into_iter().collect();
        families.sort();
        families
    }

    /// Get default metrics for a given font size
    pub fn default_metrics(font_size: f32) -> Metrics {
        Metrics::new(font_size, font_size * 1.4) // 1.4 line height multiplier
    }

    /// Get default attributes for Japanese text
    pub fn japanese_attrs() -> Attrs<'static> {
        Attrs::new().family(Family::SansSerif)
    }
}

impl Default for FontManager {
    fn default() -> Self {
        // SAFETY: FontSystem::new() is infallible in cosmic-text.
        // The constructor only allocates internal data structures and loads system fonts.
        // System font loading failures are silently ignored, so new() never fails.
        Self::new().expect("FontSystem initialization is infallible")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let manager = FontManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_font_manager_default() {
        let _manager = FontManager::default();
        // Should not panic
    }

    #[test]
    fn test_font_manager_has_system_fonts() {
        let manager = FontManager::new().unwrap();
        let families = manager.list_font_families();

        // System should have at least some fonts loaded
        assert!(!families.is_empty(), "No system fonts loaded");
    }

    #[test]
    fn test_default_metrics() {
        let metrics = FontManager::default_metrics(16.0);
        assert_eq!(metrics.font_size, 16.0);
        assert_eq!(metrics.line_height, 16.0 * 1.4);
    }

    #[test]
    fn test_japanese_attrs() {
        let attrs = FontManager::japanese_attrs();
        // Should create valid attributes for Japanese text
        assert_eq!(attrs.family, Family::SansSerif);
    }

    #[test]
    fn test_load_invalid_font_path() {
        let mut manager = FontManager::new().unwrap();
        let result = manager.load_japanese_font("non_existent_font.ttf");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EngineError::FontLoad(_)));
    }

    #[test]
    fn test_has_font_family() {
        let manager = FontManager::new().unwrap();

        // Check against all available families
        let families = manager.list_font_families();
        if let Some(first_family) = families.first() {
            assert!(manager.has_font_family(first_family));
        }

        // Non-existent family should return false
        assert!(!manager.has_font_family("NonExistentFontFamily12345"));
    }
}
