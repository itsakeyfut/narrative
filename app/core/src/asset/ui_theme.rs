use crate::error::EngineError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// UI Theme definition
///
/// Defines a complete UI theme with dialogue boxes, buttons, choices, and colors.
///
/// # Example RON format
///
/// ```ron
/// UiThemeDef(
///     id: "light",
///     name: "Light Theme",
///     dialogue_box: (
///         default: "assets/ui/dialoguebox/dialoguebox_light_blue.png",
///         variants: {
///             "blue": "assets/ui/dialoguebox/dialoguebox_light_blue.png",
///             "pink": "assets/ui/dialoguebox/dialoguebox_light_pink.png",
///         },
///     ),
///     buttons: (
///         continue_idle: "assets/ui/buttons/btn_light_continue_idle.png",
///         continue_hover: "assets/ui/buttons/btn_light_continue_hover.png",
///     ),
///     choices: (
///         idle: "assets/ui/choices/choice_light_idle.png",
///         hover: "assets/ui/choices/choice_light_hover.png",
///         disabled: "assets/ui/choices/choice_light_disabled.png",
///     ),
///     colors: Some((
///         text_primary: (0, 0, 0, 255),
///         text_secondary: (64, 64, 64, 255),
///         accent: (100, 150, 255, 255),
///         background: (255, 255, 255, 230),
///     )),
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiThemeDef {
    /// Theme identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Dialogue box assets
    pub dialogue_box: DialogueBoxAssets,

    /// Button assets
    pub buttons: ButtonAssets,

    /// Choice assets
    pub choices: ChoiceAssets,

    /// Optional color palette
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<ColorPalette>,
}

impl UiThemeDef {
    /// Validate the UI theme definition
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("UI theme ID cannot be empty".to_string());
        }

        self.dialogue_box.validate()?;
        self.buttons.validate()?;
        self.choices.validate()?;

        Ok(())
    }
}

/// Dialogue box assets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogueBoxAssets {
    /// Default dialogue box image
    pub default: String,

    /// Variants (e.g., different colors for different characters)
    #[serde(default)]
    pub variants: HashMap<String, String>,
}

impl DialogueBoxAssets {
    pub fn validate(&self) -> Result<(), String> {
        if self.default.is_empty() {
            return Err("Dialogue box default path cannot be empty".to_string());
        }
        Ok(())
    }

    /// Get a variant or fall back to default
    pub fn get_variant(&self, variant: &str) -> &str {
        self.variants
            .get(variant)
            .map(|s| s.as_str())
            .unwrap_or(&self.default)
    }
}

/// Button assets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonAssets {
    pub continue_idle: String,
    pub continue_hover: String,

    pub history_idle: String,
    pub history_hover: String,

    pub skip_idle: String,
    pub skip_hover: String,

    pub options_idle: String,
    pub options_hover: String,
}

impl ButtonAssets {
    pub fn validate(&self) -> Result<(), String> {
        let fields = [
            ("continue_idle", &self.continue_idle),
            ("continue_hover", &self.continue_hover),
            ("history_idle", &self.history_idle),
            ("history_hover", &self.history_hover),
            ("skip_idle", &self.skip_idle),
            ("skip_hover", &self.skip_hover),
            ("options_idle", &self.options_idle),
            ("options_hover", &self.options_hover),
        ];

        for (name, value) in &fields {
            if value.is_empty() {
                return Err(format!("Button asset '{}' path cannot be empty", name));
            }
        }

        Ok(())
    }
}

/// Choice assets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoiceAssets {
    pub idle: String,
    pub hover: String,
    pub disabled: String,
}

impl ChoiceAssets {
    pub fn validate(&self) -> Result<(), String> {
        if self.idle.is_empty() {
            return Err("Choice idle path cannot be empty".to_string());
        }
        if self.hover.is_empty() {
            return Err("Choice hover path cannot be empty".to_string());
        }
        if self.disabled.is_empty() {
            return Err("Choice disabled path cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Color palette for UI theme
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorPalette {
    /// Primary text color (RGBA 0-255)
    pub text_primary: (u8, u8, u8, u8),

    /// Secondary text color (RGBA 0-255)
    pub text_secondary: (u8, u8, u8, u8),

    /// Accent color (RGBA 0-255)
    pub accent: (u8, u8, u8, u8),

    /// Background color (RGBA 0-255)
    pub background: (u8, u8, u8, u8),
}

/// UI Theme manifest - collection of UI themes
///
/// # Example RON format
///
/// ```ron
/// UiThemeManifest(
///     themes: {
///         "light": UiThemeDef(
///             id: "light",
///             name: "Light Theme",
///             // ... theme definition
///         ),
///     },
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiThemeManifest {
    /// Map of theme IDs to definitions
    pub themes: HashMap<String, UiThemeDef>,
}

impl UiThemeManifest {
    /// Create a new empty manifest
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
        }
    }

    /// Add a UI theme
    pub fn add_theme(mut self, theme: UiThemeDef) -> Self {
        self.themes.insert(theme.id.clone(), theme);
        self
    }

    /// Load manifest from a RON file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let manifest: Self = ron::from_str(&content).map_err(|e| EngineError::RonSer(e.into()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate all UI themes in the manifest
    pub fn validate(&self) -> Result<(), EngineError> {
        for (id, theme) in &self.themes {
            theme
                .validate()
                .map_err(|e| EngineError::Other(format!("UI theme '{}': {}", id, e)))?;

            if &theme.id != id {
                return Err(EngineError::Other(format!(
                    "UI theme map key '{}' does not match theme id '{}'",
                    id, theme.id
                )));
            }
        }
        Ok(())
    }

    /// Get a UI theme by ID
    pub fn get(&self, id: &str) -> Option<&UiThemeDef> {
        self.themes.get(id)
    }

    /// Get all theme IDs
    pub fn ids(&self) -> Vec<&str> {
        self.themes.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for UiThemeManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_button_assets() -> ButtonAssets {
        ButtonAssets {
            continue_idle: "btn_continue_idle.png".to_string(),
            continue_hover: "btn_continue_hover.png".to_string(),
            history_idle: "btn_history_idle.png".to_string(),
            history_hover: "btn_history_hover.png".to_string(),
            skip_idle: "btn_skip_idle.png".to_string(),
            skip_hover: "btn_skip_hover.png".to_string(),
            options_idle: "btn_options_idle.png".to_string(),
            options_hover: "btn_options_hover.png".to_string(),
        }
    }

    #[test]
    fn test_dialogue_box_assets_validation() {
        let valid = DialogueBoxAssets {
            default: "box.png".to_string(),
            variants: HashMap::new(),
        };
        assert!(valid.validate().is_ok());

        let invalid = DialogueBoxAssets {
            default: String::new(),
            variants: HashMap::new(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_dialogue_box_get_variant() {
        let mut variants = HashMap::new();
        variants.insert("blue".to_string(), "box_blue.png".to_string());

        let assets = DialogueBoxAssets {
            default: "box_default.png".to_string(),
            variants,
        };

        assert_eq!(assets.get_variant("blue"), "box_blue.png");
        assert_eq!(assets.get_variant("nonexistent"), "box_default.png");
    }

    #[test]
    fn test_button_assets_validation() {
        let valid = create_test_button_assets();
        assert!(valid.validate().is_ok());

        let mut invalid = create_test_button_assets();
        invalid.continue_idle = String::new();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_choice_assets_validation() {
        let valid = ChoiceAssets {
            idle: "choice_idle.png".to_string(),
            hover: "choice_hover.png".to_string(),
            disabled: "choice_disabled.png".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid = ChoiceAssets {
            idle: String::new(),
            hover: "choice_hover.png".to_string(),
            disabled: "choice_disabled.png".to_string(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_ui_theme_manifest_serialization() {
        let theme = UiThemeDef {
            id: "test".to_string(),
            name: "Test Theme".to_string(),
            dialogue_box: DialogueBoxAssets {
                default: "box.png".to_string(),
                variants: HashMap::new(),
            },
            buttons: create_test_button_assets(),
            choices: ChoiceAssets {
                idle: "choice_idle.png".to_string(),
                hover: "choice_hover.png".to_string(),
                disabled: "choice_disabled.png".to_string(),
            },
            colors: None,
        };

        let manifest = UiThemeManifest::new().add_theme(theme);

        let ron_str = ron::to_string(&manifest).unwrap();
        let deserialized: UiThemeManifest = ron::from_str(&ron_str).unwrap();

        assert_eq!(manifest, deserialized);
    }
}
