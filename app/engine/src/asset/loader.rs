//! Unified asset loader for all asset types
//!
//! This module provides centralized loading and caching for all assets:
//! - Manifests (backgrounds, BGM, SE, UI themes, characters)
//! - Scenarios (TOML files)
//! - Textures (images)
//! - Audio (BGM, SE)

use super::{AssetRegistry, TextureCache, TextureHandle};
use crate::error::{EngineError, EngineResult};
use narrative_core::{
    AssetRef, BackgroundDef, BgmDef, CharacterDef, CharacterPosition, CharacterRegistry, Choice,
    ChoiceOption, Dialogue, Scenario, ScenarioCommand, ScenarioMetadata, Scene, SeDef, Speaker,
    Transition, UiThemeDef,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Unified asset loader
///
/// Central hub for loading all assets from the `assets/` directory.
/// All asset loading should go through this loader to avoid confusion.
///
/// # Example
///
/// ```rust,ignore
/// use narrative_engine::asset::AssetLoader;
///
/// let mut loader = AssetLoader::new("assets");
///
/// // Load all manifests
/// loader.load_manifests()?;
///
/// // Load a scenario
/// let scenario = loader.load_scenario("scenarios/chapter_01.toml")?;
///
/// // Access asset definitions
/// let bg = loader.background("bg.school.classroom")?;
/// let bgm = loader.bgm("bgm.dailylife.school")?;
/// ```
pub struct AssetLoader {
    base_path: PathBuf,
    texture_cache: TextureCache,
    registry: AssetRegistry,
    scenarios: HashMap<String, Scenario>,
}

impl AssetLoader {
    /// Create a new asset loader
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        let base_path = base_path.into();
        Self {
            registry: AssetRegistry::new(&base_path),
            base_path,
            texture_cache: TextureCache::default(),
            scenarios: HashMap::new(),
        }
    }

    /// Load all manifest files
    ///
    /// Loads:
    /// - `manifests/characters.ron`
    /// - `manifests/backgrounds.ron`
    /// - `manifests/bgm.ron`
    /// - `manifests/se.ron`
    /// - `manifests/ui_themes.ron`
    pub fn load_manifests(&mut self) -> EngineResult<()> {
        Ok(self.registry.load_all_manifests()?)
    }

    /// Load a scenario from TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path from base_path (e.g., "scenarios/chapter_01.toml")
    ///
    /// # Returns
    ///
    /// Returns a reference to the loaded and cached scenario.
    pub fn load_scenario(&mut self, path: impl AsRef<Path>) -> EngineResult<&Scenario> {
        let full_path = self.base_path.join(path.as_ref());
        let scenario = load_scenario_from_toml(&full_path)?;
        let scenario_id = scenario.metadata.id.clone();
        self.scenarios.insert(scenario_id.clone(), scenario);
        self.scenarios.get(&scenario_id).ok_or_else(|| {
            EngineError::Other("Failed to retrieve just-inserted scenario".to_string())
        })
    }

    /// Get a previously loaded scenario
    pub fn get_scenario(&self, scenario_id: &str) -> Option<&Scenario> {
        self.scenarios.get(scenario_id)
    }

    /// Load a texture
    pub fn load_texture(&mut self, _asset_ref: &AssetRef) -> EngineResult<TextureHandle> {
        // TODO: Phase 0.5 - asset loading implementation
        Ok(TextureHandle::default())
    }

    /// Get the asset registry
    pub fn registry(&self) -> &AssetRegistry {
        &self.registry
    }

    /// Get mutable asset registry (for hot-reload)
    pub fn registry_mut(&mut self) -> &mut AssetRegistry {
        &mut self.registry
    }

    /// Get a background definition by ID
    pub fn background(&self, id: &str) -> Option<&BackgroundDef> {
        self.registry.background(id)
    }

    /// Get a BGM definition by ID
    pub fn bgm(&self, id: &str) -> Option<&BgmDef> {
        self.registry.bgm(id)
    }

    /// Get a sound effect definition by ID
    pub fn sound_effect(&self, id: &str) -> Option<&SeDef> {
        self.registry.sound_effect(id)
    }

    /// Get a character definition by ID
    pub fn character(&self, id: &str) -> EngineResult<&CharacterDef> {
        Ok(self.registry.character(id)?)
    }

    /// Get a UI theme definition by ID
    pub fn ui_theme(&self, id: &str) -> Option<&UiThemeDef> {
        self.registry.ui_theme(id)
    }

    /// Get the texture cache
    pub fn texture_cache(&self) -> &TextureCache {
        &self.texture_cache
    }

    /// Get mutable texture cache
    pub fn texture_cache_mut(&mut self) -> &mut TextureCache {
        &mut self.texture_cache
    }

    /// Get base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Get statistics about loaded assets
    pub fn stats(&self) -> AssetStats {
        let registry_stats = self.registry.stats();
        AssetStats {
            characters: registry_stats.characters,
            backgrounds: registry_stats.backgrounds,
            bgm_tracks: registry_stats.bgm_tracks,
            sound_effects: registry_stats.sound_effects,
            ui_themes: registry_stats.ui_themes,
            scenarios: self.scenarios.len(),
        }
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new("assets")
    }
}

/// Statistics about loaded assets
#[derive(Debug, Clone, Copy)]
pub struct AssetStats {
    pub characters: usize,
    pub backgrounds: usize,
    pub bgm_tracks: usize,
    pub sound_effects: usize,
    pub ui_themes: usize,
    pub scenarios: usize,
}

impl AssetStats {
    /// Get total number of loaded assets
    pub fn total(&self) -> usize {
        self.characters
            + self.backgrounds
            + self.bgm_tracks
            + self.sound_effects
            + self.ui_themes
            + self.scenarios
    }
}

// ============================================================================
// TOML Scenario Parsing (Private Implementation)
// ============================================================================

/// Flexible transition that can be either a string name or a full Transition object
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FlexibleTransition {
    Name(String),
    Object(Transition),
}

/// TOML intermediate structure for deserializing scenario files
#[derive(Debug, Deserialize)]
struct TomlScenario {
    chapter: ChapterInfo,
    #[serde(default)]
    settings: Option<ScenarioSettings>,
    #[serde(default)]
    characters: Vec<CharacterDef>,
    #[serde(default)]
    scenes: Vec<TomlScene>,
}

/// TOML scene structure
#[derive(Debug, Deserialize)]
struct TomlScene {
    id: String,
    title: String,
    #[serde(default)]
    background: Option<String>,
    #[serde(default)]
    dialogue: Vec<TomlDialogue>,
    #[serde(default)]
    choices: Vec<ChoiceOption>,
    #[serde(default)]
    commands: Vec<ScenarioCommand>,
    #[serde(default)]
    entry_transition: Option<FlexibleTransition>,
    #[serde(default)]
    exit_transition: Option<FlexibleTransition>,
    #[serde(default)]
    transition_duration: Option<f32>,
}

/// TOML sound effect entry
#[derive(Debug, Deserialize)]
struct TomlSoundEffect {
    sound: String,
    #[serde(default)]
    delay: f32,
    #[serde(default = "default_se_volume")]
    volume: f32,
}

fn default_se_volume() -> f32 {
    1.0
}

/// TOML dialogue entry
#[derive(Debug, Deserialize)]
struct TomlDialogue {
    speaker: String,
    text: String,
    #[serde(default)]
    character_sprite: Option<String>,
    #[serde(default)]
    sprite_position: Option<String>,
    #[serde(default)]
    sprite_transition: Option<String>,
    #[serde(default)]
    transition_duration: Option<f32>,
    #[serde(default)]
    sound_effects: Vec<TomlSoundEffect>,
    #[serde(default)]
    animation: Option<narrative_core::character::CharacterAnimation>,
}

/// Chapter metadata
#[derive(Debug, Deserialize)]
struct ChapterInfo {
    id: String,
    title: String,
    #[serde(default)]
    description: Option<String>,
}

/// Scenario settings
#[derive(Debug, Deserialize)]
struct ScenarioSettings {
    #[serde(default)]
    character_manifest: Option<String>,
}

impl TomlDialogue {
    fn parse_transition(&self) -> Transition {
        use narrative_core::{SlideDirection, TransitionKind};
        let duration = self.transition_duration.unwrap_or(0.5);
        match self.sprite_transition.as_deref() {
            Some("fade_in") | Some("fade") => Transition::new(TransitionKind::Fade, duration),
            Some("crossfade") => Transition::new(TransitionKind::Crossfade, duration),
            Some("slide_in_left") => {
                Transition::new(TransitionKind::Slide(SlideDirection::Left), duration)
            }
            Some("slide_in_right") => {
                Transition::new(TransitionKind::Slide(SlideDirection::Right), duration)
            }
            _ => Transition::instant(),
        }
    }
}

impl TomlScene {
    fn into_scene(self) -> EngineResult<Scene> {
        let mut commands = Vec::new();
        let mut displayed_characters: Vec<(String, String, CharacterPosition)> = Vec::new();

        if let Some(bg) = self.background {
            commands.push(ScenarioCommand::ShowBackground {
                asset: AssetRef::from(bg),
                transition: Transition::default(),
            });
        }

        if !self.commands.is_empty() {
            for cmd in &self.commands {
                if let ScenarioCommand::ShowCharacter {
                    character_id,
                    sprite,
                    position,
                    ..
                } = cmd
                {
                    displayed_characters.push((character_id.clone(), sprite.0.clone(), *position));
                }
            }
            commands.extend(self.commands);
        }

        for dialogue_entry in self.dialogue {
            let transition = dialogue_entry.parse_transition();
            let speaker = dialogue_entry.speaker;
            let text = dialogue_entry.text;

            if let Some(sprite) = dialogue_entry.character_sprite {
                let position = match dialogue_entry.sprite_position.as_deref() {
                    Some("left") => CharacterPosition::Left,
                    Some("center") => CharacterPosition::Center,
                    Some("right") => CharacterPosition::Right,
                    _ => CharacterPosition::Center,
                };

                let char_key = (speaker.clone(), sprite.clone(), position);
                if !displayed_characters
                    .iter()
                    .any(|k| k.0 == char_key.0 && k.1 == char_key.1 && k.2 == char_key.2)
                {
                    commands.push(ScenarioCommand::ShowCharacter {
                        character_id: speaker.clone(),
                        sprite: AssetRef::from(sprite),
                        position,
                        expression: None,
                        transition,
                    });
                    displayed_characters.push(char_key);
                }
            }

            for se in &dialogue_entry.sound_effects {
                if se.delay > 0.0 {
                    commands.push(ScenarioCommand::Wait { duration: se.delay });
                }
                commands.push(ScenarioCommand::PlaySe {
                    asset: AssetRef::from(se.sound.clone()),
                    volume: se.volume,
                });
            }

            commands.push(ScenarioCommand::Dialogue {
                dialogue: Dialogue {
                    speaker: Speaker::from(speaker),
                    text,
                    expression: None,
                    animation: dialogue_entry.animation,
                },
            });
        }

        if !self.choices.is_empty() {
            commands.push(ScenarioCommand::ShowChoice {
                choice: Choice::new(self.choices),
            });
        }

        let entry_transition = self.entry_transition.map(|trans| match trans {
            FlexibleTransition::Name(name) => {
                Transition::from_name(&name, self.transition_duration.unwrap_or(0.5))
            }
            FlexibleTransition::Object(obj) => obj,
        });

        let exit_transition = self.exit_transition.map(|trans| match trans {
            FlexibleTransition::Name(name) => {
                Transition::from_name(&name, self.transition_duration.unwrap_or(0.5))
            }
            FlexibleTransition::Object(obj) => obj,
        });

        Ok(Scene {
            id: self.id,
            title: self.title,
            commands,
            entry_transition,
            exit_transition,
        })
    }
}

impl TomlScenario {
    fn into_scenario(self) -> EngineResult<Scenario> {
        let mut scenes: HashMap<String, Scene> = HashMap::new();
        let mut first_scene_id: Option<String> = None;

        for toml_scene in self.scenes {
            let scene_id = toml_scene.id.clone();
            if first_scene_id.is_none() {
                first_scene_id = Some(scene_id.clone());
            }
            if scenes.contains_key(&scene_id) {
                return Err(EngineError::ScenarioExecution(format!(
                    "Duplicate scene ID: '{}'",
                    scene_id
                )));
            }
            scenes.insert(scene_id, toml_scene.into_scene()?);
        }

        let start_scene = first_scene_id.ok_or_else(|| {
            EngineError::ScenarioExecution("No scenes found in scenario".to_string())
        })?;

        Ok(Scenario {
            metadata: ScenarioMetadata {
                id: self.chapter.id,
                title: self.chapter.title,
                description: self.chapter.description,
                author: None,
                version: None,
            },
            characters: self.characters,
            scenes,
            start_scene,
        })
    }
}

/// Load a scenario from a TOML file (private)
fn load_scenario_from_toml<P: AsRef<Path>>(path: P) -> EngineResult<Scenario> {
    let scenario_path = path.as_ref();
    let content = std::fs::read_to_string(scenario_path).map_err(|e| {
        EngineError::ScenarioExecution(format!(
            "Failed to read scenario file '{}': {}",
            scenario_path.display(),
            e
        ))
    })?;

    let mut toml_scenario: TomlScenario = toml::from_str(&content).map_err(|e| {
        EngineError::ScenarioExecution(format!(
            "Failed to parse TOML '{}': {}",
            scenario_path.display(),
            e
        ))
    })?;

    // Debug logging removed - animations verified to load correctly

    if let Some(settings) = &toml_scenario.settings
        && let Some(manifest_path) = &settings.character_manifest
    {
        let scenario_dir = scenario_path.parent().unwrap_or_else(|| Path::new("."));
        let mut registry = CharacterRegistry::with_base_dir(scenario_dir);
        registry.load_from_manifest(manifest_path).map_err(|e| {
            EngineError::ScenarioExecution(format!(
                "Failed to load character manifest '{}': {}",
                manifest_path, e
            ))
        })?;

        let character_ids = registry.character_ids();
        let mut characters = Vec::new();
        for char_id in character_ids {
            if let Ok(char_def) = registry.get(char_id) {
                characters.push(char_def.clone());
            }
        }
        toml_scenario.characters = characters;
    }

    let scenario = toml_scenario.into_scenario()?;
    Ok(scenario)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_loader_new() {
        let loader = AssetLoader::new("assets");
        assert_eq!(loader.base_path(), Path::new("assets"));
    }

    #[test]
    fn test_asset_loader_default() {
        let loader = AssetLoader::default();
        assert_eq!(loader.base_path(), Path::new("assets"));
    }

    #[test]
    fn test_asset_stats() {
        let loader = AssetLoader::new("assets");
        let stats = loader.stats();
        assert_eq!(stats.total(), 0);
    }

    #[test]
    fn test_scene_command_dialogue_with_animation_deserialization() {
        use narrative_core::ScenarioCommand;

        let toml_str = r#"
[[scenes]]
id = "test"
title = "Test"

[[scenes.commands]]
type = "Dialogue"
dialogue = {
    speaker = { Character = "bob" },
    text = "Run!",
    animation = { type = "escape", direction = "right", preset = "small" }
}
"#;

        #[derive(Debug, serde::Deserialize)]
        struct TestFile {
            scenes: Vec<TestScene>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct TestScene {
            id: String,
            title: String,
            commands: Vec<ScenarioCommand>,
        }

        let file: TestFile = toml::from_str(toml_str).unwrap();
        assert_eq!(file.scenes.len(), 1);
        assert_eq!(file.scenes[0].commands.len(), 1);

        match &file.scenes[0].commands[0] {
            ScenarioCommand::Dialogue { dialogue } => {
                assert_eq!(dialogue.text, "Run!");
                assert!(
                    dialogue.animation.is_some(),
                    "Animation should be present in deserialized dialogue!"
                );
                assert!(dialogue.animation.as_ref().unwrap().is_keyframe_based());
            }
            _ => panic!("Expected Dialogue command"),
        }
    }
}
