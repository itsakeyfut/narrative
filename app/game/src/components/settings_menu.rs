//! Settings menu UI element
//!
//! Provides UI for adjusting game settings including:
//! - Text speed control
//! - Auto-play speed control
//! - Audio volumes
//! - Display options (fullscreen)
//!
//! Settings are persisted in RON format to `assets/config/settings.ron`.

use narrative_core::config::{COMMON_RESOLUTIONS, UserSettings};
use narrative_engine::AudioManager;
use narrative_gui::components::common::{
    Button, ButtonVariant, DropdownItem, DropdownMenu, Slider, Toggle, ToggleStyle,
};
use narrative_gui::framework::animation::AnimationContext;
use narrative_gui::framework::element::{
    Element, ElementId, LayoutContext, PaintContext, WindowOperation,
};
use narrative_gui::framework::input::InputEvent;
use narrative_gui::framework::layout::{Bounds, Point};
use narrative_gui::theme::{colors, font_size, spacing};
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use taffy::NodeId;

/// Total number of child elements in settings menu
/// (7 sliders + 2 toggles + 1 resolution button + 1 back button)
const EXPECTED_CHILDREN_COUNT: usize = 11;

/// Shared state for settings menu (single mutex reduces lock contention and complexity)
struct SettingsState {
    settings: UserSettings,
    settings_changed: bool,
    back_pressed: bool,
    open_resolution_dropdown: bool,
    resolution_children_dirty: bool,
    window_operations: Vec<WindowOperation>,
}

/// Settings menu element
pub struct SettingsMenuElement {
    id: ElementId,
    layout_node: Option<NodeId>,
    /// Shared state (single mutex for all state)
    state: Arc<Mutex<SettingsState>>,
    /// Audio manager for real-time volume control
    audio_manager: Arc<Mutex<AudioManager>>,
    /// Child elements (sliders, toggles, buttons)
    children: Vec<Box<dyn Element>>,
    /// Whether children need rebuilding
    children_dirty: bool,
    /// Animation context for global settings
    animation_context: AnimationContext,
    /// Component-specific animation override (None = follow global)
    animations_enabled: Option<bool>,
    /// Resolution dropdown menu
    resolution_dropdown: DropdownMenu,
    /// Resolution button bounds (for dropdown positioning)
    resolution_button_bounds: Option<Bounds>,
}

impl SettingsMenuElement {
    /// Create a new settings menu
    pub fn new(settings: UserSettings, audio_manager: Arc<Mutex<AudioManager>>) -> Self {
        let state = Arc::new(Mutex::new(SettingsState {
            settings,
            settings_changed: false,
            back_pressed: false,
            open_resolution_dropdown: false,
            resolution_children_dirty: false,
            window_operations: Vec::new(),
        }));

        // Setup resolution dropdown callback
        let state_clone = Arc::clone(&state);
        let resolution_dropdown = DropdownMenu::new().with_on_item_click(move |item_id| {
            // Parse resolution from item_id (format: "WIDTHxHEIGHT")
            if let Some((width_str, height_str)) = item_id.split_once('x')
                && let (Ok(width), Ok(height)) =
                    (width_str.parse::<u32>(), height_str.parse::<u32>())
            {
                tracing::info!(
                    "Resolution changed to: {}x{} (applying immediately)",
                    width,
                    height
                );
                if let Ok(mut state) = state_clone.lock() {
                    state.settings.display.resolution = (width, height);
                    state.settings_changed = true;
                    state.resolution_children_dirty = true;
                    state
                        .window_operations
                        .push(WindowOperation::Resize { width, height });
                    tracing::debug!("Settings updated: resolution = {}x{}", width, height);
                } else {
                    tracing::warn!("Failed to lock state for resolution update");
                }
            }
        });

        Self {
            id: ElementId::new(),
            layout_node: None,
            state,
            audio_manager,
            children: Vec::new(),
            children_dirty: true,
            animation_context: AnimationContext::default(),
            animations_enabled: None,
            resolution_dropdown,
            resolution_button_bounds: None,
        }
    }

    /// Set the animation context
    pub fn with_animation_context(mut self, context: AnimationContext) -> Self {
        self.animation_context = context;
        self
    }

    /// Set component-specific animation override
    pub fn with_animations_enabled(mut self, enabled: impl Into<Option<bool>>) -> Self {
        self.animations_enabled = enabled.into();
        self
    }

    /// Check if settings have changed and return them if so (also clears the changed flag)
    pub fn take_settings_if_changed(&self) -> Option<UserSettings> {
        let mut state = self.state.lock().ok()?;
        if state.settings_changed {
            state.settings_changed = false;
            Some(state.settings.clone())
        } else {
            None
        }
    }

    /// Check if back button was pressed (also clears the flag)
    pub fn take_back_pressed(&self) -> bool {
        let mut state = match self.state.lock() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to lock state mutex: {}", e);
                return false;
            }
        };
        let result = state.back_pressed;
        state.back_pressed = false;
        result
    }

    /// Take window operations (called by parent to get queued operations)
    pub fn take_window_operations(&mut self) -> Vec<WindowOperation> {
        if let Ok(mut state) = self.state.lock() {
            std::mem::take(&mut state.window_operations)
        } else {
            tracing::warn!("Failed to lock state for window operations");
            Vec::new()
        }
    }

    /// Build child elements
    fn rebuild_children(&mut self) {
        self.children.clear();

        // --- Text Speed Slider ---
        let text_speed = self
            .state
            .lock()
            .map(|s| {
                // Convert TextSpeed enum to numeric value
                match s.settings.text.speed {
                    narrative_core::TextSpeed::Slow => 15.0,
                    narrative_core::TextSpeed::Normal => 30.0,
                    narrative_core::TextSpeed::Fast => 60.0,
                    narrative_core::TextSpeed::Instant => 200.0,
                }
            })
            .unwrap_or(30.0);

        let state_arc = Arc::clone(&self.state);

        let text_slider = Slider::new("Text Speed (characters/second)", 1.0, 200.0)
            .with_value(text_speed)
            .with_step(1.0)
            .with_width(400.0)
            .with_on_change(move |value| {
                if let Ok(mut state) = state_arc.lock() {
                    state.settings.text.speed = if value <= 20.0 {
                        narrative_core::TextSpeed::Slow
                    } else if value <= 45.0 {
                        narrative_core::TextSpeed::Normal
                    } else if value <= 100.0 {
                        narrative_core::TextSpeed::Fast
                    } else {
                        narrative_core::TextSpeed::Instant
                    };
                    state.settings_changed = true;
                }
            });

        self.children.push(Box::new(text_slider));

        // --- Auto-Play Speed Slider ---
        let auto_wait = self
            .state
            .lock()
            .map(|s| s.settings.text.auto_wait)
            .unwrap_or(2.0);

        let state_arc = Arc::clone(&self.state);

        let auto_slider = Slider::new("Auto-Play Speed (seconds)", 0.5, 10.0)
            .with_value(auto_wait)
            .with_step(0.5)
            .with_width(400.0)
            .with_on_change(move |value| {
                if let Ok(mut state) = state_arc.lock() {
                    state.settings.text.auto_wait = value;
                    state.settings_changed = true;
                }
            });

        self.children.push(Box::new(auto_slider));

        // --- Master Volume Slider ---
        let master_volume = self
            .state
            .lock()
            .map(|s| s.settings.audio.master_volume)
            .unwrap_or(1.0);

        let audio_arc = Arc::clone(&self.audio_manager);
        let state_arc = Arc::clone(&self.state);

        let master_slider = Slider::new("Master Volume", 0.0, 1.0)
            .with_value(master_volume)
            .with_step(0.05)
            .with_width(400.0)
            .with_on_change(move |value| {
                // Update audio manager for real-time feedback
                if let Ok(mut audio) = audio_arc.lock()
                    && let Err(e) = audio.set_master_volume(value)
                {
                    tracing::error!("Failed to set master volume: {}", e);
                }
                // Update settings
                if let Ok(mut state) = state_arc.lock() {
                    state.settings.audio.master_volume = value;
                    state.settings_changed = true;
                }
            });

        self.children.push(Box::new(master_slider));

        // --- Music (BGM) Volume Slider ---
        let music_volume = self
            .state
            .lock()
            .map(|s| s.settings.audio.bgm_volume)
            .unwrap_or(0.7);

        let audio_arc = Arc::clone(&self.audio_manager);
        let state_arc = Arc::clone(&self.state);

        let music_slider = Slider::new("Music Volume", 0.0, 1.0)
            .with_value(music_volume)
            .with_step(0.05)
            .with_width(400.0)
            .with_on_change(move |value| {
                if let Ok(mut audio) = audio_arc.lock()
                    && let Err(e) = audio.set_music_volume(value)
                {
                    tracing::error!("Failed to set music volume: {}", e);
                }
                if let Ok(mut state) = state_arc.lock() {
                    state.settings.audio.bgm_volume = value;
                    state.settings_changed = true;
                }
            });

        self.children.push(Box::new(music_slider));

        // --- Sound Effects Volume Slider ---
        let sound_volume = self
            .state
            .lock()
            .map(|s| s.settings.audio.se_volume)
            .unwrap_or(1.0);

        let audio_arc = Arc::clone(&self.audio_manager);
        let state_arc = Arc::clone(&self.state);

        let sound_slider = Slider::new("Sound Effects Volume", 0.0, 1.0)
            .with_value(sound_volume)
            .with_step(0.05)
            .with_width(400.0)
            .with_on_change(move |value| {
                if let Ok(mut audio) = audio_arc.lock()
                    && let Err(e) = audio.set_sound_volume(value)
                {
                    tracing::error!("Failed to set sound volume: {}", e);
                }
                if let Ok(mut state) = state_arc.lock() {
                    state.settings.audio.se_volume = value;
                    state.settings_changed = true;
                }
            });

        self.children.push(Box::new(sound_slider));

        // --- Voice Volume Slider ---
        let voice_volume = self
            .state
            .lock()
            .map(|s| s.settings.audio.voice_volume)
            .unwrap_or(1.0);

        let audio_arc = Arc::clone(&self.audio_manager);
        let state_arc = Arc::clone(&self.state);

        let voice_slider = Slider::new("Voice Volume", 0.0, 1.0)
            .with_value(voice_volume)
            .with_step(0.05)
            .with_width(400.0)
            .with_on_change(move |value| {
                if let Ok(mut audio) = audio_arc.lock()
                    && let Err(e) = audio.set_voice_volume(value)
                {
                    tracing::error!("Failed to set voice volume: {}", e);
                }
                if let Ok(mut state) = state_arc.lock() {
                    state.settings.audio.voice_volume = value;
                    state.settings_changed = true;
                }
            });

        self.children.push(Box::new(voice_slider));

        // --- Fullscreen Toggle ---
        let fullscreen = self
            .state
            .lock()
            .map(|s| s.settings.display.fullscreen)
            .unwrap_or(false);

        let state_arc = Arc::clone(&self.state);

        let fullscreen_toggle = Toggle::new("Fullscreen", fullscreen)
            .with_style(ToggleStyle::Switch)
            .with_width(400.0)
            .with_on_change(move |value| {
                if let Ok(mut state) = state_arc.lock() {
                    state.settings.display.fullscreen = value;
                    state.settings_changed = true;
                }
            });

        self.children.push(Box::new(fullscreen_toggle));

        // --- Resolution Selection Button ---
        let resolution_display = self
            .state
            .lock()
            .map(|s| s.settings.display.resolution_display_name())
            .unwrap_or_else(|_| "1920x1080 (1080p Full HD)".to_string());

        let state_arc = Arc::clone(&self.state);
        let resolution_button = Button::new(format!("Resolution: {}", resolution_display))
            .with_variant(ButtonVariant::Secondary)
            .with_on_click(move || {
                if let Ok(mut state) = state_arc.lock() {
                    state.open_resolution_dropdown = true;
                }
            });

        self.children.push(Box::new(resolution_button));

        // --- Animation Enabled Toggle ---
        let animations_enabled = self
            .state
            .lock()
            .map(|s| s.settings.animation.enabled)
            .unwrap_or(true);

        let state_arc = Arc::clone(&self.state);

        let animation_toggle = Toggle::new("Enable Animations", animations_enabled)
            .with_style(ToggleStyle::Switch)
            .with_width(400.0)
            .with_on_change(move |value| {
                if let Ok(mut state) = state_arc.lock() {
                    state.settings.animation.enabled = value;
                    state.settings_changed = true;
                }
            });

        self.children.push(Box::new(animation_toggle));

        // --- Animation Speed Slider ---
        let animation_speed = self
            .state
            .lock()
            .map(|s| s.settings.animation.speed)
            .unwrap_or(1.0);

        let state_arc = Arc::clone(&self.state);

        let speed_slider = Slider::new("Animation Speed", 0.5, 2.0)
            .with_value(animation_speed)
            .with_step(0.1)
            .with_width(400.0)
            .with_on_change(move |value| {
                if let Ok(mut state) = state_arc.lock() {
                    state.settings.animation.speed = value;
                    state.settings_changed = true;
                }
            });

        self.children.push(Box::new(speed_slider));

        // --- Back Button ---
        let state_arc = Arc::clone(&self.state);
        let back_button = Button::new("Back")
            .with_variant(ButtonVariant::Secondary)
            .with_on_click(move || {
                if let Ok(mut state) = state_arc.lock() {
                    state.back_pressed = true;
                }
            });

        self.children.push(Box::new(back_button));

        self.children_dirty = false;
    }
}

impl Element for SettingsMenuElement {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout_node(&self) -> Option<NodeId> {
        self.layout_node
    }

    fn set_layout_node(&mut self, node: NodeId) {
        self.layout_node = Some(node);
    }

    fn layout(&mut self, _cx: &mut LayoutContext) -> taffy::Style {
        use taffy::prelude::*;

        // Rebuild children if needed
        if self.children_dirty {
            self.rebuild_children();
        }

        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: Some(AlignItems::Center),
            justify_content: Some(JustifyContent::Center),
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            gap: Size {
                width: LengthPercentage::length(0.0),
                height: LengthPercentage::length(spacing::LG),
            },
            padding: taffy::Rect::length(spacing::XXL),
            ..Default::default()
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        let bounds = cx.bounds;

        // Background
        cx.fill_rect(bounds, colors::BG_DARKEST);

        // Title
        let title = "Settings";
        let title_x =
            bounds.x() + (bounds.width() - title.len() as f32 * font_size::TITLE * 0.6) / 2.0;
        let title_y = bounds.y() + spacing::XXL + font_size::TITLE;
        cx.draw_text(
            title,
            Point::new(title_x, title_y),
            colors::TEXT_PRIMARY,
            font_size::TITLE,
        );

        // Paint dropdown overlay (must be last to render on top)
        if self.resolution_dropdown.is_open() {
            self.resolution_dropdown.paint_overlay(cx);
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Bounds) -> bool {
        // Rebuild if needed
        if self.children_dirty {
            return false; // Will rebuild on next layout
        }

        // Check if dropdown should be opened
        if let Ok(mut state) = self.state.lock()
            && state.open_resolution_dropdown
        {
            state.open_resolution_dropdown = false;
            if let Some(button_bounds) = self.resolution_button_bounds {
                // Create dropdown items from common resolutions
                let items: Vec<DropdownItem> = COMMON_RESOLUTIONS
                    .iter()
                    .map(|(width, height, label)| {
                        DropdownItem::new(format!("{}x{}", width, height), *label)
                    })
                    .collect();

                self.resolution_dropdown.open(button_bounds, items);
            }
        }

        // Handle dropdown events first (if open, it has priority)
        if self.resolution_dropdown.is_open()
            && self.resolution_dropdown.handle_event(event, bounds)
        {
            return true;
        }

        // Calculate child bounds manually (column layout, centered, with gap spacing::LG)
        if self.children.len() >= EXPECTED_CHILDREN_COUNT {
            let content_x = bounds.x() + spacing::XXL;
            let content_y = bounds.y() + spacing::XXL;
            let content_width = bounds.width() - spacing::XXL * 2.0;
            let content_height = bounds.height() - spacing::XXL * 2.0;

            // Element dimensions
            let slider_width = 400.0;
            let slider_height = 40.0;
            let toggle_width = 400.0;
            let toggle_height = 40.0;
            let button_width = 400.0; // Resolution button width
            let button_height = 40.0;
            let back_button_width = 100.0;

            // Total content height (7 sliders + 2 toggles + 1 resolution button + 1 back button + 10 gaps)
            let total_content_height = slider_height * 7.0
                + toggle_height * 2.0
                + button_height * 2.0
                + spacing::LG * 10.0;

            // Center vertically in content area
            let start_y = content_y + (content_height - total_content_height) / 2.0;

            // Element x position (centered horizontally)
            let element_x = content_x + (content_width - slider_width) / 2.0;

            // Calculate bounds for each element
            let mut y_offset = start_y;

            // Text speed slider
            let bounds_0 = Bounds::new(element_x, y_offset, slider_width, slider_height);
            y_offset += slider_height + spacing::LG;

            // Auto-play speed slider
            let bounds_1 = Bounds::new(element_x, y_offset, slider_width, slider_height);
            y_offset += slider_height + spacing::LG;

            // Master volume slider
            let bounds_2 = Bounds::new(element_x, y_offset, slider_width, slider_height);
            y_offset += slider_height + spacing::LG;

            // Music volume slider
            let bounds_3 = Bounds::new(element_x, y_offset, slider_width, slider_height);
            y_offset += slider_height + spacing::LG;

            // Sound volume slider
            let bounds_4 = Bounds::new(element_x, y_offset, slider_width, slider_height);
            y_offset += slider_height + spacing::LG;

            // Voice volume slider
            let bounds_5 = Bounds::new(element_x, y_offset, slider_width, slider_height);
            y_offset += slider_height + spacing::LG;

            // Fullscreen toggle
            let bounds_6 = Bounds::new(element_x, y_offset, toggle_width, toggle_height);
            y_offset += toggle_height + spacing::LG;

            // Resolution button
            let bounds_7 = Bounds::new(element_x, y_offset, button_width, button_height);
            self.resolution_button_bounds = Some(bounds_7); // Save for dropdown positioning
            y_offset += button_height + spacing::LG;

            // Animation enabled toggle
            let bounds_8 = Bounds::new(element_x, y_offset, toggle_width, toggle_height);
            y_offset += toggle_height + spacing::LG;

            // Animation speed slider
            let bounds_9 = Bounds::new(element_x, y_offset, slider_width, slider_height);
            y_offset += slider_height + spacing::LG;

            // Back button (centered)
            let back_x = content_x + (content_width - back_button_width) / 2.0;
            let bounds_10 = Bounds::new(back_x, y_offset, back_button_width, button_height);

            // Forward events to children
            let child_bounds = [
                bounds_0, bounds_1, bounds_2, bounds_3, bounds_4, bounds_5, bounds_6, bounds_7,
                bounds_8, bounds_9, bounds_10,
            ];

            for (i, child_bounds) in child_bounds.iter().enumerate() {
                if let Some(child) = self.children.get_mut(i)
                    && child.handle_event(event, *child_bounds)
                {
                    return true;
                }
            }
        }

        false
    }

    fn tick(&mut self, delta: Duration) -> bool {
        let mut needs_update = false;

        // Check if resolution changed and children need rebuilding
        if let Ok(mut state) = self.state.lock()
            && state.resolution_children_dirty
        {
            state.resolution_children_dirty = false;
            self.children_dirty = true;
            needs_update = true;
        }

        for child in &mut self.children {
            if child.tick(delta) {
                needs_update = true;
            }
        }
        needs_update
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn children(&self) -> &[Box<dyn Element>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Element>] {
        &mut self.children
    }
}
