//! Window management using winit and wgpu
//!
//! Issue #250: Optimized for 60+ FPS with metrics tracking
//! Phase 2: Frame pacing, VSync support, and batched draw calls

use super::Color;
use super::dirty::DirtyTracker;
use super::element::{Element, LayoutContext, PaintContext};
use super::error::{FrameworkError, FrameworkResult};
use super::input::{InputEvent, InputState, Modifiers, MouseButton};
use super::layout::{Bounds, LayoutEngine, Point, Size};
use super::metrics::{FrameMetrics, PerformanceStats};
use super::renderer::{BatchBuilder, DrawCommand, Renderer, ZLayer};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::window::Window as WinitWindow;

/// Present mode for frame synchronization
///
/// Issue #250 Phase 2: Frame pacing support
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PresentMode {
    /// VSync enabled - limits to display refresh rate, no tearing
    #[default]
    VSync,
    /// Adaptive VSync - VSync when above target, immediate when below
    Adaptive,
    /// No VSync - lowest latency, may cause tearing
    Immediate,
    /// Mailbox - triple buffering, lowest latency with VSync
    Mailbox,
}

impl From<PresentMode> for wgpu::PresentMode {
    fn from(mode: PresentMode) -> Self {
        match mode {
            // Use Fifo for VSync - guaranteed to be supported on all platforms
            PresentMode::VSync => wgpu::PresentMode::Fifo,
            // FifoRelaxed allows tearing when below refresh rate (adaptive sync)
            PresentMode::Adaptive => wgpu::PresentMode::FifoRelaxed,
            PresentMode::Immediate => wgpu::PresentMode::Immediate,
            PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
        }
    }
}

/// Options for creating a window
#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub decorations: bool,
    /// Present mode for frame synchronization (Issue #250 Phase 2)
    pub present_mode: PresentMode,
    /// Target FPS for frame pacing (0 = unlimited)
    pub target_fps: u32,
    /// Show FPS overlay (Issue #250)
    pub show_fps_overlay: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "Narrative".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
            decorations: true,
            present_mode: PresentMode::VSync,
            target_fps: 60,
            show_fps_overlay: cfg!(debug_assertions),
        }
    }
}

/// Context passed to elements during event handling and rendering
pub struct WindowContext<'a> {
    pub size: Size,
    pub input_state: &'a InputState,
    pub layout_engine: &'a mut LayoutEngine,
}

/// A window that can render UI elements
pub struct Window {
    winit_window: Arc<WinitWindow>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    layout_engine: LayoutEngine,
    input_state: InputState,
    root_element: Option<Box<dyn Element>>,
    background_color: Color,
    needs_redraw: bool,
    /// Whether the layout needs to be recalculated (incremental layout optimization)
    /// See Issue #250 for GUI optimization tracking
    needs_layout: bool,
    /// Performance metrics (Issue #250)
    metrics: FrameMetrics,
    /// Dirty tracking for partial redraws (Issue #250)
    dirty_tracker: DirtyTracker,
    /// Whether to show FPS overlay
    show_fps_overlay: bool,
    /// Frame pacing configuration (Issue #250 Phase 2)
    target_frame_time: Option<Duration>,
    /// Last frame end time for frame pacing
    last_frame_time: Instant,
    /// Current present mode
    present_mode: PresentMode,
}

impl Window {
    /// Create a new window
    ///
    /// Issue #250 Phase 2: Now uses WindowOptions for present mode and frame pacing
    pub async fn new(
        winit_window: Arc<WinitWindow>,
        options: &WindowOptions,
    ) -> FrameworkResult<Self> {
        let size = winit_window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(winit_window.clone())
            .map_err(|e| FrameworkError::GpuInit(e.to_string()))?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| {
                FrameworkError::GpuInit(format!("No suitable GPU adapter found: {}", e))
            })?;

        // Request device
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Narrative GUI Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                ..Default::default()
            })
            .await
            .map_err(|e| FrameworkError::GpuInit(e.to_string()))?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .or_else(|| surface_caps.formats.first().copied())
            .ok_or_else(|| {
                FrameworkError::GpuInit(format!(
                    "No supported surface formats available. Reported formats: {:?}",
                    surface_caps.formats
                ))
            })?;

        let alpha_mode = surface_caps.alpha_modes.first().copied().ok_or_else(|| {
            FrameworkError::GpuInit(format!(
                "No supported alpha modes available. Reported modes: {:?}",
                surface_caps.alpha_modes
            ))
        })?;

        // Issue #250 Phase 2: Use present mode from options, with fallback
        let requested_present_mode: wgpu::PresentMode = options.present_mode.into();
        let present_mode = if surface_caps.present_modes.contains(&requested_present_mode) {
            tracing::info!(
                "Using present mode {:?} (supported modes: {:?})",
                requested_present_mode,
                surface_caps.present_modes
            );
            requested_present_mode
        } else {
            // Fifo is guaranteed to be supported, use it as fallback
            let fallback = wgpu::PresentMode::Fifo;
            tracing::info!(
                "Requested present mode {:?} not available, using {:?} (supported: {:?})",
                requested_present_mode,
                fallback,
                surface_caps.present_modes
            );
            fallback
        };

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Create renderer
        let renderer = Renderer::new(device, queue, surface_format, width, height);

        // Calculate target frame time from target FPS
        let target_frame_time = if options.target_fps > 0 {
            Some(Duration::from_secs_f64(1.0 / options.target_fps as f64))
        } else {
            None
        };

        Ok(Self {
            winit_window,
            surface,
            surface_config,
            renderer,
            layout_engine: LayoutEngine::new(),
            input_state: InputState::new(),
            root_element: None,
            background_color: Color::from_hex(0x1a1a1a),
            needs_redraw: true,
            needs_layout: true,
            metrics: FrameMetrics::new(),
            dirty_tracker: DirtyTracker::new(),
            show_fps_overlay: options.show_fps_overlay,
            target_frame_time,
            last_frame_time: Instant::now(),
            present_mode: options.present_mode,
        })
    }

    /// Enable or disable the FPS overlay
    pub fn set_show_fps_overlay(&mut self, show: bool) {
        self.show_fps_overlay = show;
        self.needs_redraw = true;
    }

    /// Get current performance statistics
    pub fn performance_stats(&self) -> PerformanceStats {
        self.metrics.get_stats()
    }

    /// Get the current FPS
    pub fn fps(&self) -> f32 {
        self.metrics.fps()
    }

    /// Set the present mode (Issue #250 Phase 2)
    ///
    /// Changes how frames are synchronized with the display.
    pub fn set_present_mode(&mut self, mode: PresentMode) {
        if self.present_mode != mode {
            self.present_mode = mode;
            self.surface_config.present_mode = mode.into();
            self.surface
                .configure(self.renderer.device(), &self.surface_config);
            tracing::info!("Present mode changed to {:?}", mode);
        }
    }

    /// Get the current present mode
    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    /// Set target FPS for frame pacing (Issue #250 Phase 2)
    ///
    /// Set to 0 for unlimited FPS.
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_frame_time = if fps > 0 {
            Some(Duration::from_secs_f64(1.0 / fps as f64))
        } else {
            None
        };
        tracing::info!("Target FPS set to {}", if fps > 0 { fps } else { 0 });
    }

    /// Get time until next frame should be rendered (Issue #250 Phase 2)
    ///
    /// Returns `Some(duration)` if we should wait, `None` if we should render immediately.
    pub fn time_until_next_frame(&self) -> Option<Duration> {
        self.target_frame_time.and_then(|target| {
            let elapsed = self.last_frame_time.elapsed();
            if elapsed < target {
                Some(target - elapsed)
            } else {
                None
            }
        })
    }

    /// Check if we should render the next frame (Issue #250 Phase 2)
    ///
    /// For frame pacing, returns true if enough time has passed since last frame.
    pub fn should_render(&self) -> bool {
        match self.target_frame_time {
            Some(target) => self.last_frame_time.elapsed() >= target,
            None => true, // No target = always ready
        }
    }

    /// Set the root element
    pub fn set_root(&mut self, element: Box<dyn Element>) {
        self.root_element = Some(element);
        self.needs_layout = true;
        self.needs_redraw = true;
        self.dirty_tracker.request_full_relayout();
        self.dirty_tracker.request_full_redraw();
    }

    /// Set the background color
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
        self.needs_redraw = true;
        self.dirty_tracker.request_full_redraw();
    }

    /// Handle a window resize
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface
                .configure(self.renderer.device(), &self.surface_config);
            self.renderer.resize(new_size.width, new_size.height);
            self.needs_layout = true;
            self.needs_redraw = true;
            // Full relayout and redraw needed on resize
            self.dirty_tracker.request_full_relayout();
            self.dirty_tracker.request_full_redraw();
        }
    }

    /// Handle an input event
    pub fn handle_input(&mut self, event: InputEvent) {
        // For mouse button events, fill in the current mouse position
        // (winit's MouseInput event doesn't include position)
        let event = match event {
            InputEvent::MouseDown {
                button,
                position: _,
                modifiers,
            } => InputEvent::MouseDown {
                button,
                position: self.input_state.mouse_position,
                modifiers,
            },
            InputEvent::MouseUp {
                button,
                position: _,
                modifiers,
            } => InputEvent::MouseUp {
                button,
                position: self.input_state.mouse_position,
                modifiers,
            },
            InputEvent::MouseScroll {
                delta,
                position: _,
                modifiers,
            } => InputEvent::MouseScroll {
                delta,
                position: self.input_state.mouse_position,
                modifiers,
            },
            other => other,
        };

        self.input_state.handle_event(&event);

        // Dispatch to root element
        if let Some(root) = &mut self.root_element {
            let bounds = Bounds::new(
                0.0,
                0.0,
                self.surface_config.width as f32,
                self.surface_config.height as f32,
            );
            if root.handle_event(&event, bounds) {
                self.needs_redraw = true;
            }
        }

        // Always request redraw on mouse move for hover effects
        if matches!(event, InputEvent::MouseMove { .. }) {
            self.needs_redraw = true;
        }
    }

    /// Check if the window needs to be redrawn
    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    /// Request a redraw
    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
        self.winit_window.request_redraw();
    }

    /// Render the window
    ///
    /// Issue #250: Includes performance metrics tracking
    /// Issue #250 Phase 2: Uses BatchBuilder for optimized draw call ordering
    pub fn render(&mut self) -> FrameworkResult<()> {
        // Start frame timing (Issue #250)
        self.metrics.begin_frame();

        // Calculate delta time for frame-rate independent animations
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;

        // Call tick on root element for time-based updates
        // Common cases where tick returns true:
        // - Children structure changed (rebuild_children was called)
        // - Animation frame updated (typewriter effect progressed, video frame advanced)
        // - Internal state changed requiring visual update
        if let Some(root) = &mut self.root_element
            && root.tick(delta)
        {
            // If tick returns true, request both redraw and relayout
            // Layout is needed because children may have changed structure or content
            self.needs_redraw = true;
            self.needs_layout = true;
        }

        // Load pending background textures (if any)
        if let Some(root) = &mut self.root_element
            && root.load_pending_background_texture(&mut self.renderer)
        {
            self.needs_redraw = true;
        }

        // Get surface texture
        let output = self.surface.get_current_texture().map_err(|e| match e {
            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                self.resize(self.winit_window.inner_size());
                FrameworkError::Render("Surface lost, reconfigured".to_string())
            }
            e => FrameworkError::Render(e.to_string()),
        })?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Issue #250 Phase 2: Use BatchBuilder for optimized command ordering
        let mut batch = BatchBuilder::with_capacity(256);

        if let Some(root) = &mut self.root_element {
            // Layout phase - only recalculate if needed (incremental layout optimization)
            let available_size = Size::new(
                self.surface_config.width as f32,
                self.surface_config.height as f32,
            );

            if self.needs_layout {
                // Start layout timing (Issue #250)
                self.metrics.begin_layout();

                // Issue #250: Incremental layout optimization
                // Only force a full rebuild if this is the first layout or after resize
                // Otherwise, reuse existing layout nodes
                let force_rebuild = self.dirty_tracker.needs_full_relayout();

                if force_rebuild {
                    // Full rebuild needed - create new layout engine
                    self.layout_engine = LayoutEngine::new();
                }

                let root_node =
                    build_layout_tree(root.as_mut(), &mut self.layout_engine, force_rebuild)?;
                self.layout_engine
                    .compute_layout(root_node, available_size)?;
                self.needs_layout = false;

                // End layout timing (Issue #250)
                self.metrics.end_layout();
            }

            // Paint phase - recursively paint all elements
            // Start paint timing (Issue #250)
            self.metrics.begin_paint();

            // Use window bounds for root element (root should fill the window)
            // Issue #250 Phase 2: Use paint_element_tree_batched for layer support
            let window_bounds = Bounds::new(0.0, 0.0, available_size.width, available_size.height);
            paint_element_tree_batched(
                root.as_ref(),
                window_bounds,
                &self.layout_engine,
                &mut batch,
                ZLayer::DEFAULT,
            );

            // Paint overlay content (popups, dropdowns) at POPUP layer
            paint_overlay_tree_batched(
                root.as_ref(),
                window_bounds,
                &self.layout_engine,
                &mut batch,
            );

            // End paint timing (Issue #250)
            self.metrics.end_paint();
        }

        // Add FPS overlay if enabled (at DEBUG layer - always on top)
        if self.show_fps_overlay {
            let stats = self.metrics.get_stats();
            batch.rect_at_layer(
                Bounds::new(8.0, 8.0, 280.0, 24.0),
                Color::new(0.0, 0.0, 0.0, 0.7),
                4.0,
                ZLayer::DEBUG,
            );
            batch.text_at_layer(
                format!(
                    "FPS: {:.0} | Frame: {:.2}ms | Draws: {}",
                    stats.fps, stats.avg_frame_time_ms, stats.draw_calls
                ),
                Point::new(12.0, 24.0),
                Color::WHITE,
                12.0,
                ZLayer::DEBUG,
            );
        }

        // GPU submit timing (Issue #250)
        self.metrics.begin_gpu_submit();

        // Issue #250 Phase 2: Render using batched commands for optimized draw order
        let batch_stats = self
            .renderer
            .render_batched(&view, batch, self.background_color);

        // Record accurate draw call metrics from BatchStats (Issue #250 Phase 2)
        self.metrics.record_draw_calls(batch_stats.draw_calls);
        self.metrics.record_quad_count(batch_stats.quad_count);

        // Present
        output.present();

        // End GPU submit and frame timing (Issue #250)
        self.metrics.end_gpu_submit();
        self.metrics.end_frame();

        // Log FPS periodically for performance monitoring (debug builds only)
        #[cfg(debug_assertions)]
        {
            // Using 120 frames as interval (roughly 2 seconds at 60 FPS or 1 second at 120 FPS)
            let total_frames = self.metrics.total_frames();
            if total_frames > 0 && total_frames.is_multiple_of(120) {
                let stats = self.metrics.get_stats();
                tracing::debug!(
                    "FPS: {:.1} | Frame: {:.2}ms | Draw calls: {}",
                    stats.fps,
                    stats.avg_frame_time_ms,
                    stats.draw_calls
                );
            }
        }

        // Clear dirty tracker for next frame
        self.dirty_tracker.clear();

        Ok(())
    }

    /// Get the window size
    pub fn size(&self) -> Size {
        Size::new(
            self.surface_config.width as f32,
            self.surface_config.height as f32,
        )
    }

    /// Get the winit window
    pub fn winit_window(&self) -> &WinitWindow {
        &self.winit_window
    }

    /// Get the renderer (for advanced use)
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    /// Get mutable renderer (for advanced use)
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    /// Load default game assets (backgrounds and characters)
    ///
    /// Loads assets from `assets/config/dev_assets.ron` if available.
    /// Falls back to placeholder textures if assets are missing (graceful degradation).
    ///
    /// Returns (background_texture_id, character_texture_id) on success
    pub fn load_default_assets(
        &mut self,
    ) -> Result<(u64, u64), crate::framework::renderer::RendererError> {
        use crate::framework::dev_assets::DevAssetConfig;

        // Load configuration from RON file
        let config = DevAssetConfig::load().unwrap_or_else(|| {
            tracing::warn!("assets/config/dev_assets.ron not found, using placeholders");
            DevAssetConfig::default()
        });

        tracing::info!("Loading development assets: {:?}", config);

        // Load or create background texture
        let bg_id = if let Some(bg_path) = &config.background {
            match self.renderer.load_texture_from_path(bg_path) {
                Ok(id) => {
                    tracing::info!(
                        "Loaded background texture: {} (ID: {})",
                        bg_path.display(),
                        id
                    );
                    id
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load background {}: {}. Using placeholder.",
                        bg_path.display(),
                        e
                    );
                    // Create placeholder: dark blue/gray checkerboard for background
                    self.renderer.create_placeholder_texture(
                        1280,
                        720,
                        [30, 40, 50, 255], // Dark blue-gray
                        [40, 50, 60, 255], // Slightly lighter
                    )?
                }
            }
        } else {
            tracing::info!("No background configured, using placeholder");
            self.renderer.create_placeholder_texture(
                1280,
                720,
                [30, 40, 50, 255],
                [40, 50, 60, 255],
            )?
        };

        // Load or create character texture
        let char_id = if let Some(char_path) = &config.character {
            match self.renderer.load_texture_from_path(char_path) {
                Ok(id) => {
                    tracing::info!(
                        "Loaded character texture: {} (ID: {})",
                        char_path.display(),
                        id
                    );
                    id
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load character {}: {}. Using placeholder.",
                        char_path.display(),
                        e
                    );
                    // Create placeholder: pink/purple checkerboard for character
                    self.renderer.create_placeholder_texture(
                        512,
                        1024,
                        [200, 100, 200, 255], // Pink
                        [180, 80, 180, 255],  // Purple
                    )?
                }
            }
        } else {
            tracing::info!("No character configured, using placeholder");
            self.renderer.create_placeholder_texture(
                512,
                1024,
                [200, 100, 200, 255],
                [180, 80, 180, 255],
            )?
        };

        Ok((bg_id, char_id))
    }

    /// Get mutable reference to root element
    pub fn root_element_mut(&mut self) -> Option<&mut Box<dyn Element>> {
        self.root_element.as_mut()
    }

    /// Process pending window operations from the root element
    ///
    /// This should be called after rendering to handle resize/maximize requests.
    /// Returns true if the window should be closed.
    pub fn process_window_operations(&mut self) -> bool {
        use super::element::WindowOperation;

        let mut should_close = false;

        if let Some(root) = &mut self.root_element {
            let operations = root.take_window_operations();
            for op in operations {
                match op {
                    WindowOperation::Resize { width, height } => {
                        tracing::info!("Processing window resize to {}x{}", width, height);
                        let _ = self
                            .winit_window
                            .request_inner_size(winit::dpi::LogicalSize::new(width, height));
                    }
                    WindowOperation::Maximize => {
                        tracing::info!("Processing window maximize");
                        self.winit_window.set_maximized(true);
                    }
                    WindowOperation::Restore => {
                        tracing::info!("Processing window restore");
                        self.winit_window.set_maximized(false);
                    }
                    WindowOperation::ToggleMaximize => {
                        let is_maximized = self.winit_window.is_maximized();
                        tracing::info!(
                            "Processing window toggle maximize (currently: {})",
                            is_maximized
                        );
                        self.winit_window.set_maximized(!is_maximized);
                    }
                    WindowOperation::Minimize => {
                        tracing::info!("Processing window minimize");
                        self.winit_window.set_minimized(true);
                    }
                    WindowOperation::Close => {
                        tracing::info!("Processing window close");
                        should_close = true;
                    }
                    WindowOperation::Center => {
                        tracing::info!("Processing window center");
                        // Center window on current monitor
                        if let Some(monitor) = self.winit_window.current_monitor() {
                            let monitor_size = monitor.size();
                            let window_size = self.winit_window.outer_size();
                            let x = (monitor_size.width.saturating_sub(window_size.width)) / 2;
                            let y = (monitor_size.height.saturating_sub(window_size.height)) / 2;
                            let position = monitor.position();
                            self.winit_window.set_outer_position(
                                winit::dpi::PhysicalPosition::new(
                                    position.x + x as i32,
                                    position.y + y as i32,
                                ),
                            );
                        }
                    }
                    WindowOperation::SetDecorations(enabled) => {
                        tracing::info!("Processing window set decorations: {}", enabled);
                        self.winit_window.set_decorations(enabled);
                    }
                    WindowOperation::DragWindow => {
                        tracing::debug!("Processing window drag");
                        if let Err(e) = self.winit_window.drag_window() {
                            tracing::warn!("Failed to start window drag: {}", e);
                        }
                    }
                }
            }
        }

        should_close
    }
}

/// Convert winit events to framework events
pub fn convert_winit_event(
    event: &winit::event::WindowEvent,
    modifiers: &winit::keyboard::ModifiersState,
) -> Option<InputEvent> {
    match event {
        winit::event::WindowEvent::CursorMoved { position, .. } => Some(InputEvent::MouseMove {
            position: Point::new(position.x as f32, position.y as f32),
            modifiers: (*modifiers).into(),
        }),
        winit::event::WindowEvent::MouseInput { state, button, .. } => {
            let button = MouseButton::from(*button);
            match state {
                winit::event::ElementState::Pressed => Some(InputEvent::MouseDown {
                    button,
                    position: Point::ZERO, // Will be updated by state
                    modifiers: (*modifiers).into(),
                }),
                winit::event::ElementState::Released => Some(InputEvent::MouseUp {
                    button,
                    position: Point::ZERO,
                    modifiers: (*modifiers).into(),
                }),
            }
        }
        winit::event::WindowEvent::MouseWheel { delta, .. } => {
            let delta = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => Point::new(*x * 20.0, *y * 20.0),
                winit::event::MouseScrollDelta::PixelDelta(pos) => {
                    Point::new(pos.x as f32, pos.y as f32)
                }
            };
            Some(InputEvent::MouseScroll {
                delta,
                position: Point::ZERO,
                modifiers: (*modifiers).into(),
            })
        }
        winit::event::WindowEvent::KeyboardInput { event, .. } => {
            let key = match &event.physical_key {
                winit::keyboard::PhysicalKey::Code(code) => (*code).into(),
                _ => super::input::KeyCode::Unknown,
            };
            match event.state {
                winit::event::ElementState::Pressed => Some(InputEvent::KeyDown {
                    key,
                    modifiers: (*modifiers).into(),
                }),
                winit::event::ElementState::Released => Some(InputEvent::KeyUp {
                    key,
                    modifiers: (*modifiers).into(),
                }),
            }
        }
        winit::event::WindowEvent::Focused(focused) => {
            if *focused {
                Some(InputEvent::Focus)
            } else {
                Some(InputEvent::Blur)
            }
        }
        winit::event::WindowEvent::DroppedFile(path) => {
            Some(InputEvent::DroppedFile { path: path.clone() })
        }
        winit::event::WindowEvent::HoveredFile(path) => {
            Some(InputEvent::HoveredFile { path: path.clone() })
        }
        winit::event::WindowEvent::HoveredFileCancelled => Some(InputEvent::HoveredFileCancelled),
        _ => None,
    }
}

/// Recursively build the taffy layout tree from an element tree
///
/// Issue #250: Supports incremental layout by reusing existing layout nodes
fn build_layout_tree(
    element: &mut dyn Element,
    engine: &mut LayoutEngine,
    force_rebuild: bool,
) -> Result<taffy::NodeId, FrameworkError> {
    use super::element::LayoutContext;

    // Create layout context
    let mut layout_cx = LayoutContext {
        available_size: Size::new(0.0, 0.0), // Will be set during compute
        layout_engine: engine,
    };

    // Get the element's style
    let style = element.layout(&mut layout_cx);

    // Build children first
    let mut child_nodes = Vec::new();
    for child in element.children_mut().iter_mut() {
        child_nodes.push(build_layout_tree(child.as_mut(), engine, force_rebuild)?);
    }

    // Issue #250: Incremental layout - reuse existing node if possible
    let node = match (force_rebuild, element.layout_node()) {
        (false, Some(existing_node)) => {
            // Reuse existing node and update its style and children
            engine.set_style(existing_node, style)?;
            if !child_nodes.is_empty() {
                engine.set_children(existing_node, &child_nodes)?;
            }
            existing_node
        }
        _ => {
            // Create new node (either force rebuild or no existing node)
            let node = if child_nodes.is_empty() {
                engine.new_node(style)?
            } else {
                engine.new_node_with_children(style, &child_nodes)?
            };
            // Store node reference in element
            element.set_layout_node(node);
            node
        }
    };

    Ok(node)
}

/// Recursively paint an element tree
fn paint_element_tree(
    element: &dyn Element,
    bounds: Bounds,
    engine: &LayoutEngine,
    commands: &mut Vec<DrawCommand>,
) {
    use super::element::PaintContext;

    // Paint this element
    let mut paint_cx = PaintContext {
        bounds,
        clip_bounds: None,
        commands,
    };
    element.paint(&mut paint_cx);

    // Paint children with their computed bounds
    for child in element.children() {
        if let Some(child_node) = child.layout_node() {
            match engine.get_bounds(child_node) {
                Ok(child_layout) => {
                    // Child bounds are relative to parent, so offset by parent position
                    let child_bounds = Bounds::new(
                        bounds.x() + child_layout.x(),
                        bounds.y() + child_layout.y(),
                        child_layout.width(),
                        child_layout.height(),
                    );
                    paint_element_tree(child.as_ref(), child_bounds, engine, commands);
                }
                Err(e) => {
                    tracing::error!("Failed to get child bounds: {}", e);
                }
            }
        }
    }
}

/// Recursively paint an element tree using BatchBuilder for optimized draw ordering
///
/// Issue #250 Phase 2: Supports z-layer based rendering for proper draw order optimization.
/// Elements paint to a temporary Vec which is then added to the batch at the specified layer.
fn paint_element_tree_batched(
    element: &dyn Element,
    bounds: Bounds,
    engine: &LayoutEngine,
    batch: &mut BatchBuilder,
    layer: ZLayer,
) {
    use super::element::PaintContext;

    // Collect commands for this element into a temporary Vec
    let mut commands = Vec::new();
    let mut paint_cx = PaintContext {
        bounds,
        clip_bounds: None,
        commands: &mut commands,
    };
    element.paint(&mut paint_cx);

    // Add collected commands to batch at the specified layer
    for cmd in commands {
        batch.push_at_layer(cmd, layer);
    }

    // Paint children with their computed bounds
    for child in element.children() {
        if let Some(child_node) = child.layout_node() {
            match engine.get_bounds(child_node) {
                Ok(child_layout) => {
                    // Child bounds are relative to parent, so offset by parent position
                    let child_bounds = Bounds::new(
                        bounds.x() + child_layout.x(),
                        bounds.y() + child_layout.y(),
                        child_layout.width(),
                        child_layout.height(),
                    );
                    // Children inherit parent's layer by default
                    paint_element_tree_batched(child.as_ref(), child_bounds, engine, batch, layer);
                }
                Err(e) => {
                    tracing::error!("Failed to get child bounds: {}", e);
                }
            }
        }
    }
}

/// Paint overlay content (popups, dropdowns) at POPUP layer
///
/// This collects overlay commands from all elements in the tree and adds them
/// to the batch at the POPUP layer, ensuring they render on top of normal content.
fn paint_overlay_tree_batched(
    element: &dyn Element,
    bounds: Bounds,
    engine: &LayoutEngine,
    batch: &mut BatchBuilder,
) {
    use super::element::PaintContext;

    // Collect overlay commands for this element
    let mut commands = Vec::new();
    let mut paint_cx = PaintContext {
        bounds,
        clip_bounds: None,
        commands: &mut commands,
    };
    element.paint_overlay(&mut paint_cx);

    // Add collected overlay commands at POPUP layer
    for cmd in commands {
        batch.push_at_layer(cmd, ZLayer::POPUP);
    }

    // Recursively collect overlay commands from children
    for child in element.children() {
        if let Some(child_node) = child.layout_node() {
            match engine.get_bounds(child_node) {
                Ok(child_layout) => {
                    let child_bounds = Bounds::new(
                        bounds.x() + child_layout.x(),
                        bounds.y() + child_layout.y(),
                        child_layout.width(),
                        child_layout.height(),
                    );
                    paint_overlay_tree_batched(child.as_ref(), child_bounds, engine, batch);
                }
                Err(e) => {
                    tracing::error!("Failed to get child bounds for overlay: {}", e);
                }
            }
        }
    }
}
