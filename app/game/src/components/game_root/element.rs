//! GameRootElement struct definition and constructors

use narrative_core::config::UserSettings;
use narrative_core::{AssetRef, CgRegistry, UnlockData};
use narrative_engine::asset::TextureCache;
use narrative_engine::runtime::{AppState, InGameState, MainMenuState, ScenarioRuntime};
use narrative_engine::save::SaveManager;
use narrative_engine::{AudioManager, EngineConfig};
use narrative_gui::framework::element::{Element, ElementId, WindowOperation};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use taffy::NodeId;

/// Game root element - container for all game UI
///
/// This is the top-level element that manages the entire game state and UI.
/// It holds the ScenarioRuntime and AppState, updates them each frame,
/// and dynamically generates child elements based on the current state.
pub struct GameRootElement {
    /// Unique element ID
    pub(super) id: ElementId,
    /// Taffy layout node
    pub(super) layout_node: Option<NodeId>,
    /// Current application state
    pub(super) app_state: AppState,
    /// Scenario runtime (None during loading/menu)
    pub(super) scenario_runtime: Option<ScenarioRuntime>,
    /// Engine configuration
    pub(super) config: EngineConfig,
    /// Child UI elements (dynamically generated based on state)
    pub(super) children: Vec<Box<dyn Element>>,
    /// Previous in-game state (for pause/save-load menu)
    pub(super) previous_in_game_state: Option<Box<InGameState>>,
    /// Previous application state (for settings menu)
    pub(super) previous_app_state: Option<Box<AppState>>,
    /// Dirty flag to track if children need rebuilding
    pub(super) children_dirty: bool,
    /// Input state from previous frame
    pub(super) clicked_last_frame: bool,
    /// Pause key pressed this frame
    pub(super) pause_pressed: bool,
    /// Auto mode toggle pressed this frame
    pub(super) auto_mode_toggle_pressed: bool,
    /// Skip mode toggle pressed this frame
    pub(super) skip_mode_toggle_pressed: bool,
    /// Backlog key pressed this frame
    pub(super) backlog_pressed: bool,
    /// Background texture ID (set from Window after loading)
    pub(super) background_texture_id: Option<u64>,
    /// Character texture ID (set from Window after loading)
    pub(super) character_texture_id: Option<u64>,
    /// Currently displayed background texture ID (dynamically updated)
    pub(super) current_background_texture_id: Option<u64>,
    /// Previous background texture ID (used for crossfade transitions)
    pub(super) previous_background_texture_id: Option<u64>,
    /// Background texture cache (AssetRef -> TextureId)
    pub(super) background_texture_cache: HashMap<AssetRef, u64>,
    /// Currently displayed background AssetRef (for change detection)
    pub(super) displayed_background: Option<AssetRef>,
    /// Pending background to load in next frame
    pub(super) pending_background: Option<AssetRef>,
    /// Currently displayed CG texture ID (dynamically updated)
    pub(super) current_cg_texture_id: Option<u64>,
    /// Size of current CG texture (width, height) for aspect ratio calculation
    pub(super) current_cg_texture_size: Option<(u32, u32)>,
    /// Previous CG texture ID (used for crossfade transitions)
    pub(super) previous_cg_texture_id: Option<u64>,
    /// Size of previous CG texture for crossfade transitions
    pub(super) previous_cg_texture_size: Option<(u32, u32)>,
    /// CG texture cache (AssetRef -> (TextureId, Size))
    pub(super) cg_texture_cache: HashMap<AssetRef, (u64, (u32, u32))>,
    /// Currently displayed CG AssetRef (for change detection)
    pub(super) displayed_cg: Option<AssetRef>,
    /// Pending CG to load in next frame
    pub(super) pending_cg: Option<AssetRef>,
    /// CG thumbnail texture cache (CgId -> TextureId) for gallery display
    pub(super) cg_thumbnail_cache: HashMap<String, u64>,
    /// Audio manager for BGM/SE/Voice playback
    pub(super) audio_manager: Arc<Mutex<AudioManager>>,
    /// Save manager for save/load operations
    pub(super) save_manager: Arc<Mutex<SaveManager>>,
    /// Total play time in seconds (accumulated across sessions)
    pub(super) total_play_time_secs: u64,
    /// Accumulator for fractional seconds (for accurate play time tracking)
    pub(super) play_time_accumulator: f32,
    /// Flag to track if BGM has been started
    pub(super) bgm_started: bool,
    /// Pending window operations (e.g., close window)
    pub(super) window_operations: Vec<WindowOperation>,
    /// Flag to track if showing confirmation dialog for returning to title
    pub(super) showing_title_confirm: bool,
    /// Flag to track if UI is hidden (for background appreciation)
    ///
    /// This is only active during Typing/WaitingInput states and automatically
    /// resets to false when transitioning to other states (e.g., ShowingChoices, PauseMenu).
    /// Toggled by right-click or H key in Typing/WaitingInput states.
    pub(super) ui_hidden: bool,
    /// CG registry containing all game CGs
    pub(super) cg_registry: Arc<CgRegistry>,
    /// Global unlock data (persists across saves)
    pub(super) unlock_data: Arc<Mutex<UnlockData>>,
    /// Current window size (width, height) for responsive layout
    pub(super) window_size: (f32, f32),
    /// Last seen character states for transition optimization
    /// Maps character_id -> (sprite, position) to detect actual changes
    pub(super) last_seen_characters: HashMap<String, (AssetRef, narrative_core::CharacterPosition)>,
    /// Character texture cache with LRU eviction
    /// Capacity is configured via EngineConfig.graphics.character_cache_capacity
    /// TODO(layered-sprites): Change value type to Vec<TextureHandle> for layered sprites
    pub(super) character_texture_cache: TextureCache,
    /// Pending character textures to load in next frame
    pub(super) pending_character_textures: Vec<(String, AssetRef)>,
}

impl GameRootElement {
    /// Frame time for 60 FPS
    ///
    /// FIXME: This assumes a fixed 60 FPS frame rate. In the future, the Element::tick()
    /// signature should be extended to accept an actual delta time parameter to support
    /// variable frame rates. This would make the typewriter effect and other time-based
    /// animations more accurate across different hardware configurations.
    pub(super) const FRAME_TIME: f32 = 1.0 / 60.0;

    /// Create a new game root element
    pub fn new(config: EngineConfig) -> Self {
        // Load user settings to get audio configuration
        let audio_config = match UserSettings::load("assets/config/settings.ron") {
            Ok(settings) => {
                tracing::info!("Loaded user settings from assets/config/settings.ron");
                let core_config = settings.to_audio_config();
                // Convert narrative_core::AudioConfig to narrative_engine::app::AudioConfig
                narrative_engine::app::AudioConfig {
                    master_volume: core_config.master_volume,
                    music_volume: core_config.bgm_volume,
                    sound_volume: core_config.se_volume,
                    voice_volume: core_config.voice_volume,
                    enabled: core_config.enabled,
                }
            }
            Err(e) => {
                tracing::debug!("Could not load user settings, using defaults: {}", e);
                narrative_engine::app::AudioConfig::default()
            }
        };

        // Initialize audio manager with user-configured volumes
        let audio_manager = match AudioManager::with_config(audio_config) {
            Ok(manager) => {
                tracing::info!("AudioManager initialized successfully with user settings");
                Arc::new(Mutex::new(manager))
            }
            Err(e) => {
                tracing::error!("Failed to initialize AudioManager: {}", e);
                tracing::warn!("Running in audio-disabled mode - audio will not play");
                // Create a disabled audio manager that will continue to work without audio
                Arc::new(Mutex::new(AudioManager::disabled()))
            }
        };

        // Load CG definitions from TOML
        // TODO: Add load_cg_definitions to AssetLoader
        let cg_registry = {
            tracing::warn!("CG definitions loading temporarily disabled - using empty registry");
            Arc::new(CgRegistry::new())
        };

        // Load or create unlock data (with migration from old path)
        let unlock_data = {
            let old_path = std::path::PathBuf::from("config/unlocks.ron");
            let new_path = UnlockData::default_path();

            // Try loading from new path first
            match UnlockData::load_default() {
                Ok(data) => {
                    tracing::info!(
                        "Loaded unlock data from {}: {} CGs unlocked",
                        new_path.display(),
                        data.unlocked_cg_count()
                    );
                    Arc::new(Mutex::new(data))
                }
                Err(_) => {
                    // Try migrating from old path
                    if old_path.exists() {
                        tracing::info!(
                            "Migrating unlock data from {} to {}",
                            old_path.display(),
                            new_path.display()
                        );
                        match UnlockData::load_from_file(&old_path) {
                            Ok(data) => {
                                // Save to new path
                                if let Err(e) = data.save_default() {
                                    tracing::warn!("Failed to save migrated unlock data: {}", e);
                                } else {
                                    tracing::info!("Successfully migrated unlock data");
                                    // Optionally delete old file
                                    if let Err(e) = std::fs::remove_file(&old_path) {
                                        tracing::debug!("Could not remove old unlock file: {}", e);
                                    }
                                }
                                Arc::new(Mutex::new(data))
                            }
                            Err(e) => {
                                tracing::info!(
                                    "Could not migrate unlock data: {}, creating new",
                                    e
                                );
                                Arc::new(Mutex::new(UnlockData::new()))
                            }
                        }
                    } else {
                        tracing::info!("Creating new unlock data");
                        Arc::new(Mutex::new(UnlockData::new()))
                    }
                }
            }
        };

        // Cache capacity before moving config
        let character_cache_capacity = config.graphics.character_cache_capacity;

        Self {
            id: ElementId::new(),
            layout_node: None,
            app_state: AppState::default(), // Starts in Loading state
            scenario_runtime: None,
            config,
            children: Vec::new(),
            previous_in_game_state: None,
            previous_app_state: None,
            children_dirty: true, // Initial build needed
            clicked_last_frame: false,
            pause_pressed: false,
            auto_mode_toggle_pressed: false,
            skip_mode_toggle_pressed: false,
            backlog_pressed: false,
            background_texture_id: None,
            character_texture_id: None,
            current_background_texture_id: None,
            previous_background_texture_id: None,
            background_texture_cache: HashMap::new(),
            displayed_background: None,
            pending_background: None,
            current_cg_texture_id: None,
            current_cg_texture_size: None,
            previous_cg_texture_id: None,
            previous_cg_texture_size: None,
            cg_texture_cache: HashMap::new(),
            displayed_cg: None,
            pending_cg: None,
            cg_thumbnail_cache: HashMap::new(),
            audio_manager,
            save_manager: Arc::new(Mutex::new(SaveManager::new(std::path::PathBuf::from(
                "saves",
            )))),
            total_play_time_secs: 0,
            play_time_accumulator: 0.0,
            bgm_started: false,
            window_operations: Vec::new(),
            showing_title_confirm: false,
            ui_hidden: false,
            cg_registry,
            unlock_data,
            window_size: (1280.0, 720.0), // Default, updated in layout()
            last_seen_characters: HashMap::new(),
            character_texture_cache: TextureCache::with_capacity(character_cache_capacity)
                .expect("Invalid character cache capacity"),
            pending_character_textures: Vec::new(),
        }
    }

    /// Create a new game root element with a specific scenario
    ///
    /// This is an alternative constructor that allows specifying a scenario path
    /// different from the one in the engine config. Useful for testing and
    /// scenario-specific applications.
    ///
    /// # Arguments
    /// * `config` - Engine configuration
    /// * `scenario_path` - Path to the scenario TOML file to load
    ///
    /// # Example
    /// ```no_run
    /// use narrative_game::components::GameRootElement;
    /// use narrative_engine::EngineConfig;
    ///
    /// let config = EngineConfig::default();
    /// let root = GameRootElement::with_scenario(config, "assets/scenarios/performance_test.toml");
    /// ```
    pub fn with_scenario<P: AsRef<std::path::Path>>(
        mut config: EngineConfig,
        scenario_path: P,
    ) -> Self {
        // Override the start_scenario in config
        config.start_scenario = scenario_path.as_ref().to_path_buf();

        // Call the existing constructor
        Self::new(config)
    }

    /// Load or reload a scenario at runtime
    ///
    /// This method allows loading a new scenario while the game is running.
    /// It resets the game state to MainMenu and clears the existing runtime.
    ///
    /// # Arguments
    /// * `path` - Path to the scenario TOML file to load
    ///
    /// # Errors
    /// Returns `EngineError` if the scenario file cannot be loaded or parsed
    ///
    /// # Example
    /// ```no_run
    /// use narrative_game::components::GameRootElement;
    /// use narrative_engine::EngineConfig;
    ///
    /// let mut root = GameRootElement::new(EngineConfig::default());
    /// root.load_scenario("assets/scenarios/chapter_02.toml")?;
    /// # Ok::<(), narrative_engine::error::EngineError>(())
    /// ```
    pub fn load_scenario<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), narrative_engine::error::EngineError> {
        use narrative_engine::runtime::ScenarioRuntime;

        tracing::info!("Loading scenario: {}", path.as_ref().display());

        // Load the scenario runtime
        let mut runtime = ScenarioRuntime::from_toml(path.as_ref())?;

        // Start the runtime
        runtime.start()?;

        // Reset to main menu state with the new scenario
        self.app_state = AppState::MainMenu(MainMenuState::default());
        self.scenario_runtime = Some(runtime);
        self.previous_in_game_state = None;
        tracing::debug!("children_dirty set at line {}", line!());
        self.children_dirty = true;
        self.bgm_started = false; // Reset BGM flag when loading new scenario

        tracing::info!("Scenario loaded successfully");
        Ok(())
    }

    /// Set texture IDs for default game assets
    ///
    /// This should be called after the Window loads the default assets
    pub fn set_texture_ids(&mut self, background_id: u64, character_id: u64) {
        self.background_texture_id = Some(background_id);
        self.character_texture_id = Some(character_id);
        // Set initial background as current background
        self.current_background_texture_id = Some(background_id);
        tracing::debug!(
            "GameRoot: Set texture IDs - background: {}, character: {}",
            background_id,
            character_id
        );
    }
}
