//! Example: Text Rendering
//!
//! This example demonstrates Japanese text rendering using cosmic-text,
//! GlyphCache, and the wgpu-based renderer.
//!
//! This validates Issue #11: TextLayout implementation
//!
//! Usage:
//!   cargo run --example text_rendering

use narrative_core::{Color, Point};
use narrative_engine::render::{RenderCommand, RenderLayer, Renderer};
use std::sync::Arc as StdArc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

/// Application state
struct AppState {
    window: StdArc<Window>,
    renderer: Renderer,
    first_frame: bool,
}

/// Application handler
struct App {
    state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        // Create window
        let window_attributes = Window::default_attributes()
            .with_title("Text Rendering Example - Issue #11 Validation")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => StdArc::new(w),
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Initialize renderer
        let mut renderer = match pollster::block_on(Renderer::new(window.clone())) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to initialize renderer: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Load Japanese font
        let font_path = match std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
        {
            Some(root) => root.join("assets/fonts/DotGothic16/DotGothic16-Regular.ttf"),
            None => {
                eprintln!("Failed to find workspace root");
                event_loop.exit();
                return;
            }
        };

        if let Err(e) = load_japanese_font(&mut renderer, &font_path) {
            eprintln!("Failed to load Japanese font: {}", e);
            eprintln!("Font path: {:?}", font_path);
            event_loop.exit();
            return;
        }

        println!("Japanese font loaded successfully");

        self.state = Some(AppState {
            window,
            renderer,
            first_frame: true,
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
            WindowEvent::RedrawRequested => {
                // Create render commands
                let commands = create_text_commands(&state.renderer);

                // Debug: Print number of render commands (only once)
                if state.first_frame {
                    println!("Rendering {} commands", commands.len());
                    state.first_frame = false;
                }

                // Render using commands
                match state.renderer.render_commands(&commands) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Render error: {}", e);
                    }
                }

                // Request next frame
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

/// Load Japanese font into the renderer's font manager
fn load_japanese_font(
    renderer: &mut Renderer,
    font_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    renderer.load_font(font_path)?;
    Ok(())
}

/// Create render commands for text rendering demonstration
fn create_text_commands(_renderer: &Renderer) -> Vec<RenderCommand> {
    let mut commands = Vec::new();

    // Set UI layer
    commands.push(RenderCommand::SetLayer(RenderLayer::UI));

    // Test 1: Simple Japanese text (large size)
    commands.push(RenderCommand::DrawText {
        text: StdArc::from("こんにちは、世界！"),
        position: Point::new(100.0, 100.0),
        font_size: 32.0,
        line_height: 32.0 * 1.4,
        color: Color::new(1.0, 1.0, 1.0, 1.0), // White
        visible_chars: None,
    });

    // Test 2: Mixed Japanese and English (medium size)
    commands.push(RenderCommand::DrawText {
        text: StdArc::from("Hello 世界 123 テスト"),
        position: Point::new(100.0, 160.0),
        font_size: 24.0,
        line_height: 24.0 * 1.4,
        color: Color::new(0.5, 1.0, 0.5, 1.0), // Light green
        visible_chars: None,
    });

    // Test 3: Longer text (standard size)
    commands.push(RenderCommand::DrawText {
        text: StdArc::from("ビジュアルノベルエンジン - Visual Novel Engine"),
        position: Point::new(100.0, 210.0),
        font_size: 20.0,
        line_height: 20.0 * 1.4,
        color: Color::new(1.0, 0.8, 0.0, 1.0), // Yellow
        visible_chars: None,
    });

    // Test 4: Multiline demonstration (compact line height)
    commands.push(RenderCommand::DrawText {
        text: StdArc::from("一行目のテキスト"),
        position: Point::new(100.0, 260.0),
        font_size: 18.0,
        line_height: 18.0 * 1.2,               // Tighter line spacing
        color: Color::new(0.8, 0.8, 1.0, 1.0), // Light blue
        visible_chars: None,
    });

    commands.push(RenderCommand::DrawText {
        text: StdArc::from("二行目のテキスト"),
        position: Point::new(100.0, 282.0),
        font_size: 18.0,
        line_height: 18.0 * 1.2,
        color: Color::new(0.8, 0.8, 1.0, 1.0), // Light blue
        visible_chars: None,
    });

    commands.push(RenderCommand::DrawText {
        text: StdArc::from("三行目のテキスト"),
        position: Point::new(100.0, 304.0),
        font_size: 18.0,
        line_height: 18.0 * 1.2,
        color: Color::new(0.8, 0.8, 1.0, 1.0), // Light blue
        visible_chars: None,
    });

    // Test 5: Small size demonstration
    commands.push(RenderCommand::DrawText {
        text: StdArc::from("TextLayout Implementation ✓"),
        position: Point::new(100.0, 380.0),
        font_size: 16.0,
        line_height: 16.0 * 1.5,
        color: Color::new(0.0, 1.0, 0.0, 1.0), // Green
        visible_chars: None,
    });

    // Test 6: Issue #11 validation (extra large)
    commands.push(RenderCommand::DrawText {
        text: StdArc::from("Issue #11: テキストレイアウト実装完了"),
        position: Point::new(100.0, 450.0),
        font_size: 28.0,
        line_height: 28.0 * 1.5,
        color: Color::new(1.0, 0.5, 0.0, 1.0), // Orange
        visible_chars: None,
    });

    commands
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Text Rendering Example ===");
    println!("Issue #11 Validation: TextLayout Implementation");
    println!();
    println!("This example demonstrates:");
    println!("  - Text layout calculation");
    println!("  - Text wrapping (if needed)");
    println!("  - Line spacing and character spacing");
    println!("  - Japanese text rendering");
    println!();
    println!("Expected output:");
    println!("  - Multiple lines of Japanese and mixed text");
    println!("  - Correctly positioned and colored text");
    println!("  - Smooth rendering at 60 FPS");
    println!();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App { state: None };

    event_loop.run_app(&mut app)?;

    Ok(())
}
