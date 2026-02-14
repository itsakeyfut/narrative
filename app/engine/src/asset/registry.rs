/// Asset registry for manifest-based asset management
///
/// Provides centralized management of all asset types defined in RON manifests.
use narrative_core::{
    BackgroundDef, BackgroundManifest, BgmDef, BgmManifest, CharacterDef, CharacterRegistry,
    EngineError, EngineResult, SeDef, SeManifest, UiThemeDef, UiThemeManifest,
};
use std::path::{Path, PathBuf};

// Manifest file paths
const CHARACTERS_MANIFEST: &str = "manifests/characters.ron";
const BACKGROUNDS_MANIFEST: &str = "manifests/backgrounds.ron";
const BGM_MANIFEST: &str = "manifests/bgm.ron";
const SE_MANIFEST: &str = "manifests/se.ron";
const UI_THEMES_MANIFEST: &str = "manifests/ui_themes.ron";

/// Background registry - manages background definitions from manifest
pub struct BackgroundRegistry {
    manifest: Option<BackgroundManifest>,
    base_dir: PathBuf,
}

impl BackgroundRegistry {
    /// Create a new background registry
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            manifest: None,
            base_dir,
        }
    }

    /// Load manifest from file
    pub fn load_manifest(&mut self, manifest_path: impl AsRef<Path>) -> EngineResult<()> {
        let full_path = self.base_dir.join(manifest_path.as_ref());
        self.manifest = Some(BackgroundManifest::load_from_file(&full_path)?);
        Ok(())
    }

    /// Get a background by ID
    pub fn get(&self, id: &str) -> Option<&BackgroundDef> {
        self.manifest.as_ref()?.get(id)
    }

    /// Get all background IDs
    pub fn ids(&self) -> Vec<&str> {
        self.manifest.as_ref().map(|m| m.ids()).unwrap_or_default()
    }

    /// Check if a background exists
    pub fn contains(&self, id: &str) -> bool {
        self.get(id).is_some()
    }
}

impl Default for BackgroundRegistry {
    fn default() -> Self {
        Self::new(PathBuf::from("assets"))
    }
}

/// BGM registry - manages BGM definitions from manifest
pub struct BgmRegistry {
    manifest: Option<BgmManifest>,
    base_dir: PathBuf,
}

impl BgmRegistry {
    /// Create a new BGM registry
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            manifest: None,
            base_dir,
        }
    }

    /// Load manifest from file
    pub fn load_manifest(&mut self, manifest_path: impl AsRef<Path>) -> EngineResult<()> {
        let full_path = self.base_dir.join(manifest_path.as_ref());
        self.manifest = Some(BgmManifest::load_from_file(&full_path)?);
        Ok(())
    }

    /// Get a BGM by ID
    pub fn get(&self, id: &str) -> Option<&BgmDef> {
        self.manifest.as_ref()?.get(id)
    }

    /// Get all BGM IDs
    pub fn ids(&self) -> Vec<&str> {
        self.manifest.as_ref().map(|m| m.ids()).unwrap_or_default()
    }

    /// Check if a BGM exists
    pub fn contains(&self, id: &str) -> bool {
        self.get(id).is_some()
    }
}

impl Default for BgmRegistry {
    fn default() -> Self {
        Self::new(PathBuf::from("assets"))
    }
}

/// Sound effect registry - manages SE definitions from manifest
pub struct SeRegistry {
    manifest: Option<SeManifest>,
    base_dir: PathBuf,
}

impl SeRegistry {
    /// Create a new SE registry
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            manifest: None,
            base_dir,
        }
    }

    /// Load manifest from file
    pub fn load_manifest(&mut self, manifest_path: impl AsRef<Path>) -> EngineResult<()> {
        let full_path = self.base_dir.join(manifest_path.as_ref());
        self.manifest = Some(SeManifest::load_from_file(&full_path)?);
        Ok(())
    }

    /// Get a SE by ID
    pub fn get(&self, id: &str) -> Option<&SeDef> {
        self.manifest.as_ref()?.get(id)
    }

    /// Get all SE IDs
    pub fn ids(&self) -> Vec<&str> {
        self.manifest.as_ref().map(|m| m.ids()).unwrap_or_default()
    }

    /// Check if a SE exists
    pub fn contains(&self, id: &str) -> bool {
        self.get(id).is_some()
    }
}

impl Default for SeRegistry {
    fn default() -> Self {
        Self::new(PathBuf::from("assets"))
    }
}

/// UI theme registry - manages UI theme definitions from manifest
pub struct UiThemeRegistry {
    manifest: Option<UiThemeManifest>,
    base_dir: PathBuf,
}

impl UiThemeRegistry {
    /// Create a new UI theme registry
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            manifest: None,
            base_dir,
        }
    }

    /// Load manifest from file
    pub fn load_manifest(&mut self, manifest_path: impl AsRef<Path>) -> EngineResult<()> {
        let full_path = self.base_dir.join(manifest_path.as_ref());
        self.manifest = Some(UiThemeManifest::load_from_file(&full_path)?);
        Ok(())
    }

    /// Get a UI theme by ID
    pub fn get(&self, id: &str) -> Option<&UiThemeDef> {
        self.manifest.as_ref()?.get(id)
    }

    /// Get all UI theme IDs
    pub fn ids(&self) -> Vec<&str> {
        self.manifest.as_ref().map(|m| m.ids()).unwrap_or_default()
    }

    /// Check if a UI theme exists
    pub fn contains(&self, id: &str) -> bool {
        self.get(id).is_some()
    }
}

impl Default for UiThemeRegistry {
    fn default() -> Self {
        Self::new(PathBuf::from("assets"))
    }
}

/// Unified asset registry - manages all asset types from manifests
///
/// This registry provides centralized access to all asset definitions
/// loaded from RON manifest files.
///
/// # Example
///
/// ```rust,no_run
/// use narrative_engine::asset::AssetRegistry;
///
/// let mut registry = AssetRegistry::new("assets");
///
/// // Load all manifests
/// registry.load_all_manifests().expect("Failed to load manifests");
///
/// // Access assets by dot-notation ID
/// let bg = registry.background("bg.school.classroom");
/// let bgm = registry.bgm("bgm.dailylife.school");
/// ```
pub struct AssetRegistry {
    /// Character registry
    pub characters: CharacterRegistry,
    /// Background registry
    pub backgrounds: BackgroundRegistry,
    /// BGM registry
    pub bgm: BgmRegistry,
    /// Sound effect registry
    pub se: SeRegistry,
    /// UI theme registry
    pub ui_themes: UiThemeRegistry,

    base_dir: PathBuf,
}

impl AssetRegistry {
    /// Create a new asset registry
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        let base_dir = base_dir.into();
        Self {
            characters: CharacterRegistry::with_base_dir(&base_dir),
            backgrounds: BackgroundRegistry::new(base_dir.clone()),
            bgm: BgmRegistry::new(base_dir.clone()),
            se: SeRegistry::new(base_dir.clone()),
            ui_themes: UiThemeRegistry::new(base_dir.clone()),
            base_dir,
        }
    }

    /// Load all manifest files from the standard locations
    ///
    /// Loads manifests from:
    /// - `manifests/characters.ron`
    /// - `manifests/backgrounds.ron`
    /// - `manifests/bgm.ron`
    /// - `manifests/se.ron`
    /// - `manifests/ui_themes.ron`
    pub fn load_all_manifests(&mut self) -> EngineResult<()> {
        // Load character manifest
        self.characters
            .load_from_manifest(CHARACTERS_MANIFEST)
            .map_err(|e| EngineError::Other(format!("Failed to load character manifest: {}", e)))?;

        // Load background manifest
        self.backgrounds.load_manifest(BACKGROUNDS_MANIFEST)?;

        // Load BGM manifest (allow empty)
        if let Err(e) = self.bgm.load_manifest(BGM_MANIFEST) {
            tracing::warn!("Failed to load BGM manifest: {}", e);
        }

        // Load SE manifest (allow empty)
        if let Err(e) = self.se.load_manifest(SE_MANIFEST) {
            tracing::warn!("Failed to load SE manifest: {}", e);
        }

        // Load UI theme manifest
        self.ui_themes.load_manifest(UI_THEMES_MANIFEST)?;

        Ok(())
    }

    /// Get a character by ID
    pub fn character(&self, id: &str) -> EngineResult<&CharacterDef> {
        self.characters.get(id)
    }

    /// Get a background by ID
    pub fn background(&self, id: &str) -> Option<&BackgroundDef> {
        self.backgrounds.get(id)
    }

    /// Get a BGM by ID
    pub fn bgm(&self, id: &str) -> Option<&BgmDef> {
        self.bgm.get(id)
    }

    /// Get a sound effect by ID
    pub fn sound_effect(&self, id: &str) -> Option<&SeDef> {
        self.se.get(id)
    }

    /// Get a UI theme by ID
    pub fn ui_theme(&self, id: &str) -> Option<&UiThemeDef> {
        self.ui_themes.get(id)
    }

    /// Get base directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Get statistics about loaded assets
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            characters: self.characters.len(),
            backgrounds: self.backgrounds.ids().len(),
            bgm_tracks: self.bgm.ids().len(),
            sound_effects: self.se.ids().len(),
            ui_themes: self.ui_themes.ids().len(),
        }
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new("assets")
    }
}

/// Statistics about loaded assets
#[derive(Debug, Clone, Copy)]
pub struct RegistryStats {
    pub characters: usize,
    pub backgrounds: usize,
    pub bgm_tracks: usize,
    pub sound_effects: usize,
    pub ui_themes: usize,
}

impl RegistryStats {
    /// Get total number of loaded assets
    pub fn total(&self) -> usize {
        self.characters + self.backgrounds + self.bgm_tracks + self.sound_effects + self.ui_themes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_registry_new() {
        let registry = BackgroundRegistry::new(PathBuf::from("assets"));
        assert!(registry.ids().is_empty());
    }

    #[test]
    fn test_bgm_registry_new() {
        let registry = BgmRegistry::new(PathBuf::from("assets"));
        assert!(registry.ids().is_empty());
    }

    #[test]
    fn test_se_registry_new() {
        let registry = SeRegistry::new(PathBuf::from("assets"));
        assert!(registry.ids().is_empty());
    }

    #[test]
    fn test_ui_theme_registry_new() {
        let registry = UiThemeRegistry::new(PathBuf::from("assets"));
        assert!(registry.ids().is_empty());
    }

    #[test]
    fn test_asset_registry_new() {
        let registry = AssetRegistry::new("assets");
        assert_eq!(registry.base_dir(), Path::new("assets"));
    }

    #[test]
    fn test_registry_stats() {
        let registry = AssetRegistry::new("assets");
        let stats = registry.stats();
        assert_eq!(stats.total(), 0);
    }
}
