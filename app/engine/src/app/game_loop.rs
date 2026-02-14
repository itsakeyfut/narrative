//! Game loop
//!
//! # Frame Rate Control Strategy
//!
//! The game loop uses a dual-layer approach for 60 FPS stability:
//!
//! 1. **Primary: VSync (PresentMode::Fifo)**
//!    - Enabled in renderer configuration (see `renderer.rs:130`)
//!    - Provides hardware-level frame synchronization
//!    - Ensures stable 60 FPS on most displays
//!
//! 2. **Secondary: Sleep-based throttling**
//!    - Fine-tunes timing if frames finish early
//!    - Note: `std::thread::sleep()` has ~1-15ms precision (OS-dependent)
//!    - Prevents busy-waiting and reduces CPU usage
//!
//! This combination provides both stability (VSync) and efficiency (sleep).

use crate::app::EngineConfig;
use crate::error::EngineResult;
use crate::input::InputHandler;
use crate::render::{RenderCommand, Renderer};
use crate::runtime::{AppState, InGameState, ScenarioRuntime};
use narrative_core::{Color, Point};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

// Game loop configuration constants
const LOADING_DURATION: f32 = 1.0; // seconds

/// Game loop state
struct GameLoopState {
    window: Arc<Window>,
    renderer: Renderer,
    input: InputHandler,
    app_state: AppState,
    scenario_runtime: Option<ScenarioRuntime>,
    last_frame_time: Instant,
    delta_time: f32,
    frame_count: u64,
    fps_update_timer: f32,
    current_fps: f32,
}

/// Game loop application handler
struct GameLoopApp {
    config: EngineConfig,
    state: Option<GameLoopState>,
}

impl ApplicationHandler for GameLoopApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        // Create window
        let window_attributes = Window::default_attributes()
            .with_title(self.config.window_title())
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.window_width(),
                self.config.window_height(),
            ));

        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                tracing::error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Initialize renderer (async)
        // Use pollster to block on async wgpu initialization in synchronous resumed() callback
        let renderer = match pollster::block_on(Renderer::new(window.clone())) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to initialize renderer: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Initialize input handler
        let input = InputHandler::new();

        // Initialize application state (starts in Loading)
        let app_state = AppState::default();

        // Initialize frame timing
        let now = Instant::now();

        self.state = Some(GameLoopState {
            window,
            renderer,
            input,
            app_state,
            scenario_runtime: None,
            last_frame_time: now,
            delta_time: 0.0,
            frame_count: 0,
            fps_update_timer: 0.0,
            current_fps: 0.0,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.renderer.resize(physical_size);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Extract KeyCode from PhysicalKey
                if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                    state.input.process_keyboard_event(key_code, event.state);
                }
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                state.input.process_mouse_button_event(button, button_state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.input.process_mouse_motion(position.x, position.y);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                state.input.process_modifiers(modifiers.state());
            }
            WindowEvent::RedrawRequested => {
                // ========== Frame Timing ==========
                let now = Instant::now();
                let frame_duration = now.duration_since(state.last_frame_time);
                state.delta_time = frame_duration.as_secs_f32();
                state.last_frame_time = now;

                // Update FPS counter
                state.frame_count = state.frame_count.saturating_add(1);
                state.fps_update_timer += state.delta_time;
                if state.fps_update_timer >= 1.0 {
                    state.current_fps = state.frame_count as f32 / state.fps_update_timer;
                    state.frame_count = 0;
                    state.fps_update_timer = 0.0;
                    tracing::trace!("FPS: {:.1}", state.current_fps);
                }

                // ========== Update Phase ==========
                // Clear frame-specific input state (just_pressed, just_released)
                state.input.update();

                // Update application state
                update_app_state(
                    &mut state.app_state,
                    &mut state.scenario_runtime,
                    state.input.state(),
                    state.delta_time,
                    &self.config,
                );

                // ========== Render Phase ==========
                // Render dialogue with typewriter effect
                let result = render_dialogue(&mut state.renderer, &state.app_state).map_err(|e| {
                    tracing::error!("Render error: {}", e);
                    // Convert to a generic surface error for consistent handling
                    wgpu::SurfaceError::Timeout
                });

                match result {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        // Reconfigure surface on lost
                        state.renderer.resize(state.window.inner_size());
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        tracing::error!("Out of memory!");
                        event_loop.exit();
                    }
                    Err(e) => {
                        tracing::error!("Render error: {:?}", e);
                    }
                }

                // ========== Frame Rate Control ==========
                // Primary frame rate control: VSync (PresentMode::Fifo) in renderer
                // Secondary control: sleep for fine-tuning if frame finishes early
                let target_frame_time =
                    Duration::from_secs_f32(1.0 / self.config.target_fps() as f32);
                let elapsed = now.elapsed();

                if elapsed < target_frame_time {
                    let sleep_duration = target_frame_time.saturating_sub(elapsed);
                    // Note: std::thread::sleep() precision varies by OS:
                    // - Windows: ~1-15ms (can be improved with timeBeginPeriod)
                    // - Linux: ~1-2ms (depends on kernel timer resolution)
                    // - macOS: ~1ms (mach_wait_until provides nanosecond precision)
                    // For 60FPS (16.67ms), VSync provides the primary stability,
                    // and sleep() prevents CPU busy-waiting between frames.
                    std::thread::sleep(sleep_duration);
                }

                // Request next frame
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

/// Game loop
pub struct GameLoop {
    config: EngineConfig,
}

impl GameLoop {
    /// Create a new game loop with default configuration
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    /// Create a new game loop with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Run the game loop
    pub fn run(self) -> EngineResult<()> {
        let event_loop = EventLoop::new().map_err(|e| {
            crate::error::EngineError::GameLoop(format!("Failed to create event loop: {}", e))
        })?;

        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = GameLoopApp {
            config: self.config,
            state: None,
        };

        event_loop
            .run_app(&mut app)
            .map_err(|e| crate::error::EngineError::GameLoop(format!("Event loop error: {}", e)))?;

        Ok(())
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// UI Layout Constants
// =============================================================================

/// Dialogue box height in pixels
const DIALOGUE_BOX_HEIGHT: f32 = 200.0;

/// Text padding from dialogue box edges in pixels
const DIALOGUE_TEXT_PADDING: f32 = 20.0;

/// Dialogue text font size in pixels
const DIALOGUE_FONT_SIZE: f32 = 24.0;

/// Dialogue text line height in pixels
const DIALOGUE_LINE_HEIGHT: f32 = 32.0;

/// Dialogue box background color (semi-transparent black)
const DIALOGUE_BOX_BG_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.8);

/// Dialogue box corner radius (0.0 = sharp corners)
const DIALOGUE_BOX_CORNER_RADIUS: f32 = 0.0;

// =============================================================================
// Render Functions
// =============================================================================

/// Render dialogue with typewriter effect
fn render_dialogue(renderer: &mut Renderer, app_state: &AppState) -> EngineResult<()> {
    // Build render commands based on app state
    let mut commands = Vec::new();

    // Only add dialogue if in InGame state with Typing
    if let AppState::InGame(InGameState::Typing(typing)) = app_state {
        // Get screen dimensions for positioning
        let config = renderer.surface_config();
        let screen_w = config.width as f32;
        let screen_h = config.height as f32;

        // Dialogue box position (bottom of screen)
        let dialogue_box_y = screen_h - DIALOGUE_BOX_HEIGHT;

        // Draw dialogue box background
        commands.push(RenderCommand::DrawRect {
            rect: narrative_core::Rect::new(0.0, dialogue_box_y, screen_w, DIALOGUE_BOX_HEIGHT),
            color: DIALOGUE_BOX_BG_COLOR,
            corner_radius: DIALOGUE_BOX_CORNER_RADIUS,
        });

        // Draw speaker name if present
        // TODO: Add speaker name rendering in future enhancement

        // Draw dialogue text with typewriter effect
        commands.push(RenderCommand::DrawText {
            text: typing.text.clone(),
            position: Point::new(
                DIALOGUE_TEXT_PADDING,
                dialogue_box_y + DIALOGUE_TEXT_PADDING,
            ),
            font_size: DIALOGUE_FONT_SIZE,
            line_height: DIALOGUE_LINE_HEIGHT,
            color: Color::WHITE,
            visible_chars: Some(typing.char_index), // Typewriter effect!
        });
    }

    // Render all commands
    renderer.render_commands(&commands)
}

/// Update application state
///
/// This function handles state transitions and updates based on the current state,
/// integrating with the ScenarioRuntime to execute scenario commands.
fn update_app_state(
    app_state: &mut AppState,
    scenario_runtime: &mut Option<ScenarioRuntime>,
    input: &crate::input::InputState,
    delta: f32,
    config: &EngineConfig,
) {
    use crate::runtime::{InGameState, MainMenuState, WaitingInputState};

    match app_state {
        AppState::Loading(loading) => {
            // Update loading progress
            loading.progress += delta / LOADING_DURATION;
            loading.set_progress(loading.progress);

            // Transition to main menu when loading completes
            if loading.progress >= 1.0 {
                tracing::info!("Loading complete, transitioning to main menu");
                *app_state = AppState::MainMenu(MainMenuState::default());
            }
        }
        AppState::MainMenu(_menu) => {
            // Auto-start scenario when entering main menu
            // In a full implementation, this would wait for user to click "Start"
            if scenario_runtime.is_none() {
                tracing::info!("Loading scenario: {}", config.start_scenario.display());
                match ScenarioRuntime::from_toml(&config.start_scenario) {
                    Ok(mut runtime) => {
                        if let Err(e) = runtime.start() {
                            tracing::error!("Failed to start scenario: {}", e);
                            tracing::warn!("Staying in MainMenu due to scenario start failure");
                            // Stay in MainMenu state - user can retry or exit
                            return;
                        }

                        // Execute commands until we reach a waiting state (Dialogue, Choice, etc.)
                        if let Some(initial_state) = execute_and_transition(&mut runtime) {
                            *scenario_runtime = Some(runtime);
                            *app_state = AppState::InGame(initial_state);
                            tracing::info!("Scenario started successfully");
                        } else {
                            tracing::error!("Failed to create initial state from command");
                            tracing::warn!(
                                "Staying in MainMenu - scenario has no valid initial command"
                            );
                            // Stay in MainMenu - scenario is invalid
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to load scenario file '{}': {}",
                            config.start_scenario.display(),
                            e
                        );
                        tracing::warn!("Staying in MainMenu - please check scenario file path");
                        // Stay in MainMenu state - file not found or parse error
                    }
                }
            }
        }
        AppState::InGame(in_game_state) => {
            let Some(runtime) = scenario_runtime.as_mut() else {
                tracing::error!("InGame state without runtime!");
                return;
            };

            match in_game_state {
                InGameState::Typing(typing) => {
                    // Typewriter effect: display characters one by one
                    typing.elapsed += delta;

                    let text_len = typing.text.chars().count();

                    // Calculate character delay from text speed (chars/sec -> seconds/char)
                    let char_delay = if config.gameplay.text_speed > 0.0 {
                        1.0 / config.gameplay.text_speed
                    } else {
                        0.0 // Instant display if speed is 0
                    };

                    // Progress typewriter
                    while typing.elapsed >= char_delay && typing.char_index < text_len {
                        typing.char_index = typing.char_index.saturating_add(1);
                        typing.elapsed -= char_delay;
                    }

                    // Check if we should transition
                    let should_transition =
                        (!typing.auto_mode || input.clicked()) && typing.char_index >= text_len;

                    // Handle input - skip to end
                    if input.clicked() && typing.char_index < text_len {
                        typing.char_index = text_len;
                        return; // Early return to avoid transition this frame
                    }

                    // Transition to WaitingInput if needed
                    if should_transition {
                        let scene_id = typing.scene_id.clone();
                        let command_index = typing.command_index;
                        *in_game_state = InGameState::WaitingInput(WaitingInputState {
                            scene_id,
                            command_index,
                            auto_wait_elapsed: 0.0,
                            skip_mode: false,
                        });
                    }
                }

                InGameState::WaitingInput(_waiting) => {
                    // Wait for player to click to advance
                    if input.clicked() {
                        // Advance to next command
                        if runtime.advance_command() {
                            // Successfully advanced, execute new command
                            if let Some(new_state) = execute_and_transition(runtime) {
                                *in_game_state = new_state;
                            } else {
                                // No next state after command execution
                                tracing::warn!("No next state after command execution");
                                if runtime.is_ended() {
                                    tracing::info!("Scenario ended");
                                    *app_state = AppState::MainMenu(MainMenuState::default());
                                } else {
                                    // Unexpected: not ended but no next state
                                    tracing::error!(
                                        "Runtime not ended but no next state available"
                                    );
                                    *app_state = AppState::MainMenu(MainMenuState::default());
                                }
                            }
                        } else {
                            // At end of scene/scenario
                            if runtime.is_ended() {
                                tracing::info!("Scenario ended");
                                *app_state = AppState::MainMenu(MainMenuState::default());
                            }
                        }
                    }
                }

                InGameState::ShowingChoices(choice_state) => {
                    // Handle choice navigation
                    if input.up_pressed() && choice_state.selected > 0 {
                        choice_state.selected = choice_state.selected.saturating_sub(1);
                    }
                    if input.down_pressed()
                        && choice_state.selected < choice_state.choices.len().saturating_sub(1)
                    {
                        choice_state.selected = choice_state.selected.saturating_add(1);
                    }

                    // Handle choice selection
                    if input.clicked() && !choice_state.confirmed {
                        choice_state.confirmed = true;

                        // Execute choice in runtime
                        if let Err(e) = runtime.select_choice(choice_state.selected) {
                            tracing::error!("Failed to select choice: {}", e);
                            return;
                        }

                        // Create new state from next command
                        if let Some(new_state) = create_state_from_command(runtime) {
                            *in_game_state = new_state;
                        } else {
                            tracing::error!("Failed to create state after choice");
                        }
                    }
                }

                InGameState::Transition(transition) => {
                    transition.update(delta);
                    if transition.is_complete() {
                        // Transition complete, move to next command
                        if let Some(new_state) = execute_and_transition(runtime) {
                            *in_game_state = new_state;
                        } else {
                            // End of scenario or error
                            tracing::info!("Scenario ended after transition");
                            *app_state = AppState::MainMenu(MainMenuState::default());
                        }
                    }
                }

                InGameState::PlayingEffect(effect) => {
                    if effect.update(delta) {
                        // Effect complete, move to next command
                        if let Some(new_state) = execute_and_transition(runtime) {
                            *in_game_state = new_state;
                        } else {
                            // End of scenario or error
                            tracing::info!("Scenario ended after effect");
                            *app_state = AppState::MainMenu(MainMenuState::default());
                        }
                    }
                }

                InGameState::Waiting(wait) => {
                    if wait.update(delta) {
                        // Wait complete, move to next command
                        if let Some(new_state) = execute_and_transition(runtime) {
                            *in_game_state = new_state;
                        } else {
                            // End of scenario or error
                            tracing::info!("Scenario ended after wait");
                            *app_state = AppState::MainMenu(MainMenuState::default());
                        }
                    }
                }

                InGameState::PauseMenu(_pause) => {
                    // Pause menu handling would go here
                    if input.pause_pressed() {
                        // Return to game (unpause)
                        if let Some(new_state) = create_state_from_command(runtime) {
                            *in_game_state = new_state;
                        }
                    }
                }

                InGameState::SaveLoadMenu(_save_load) => {
                    // Save/Load menu handling would go here
                }

                InGameState::Backlog(_backlog) => {
                    // Backlog UI is handled in the GUI layer (GameRootElement)
                    // No game loop logic needed here
                }

                InGameState::CgGallery(_cg_gallery) => {
                    // CG Gallery UI is handled in the GUI layer (GameRootElement)
                    // No game loop logic needed here
                }

                InGameState::CgViewer(_cg_viewer) => {
                    // CG Viewer UI is handled in the GUI layer (GameRootElement)
                    // No game loop logic needed here
                }
            }
        }
        AppState::Settings(_settings) => {
            // Settings menu handling would go here
        }
    }
}

/// Create InGameState from the current command in the runtime
fn create_state_from_command(runtime: &ScenarioRuntime) -> Option<InGameState> {
    use crate::runtime::{ChoiceState, InGameState, TypingState, WaitState};
    use narrative_core::ScenarioCommand;

    let command = runtime.get_current_command()?;
    let scene_id = runtime.current_scene()?.clone();
    let command_index = runtime.command_index();

    match command {
        ScenarioCommand::Dialogue { dialogue } => {
            use narrative_core::Speaker;

            // Convert Speaker enum to Option<String>
            let speaker = match &dialogue.speaker {
                Speaker::Character(name) => Some(name.clone()),
                Speaker::Narrator | Speaker::System => None,
            };

            Some(InGameState::Typing(TypingState {
                scene_id,
                command_index,
                speaker,
                text: Arc::from(dialogue.text.clone()),
                char_index: 0,
                elapsed: 0.0,
                auto_mode: false,
                skip_mode: false,
            }))
        }

        ScenarioCommand::ShowChoice { choice } => Some(InGameState::ShowingChoices(ChoiceState {
            scene_id,
            command_index,
            choices: choice.options.clone(),
            selected: 0,
            confirmed: false,
        })),

        ScenarioCommand::Wait { duration } => Some(InGameState::Waiting(WaitState::new(*duration))),

        // Other commands don't create waiting states, they execute immediately
        _ => None,
    }
}

/// Execute current command and transition to next state
fn execute_and_transition(runtime: &mut ScenarioRuntime) -> Option<InGameState> {
    use crate::runtime::{ChoiceState, CommandExecutionResult, InGameState, WaitState};

    // Execute current command
    let result = match runtime.execute_current_command() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Command execution failed: {}", e);
            return None;
        }
    };

    match result {
        CommandExecutionResult::Continue => {
            // Advance to next command
            if !runtime.advance_command() {
                // End of scene
                return None;
            }

            // Create state from new command
            create_state_from_command(runtime)
        }

        CommandExecutionResult::SceneChanged {
            exit_transition,
            entry_transition,
        } => {
            // TODO: Handle transitions
            if let Some(exit) = exit_transition {
                tracing::info!("Exit transition: {:?} ({:.1}s)", exit.kind, exit.duration);
            }
            if let Some(entry) = entry_transition {
                tracing::info!(
                    "Entry transition: {:?} ({:.1}s)",
                    entry.kind,
                    entry.duration
                );
            }
            // Scene changed, get first command of new scene
            create_state_from_command(runtime)
        }

        CommandExecutionResult::ShowChoices(choices) => {
            let scene_id = runtime.current_scene()?.clone();
            let command_index = runtime.command_index();

            Some(InGameState::ShowingChoices(ChoiceState {
                scene_id,
                command_index,
                choices,
                selected: 0,
                confirmed: false,
            }))
        }

        CommandExecutionResult::Wait(duration) => {
            Some(InGameState::Waiting(WaitState::new(duration)))
        }

        CommandExecutionResult::End => {
            tracing::info!("Scenario ended");
            None
        }
    }
}
