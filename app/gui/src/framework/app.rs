//! Application lifecycle and event loop

use super::element::Element;
use super::error::{FrameworkError, FrameworkResult};
use super::menu::{AppMenu, MenuEventHandler, MenuId};
use super::window::{Window, WindowOptions, convert_winit_event};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

/// Application context for global state
pub struct AppContext {
    // Future: global state, observers, etc.
}

impl AppContext {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Callback type for menu event handling
pub type MenuEventCallback = Box<dyn Fn(MenuId) + Send + Sync>;

/// Callback type for window initialization
pub type WindowInitCallback = Box<dyn FnOnce(&mut Window) + Send>;

/// The main application struct
pub struct App {
    window: Option<Window>,
    window_options: WindowOptions,
    root_builder: Option<Box<dyn FnOnce() -> Box<dyn Element> + Send>>,
    modifiers: winit::keyboard::ModifiersState,
    menu: Option<AppMenu>,
    menu_event_handler: Option<MenuEventHandler>,
    on_menu_event: Option<MenuEventCallback>,
    on_window_created: Option<WindowInitCallback>,
}

impl App {
    /// Create a new application
    pub fn new(options: WindowOptions) -> Self {
        Self {
            window: None,
            window_options: options,
            root_builder: None,
            modifiers: winit::keyboard::ModifiersState::empty(),
            menu: None,
            menu_event_handler: None,
            on_menu_event: None,
            on_window_created: None,
        }
    }

    /// Set the root element builder
    pub fn with_root<F>(mut self, builder: F) -> Self
    where
        F: FnOnce() -> Box<dyn Element> + Send + 'static,
    {
        self.root_builder = Some(Box::new(builder));
        self
    }

    /// Enable the native menu bar
    pub fn with_menu(mut self) -> Self {
        self.menu = Some(AppMenu::new());
        self.menu_event_handler = Some(MenuEventHandler::new());
        self
    }

    /// Set menu event callback
    pub fn with_menu_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(MenuId) + Send + Sync + 'static,
    {
        self.on_menu_event = Some(Box::new(handler));
        self
    }

    /// Set window initialization callback (called after window creation)
    pub fn on_window_created<F>(mut self, callback: F) -> Self
    where
        F: FnOnce(&mut Window) + Send + 'static,
    {
        self.on_window_created = Some(Box::new(callback));
        self
    }

    /// Run the application
    pub fn run(self) -> FrameworkResult<()> {
        let event_loop = EventLoop::new().map_err(|e| FrameworkError::EventLoop(e.to_string()))?;

        // Use Poll for continuous rendering (60+ FPS target for video editing)
        // Issue #250: Changed from Wait to Poll for better FPS
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app_handler = AppHandler {
            app: self,
            pending_init: true,
        };

        event_loop
            .run_app(&mut app_handler)
            .map_err(|e| FrameworkError::EventLoop(e.to_string()))
    }
}

struct AppHandler {
    app: App,
    pending_init: bool,
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.pending_init {
            self.pending_init = false;

            // Create window
            let window_attrs = winit::window::WindowAttributes::default()
                .with_title(&self.app.window_options.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.app.window_options.width,
                    self.app.window_options.height,
                ))
                .with_resizable(self.app.window_options.resizable)
                .with_decorations(self.app.window_options.decorations);

            let winit_window = match event_loop.create_window(window_attrs) {
                Ok(w) => Arc::new(w),
                Err(e) => {
                    tracing::error!("Failed to create window: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            // Create our window wrapper (blocking on async)
            let options = self.app.window_options.clone();
            let window_result = pollster::block_on(Window::new(winit_window.clone(), &options));

            match window_result {
                Ok(mut window) => {
                    // Set root element if provided
                    if let Some(builder) = self.app.root_builder.take() {
                        let root = builder();
                        window.set_root(root);
                    }

                    // Call window initialization callback if provided
                    if let Some(callback) = self.app.on_window_created.take() {
                        callback(&mut window);
                    }

                    // Initialize native menu bar if enabled
                    if let Some(ref menu) = self.app.menu {
                        menu.init_for_window(&winit_window);
                        tracing::info!("Native menu bar initialized");
                    }

                    self.app.window = Some(window);
                }
                Err(e) => {
                    tracing::error!("Failed to initialize window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &mut self.app.window else {
            return;
        };

        match &event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                window.resize(*size);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.app.modifiers = modifiers.state();
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = window.render() {
                    tracing::error!("Render error: {}", e);
                }
                // Process any pending window operations after rendering
                if window.process_window_operations() {
                    // Window close was requested
                    event_loop.exit();
                    return;
                }
            }
            _ => {
                // Log file drop events for debugging
                if matches!(
                    event,
                    WindowEvent::DroppedFile(_)
                        | WindowEvent::HoveredFile(_)
                        | WindowEvent::HoveredFileCancelled
                ) {
                    tracing::debug!("File event received: {:?}", event);
                }

                // Convert and dispatch input events
                if let Some(input_event) = convert_winit_event(&event, &self.app.modifiers) {
                    window.handle_input(input_event);
                }
            }
        }

        // Request redraw if needed
        if window.needs_redraw() {
            window.winit_window().request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Process menu events
        if let Some(ref handler) = self.app.menu_event_handler {
            while let Some(menu_id) = handler.try_recv() {
                tracing::debug!("Menu event: {:?}", menu_id);

                // Handle built-in menu actions
                match menu_id {
                    MenuId::Exit => {
                        tracing::info!("Exit requested via menu");
                        event_loop.exit();
                        return;
                    }
                    _ => {
                        // Delegate to user-provided handler
                        if let Some(ref callback) = self.app.on_menu_event {
                            callback(menu_id);
                        }
                    }
                }
            }
        }

        // Issue #250: Always request redraw for continuous rendering
        // This enables smooth 60+ FPS for video playback preview
        if let Some(window) = &self.app.window {
            window.winit_window().request_redraw();
        }
    }
}
