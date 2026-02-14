//! Scenario file validation module
//!
//! Provides validation functionality for TOML scenario files.
//! Can be used both from CLI and from the editor.

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// TOML scenario file structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TomlScenario {
    chapter: ChapterInfo,
    characters: Vec<CharacterInfo>,
    scenes: Vec<SceneInfo>,
    settings: ScenarioSettings,
}

#[derive(Debug, Deserialize)]
struct ChapterInfo {
    id: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct CharacterInfo {
    id: String,
    name: String,
    default_sprite: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SceneInfo {
    id: String,
    title: String,
    #[serde(default)]
    background: Option<String>,
    #[serde(default)]
    dialogue: Vec<DialogueInfo>,
    #[serde(default)]
    choices: Vec<ChoiceInfo>,
}

#[derive(Debug, Deserialize)]
struct DialogueInfo {
    speaker: String,
    #[allow(dead_code)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct ChoiceInfo {
    #[allow(dead_code)]
    text: String,
    next_scene: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ScenarioSettings {
    #[serde(default)]
    background_music: Option<String>,
    #[serde(default)]
    default_background: Option<String>,
}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub strict_mode: bool,
    pub check_assets: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            check_assets: true,
        }
    }
}

/// Validation result for a single scenario
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub file_path: PathBuf,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub success: bool,
}

impl ValidationResult {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            errors: Vec::new(),
            warnings: Vec::new(),
            success: true,
        }
    }

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.success = false;
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Check if validation was successful (no errors)
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Validate a single scenario file
///
/// # Example
///
/// ```no_run
/// use narrative_tools::scenario_validator::{validate_file, ValidationConfig};
///
/// let config = ValidationConfig::default();
/// let result = validate_file("assets/scenarios/chapter_01.toml", &config)?;
///
/// if result.has_errors() {
///     eprintln!("Validation failed: {:?}", result.errors);
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn validate_file(
    file_path: impl AsRef<Path>,
    config: &ValidationConfig,
) -> Result<ValidationResult> {
    let file_path = file_path.as_ref();
    let mut result = ValidationResult::new(file_path.to_path_buf());

    // Check if file exists
    if !file_path.exists() {
        result.add_error("File does not exist".to_string());
        return Ok(result);
    }

    // Try to load and parse the TOML file
    let toml_content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            result.add_error(format!("Failed to read file: {}", e));
            return Ok(result);
        }
    };

    let scenario: TomlScenario = match toml::from_str(&toml_content) {
        Ok(scenario) => scenario,
        Err(e) => {
            result.add_error(format!("TOML parsing error: {}", e));
            return Ok(result);
        }
    };

    // Validate scenario structure
    validate_scenario_structure(&scenario, &mut result, config);

    // Validate character references
    validate_character_references(&scenario, &mut result);

    // Validate scene flow
    validate_scene_flow(&scenario, &mut result);

    // Validate assets if enabled
    if config.check_assets {
        validate_assets(&scenario, &mut result);
    }

    Ok(result)
}

/// Validate all scenario files in a directory
///
/// # Example
///
/// ```no_run
/// use narrative_tools::scenario_validator::{validate_directory, ValidationConfig};
///
/// let config = ValidationConfig::default();
/// let results = validate_directory("assets/scenarios", &config)?;
///
/// for result in &results {
///     if result.has_errors() {
///         println!("❌ {}: {} errors", result.file_path.display(), result.errors.len());
///     }
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn validate_directory(
    dir_path: impl AsRef<Path>,
    config: &ValidationConfig,
) -> Result<Vec<ValidationResult>> {
    let dir_path = dir_path.as_ref();
    let mut results = Vec::new();

    if !dir_path.exists() {
        anyhow::bail!("Directory does not exist: {}", dir_path.display());
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension() == Some(std::ffi::OsStr::new("toml")) {
            let result = validate_file(&path, config)?;
            results.push(result);
        }
    }

    Ok(results)
}

fn validate_scenario_structure(
    scenario: &TomlScenario,
    result: &mut ValidationResult,
    config: &ValidationConfig,
) {
    // Check chapter info
    if scenario.chapter.id.is_empty() {
        result.add_error("Chapter ID cannot be empty".to_string());
    }
    if scenario.chapter.title.is_empty() {
        result.add_error("Chapter title cannot be empty".to_string());
    }

    // Check characters
    if scenario.characters.is_empty() {
        result.add_warning("No characters defined in scenario".to_string());
    }

    for character in &scenario.characters {
        if character.id.is_empty() {
            result.add_error("Character ID cannot be empty".to_string());
        }
        if character.name.is_empty() {
            result.add_error("Character name cannot be empty".to_string());
        }
        if character.default_sprite.is_empty() {
            result.add_error(format!(
                "Default sprite path cannot be empty for character '{}'",
                character.id
            ));
        }
    }

    // Check scenes
    if scenario.scenes.is_empty() {
        result.add_error("No scenes defined in scenario".to_string());
    }

    for scene in &scenario.scenes {
        if scene.id.is_empty() {
            result.add_error("Scene ID cannot be empty".to_string());
        }
        if scene.title.is_empty() && config.strict_mode {
            result.add_warning(format!("Scene '{}' has no title", scene.id));
        }
        if scene.dialogue.is_empty() && scene.choices.is_empty() {
            result.add_warning(format!("Scene '{}' has no dialogue or choices", scene.id));
        }
    }
}

fn validate_character_references(scenario: &TomlScenario, result: &mut ValidationResult) {
    let character_ids: Vec<&str> = scenario.characters.iter().map(|c| c.id.as_str()).collect();
    let character_names: Vec<&str> = scenario
        .characters
        .iter()
        .map(|c| c.name.as_str())
        .collect();

    for scene in &scenario.scenes {
        for dialogue in &scene.dialogue {
            // Check if speaker exists in character definitions
            if !character_ids.contains(&dialogue.speaker.as_str())
                && !character_names.contains(&dialogue.speaker.as_str())
                && dialogue.speaker != "protagonist" // Allow special characters
                && dialogue.speaker != "narrator"
            {
                result.add_warning(format!(
                    "Scene '{}': Speaker '{}' not found in character definitions",
                    scene.id, dialogue.speaker
                ));
            }
        }
    }
}

fn validate_scene_flow(scenario: &TomlScenario, result: &mut ValidationResult) {
    let scene_ids: Vec<&str> = scenario.scenes.iter().map(|s| s.id.as_str()).collect();

    for scene in &scenario.scenes {
        // Check choice targets
        for choice in &scene.choices {
            if !scene_ids.contains(&choice.next_scene.as_str()) {
                result.add_error(format!(
                    "Scene '{}': Choice references non-existent scene '{}'",
                    scene.id, choice.next_scene
                ));
            }
        }
    }

    // Check for unreachable scenes (basic check - scenes with no incoming references)
    let mut referenced_scenes = HashSet::new();

    // Assume first scene is the entry point
    if !scenario.scenes.is_empty() {
        referenced_scenes.insert(scenario.scenes[0].id.as_str());
    }

    // Collect all referenced scenes from choices
    for scene in &scenario.scenes {
        for choice in &scene.choices {
            referenced_scenes.insert(choice.next_scene.as_str());
        }
    }

    // Check for unreachable scenes
    for scene in &scenario.scenes {
        if !referenced_scenes.contains(scene.id.as_str())
            && !scenario.scenes.is_empty()
            && scene.id != scenario.scenes[0].id
        {
            result.add_warning(format!("Scene '{}' may be unreachable", scene.id));
        }
    }
}

fn validate_assets(_scenario: &TomlScenario, _result: &mut ValidationResult) {
    // TODO: Implement asset validation
    // - Check if referenced sprite files exist
    // - Check if background images exist
    // - Check if audio files exist
}
