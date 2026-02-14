//! Integration tests for text rendering with cosmic-text

use narrative_core::{Color, Point};
use narrative_engine::text::{FontManager, GlyphCache, TextLayout, TextStyle, TextureAtlas};
use std::path::PathBuf;
use std::sync::Arc;

/// Helper function to create a test wgpu device for GPU tests
async fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find adapter");

    adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("Test Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            experimental_features: Default::default(),
            trace: Default::default(),
        })
        .await
        .expect("Failed to create device")
}

/// Get the path to the test font file
fn get_font_path() -> PathBuf {
    // Get the workspace root directory
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = PathBuf::from(manifest_dir);
    let workspace_root = manifest_path
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to find workspace root");

    workspace_root.join("assets/fonts/DotGothic16/DotGothic16-Regular.ttf")
}

#[test]
fn test_font_manager_load_japanese_font() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");

    // Load DotGothic16 font
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    // Verify font families are available
    let families = font_manager.list_font_families();
    assert!(
        !families.is_empty(),
        "Font families should be available after loading"
    );

    // Check if DotGothic16 is in the list
    assert!(
        families.iter().any(|f| f.contains("DotGothic16")),
        "DotGothic16 should be in the font families list"
    );
}

#[test]
fn test_japanese_text_layout() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    let japanese_text = "こんにちは、世界！".to_string();
    let position = Point::new(0.0, 0.0);
    let style = TextStyle {
        font_size: 16.0,
        line_height: 22.4,
        color: Color::WHITE,
        family: cosmic_text::Family::Name("DotGothic16"),
    };

    let layout = TextLayout::new(
        &mut font_manager,
        Arc::from(japanese_text.clone()),
        position,
        style,
    );

    // Verify layout was created successfully
    assert_eq!(layout.text(), "こんにちは、世界！");
    assert_eq!(layout.position(), position);

    // Verify layout has glyphs
    let glyph_count = layout.glyphs().count();
    assert!(
        glyph_count > 0,
        "Layout should contain glyphs for Japanese text"
    );

    // Verify size calculation
    let size = layout.calculate().expect("Failed to calculate size");
    assert!(size.width > 0.0, "Text width should be greater than 0");
    assert!(size.height > 0.0, "Text height should be greater than 0");
}

#[test]
fn test_text_wrapping() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    let long_text =
        "これは長いテキストです。折り返しのテストを行います。複数行になるはずです。".to_string();
    let position = Point::new(0.0, 0.0);
    let style = TextStyle {
        font_size: 16.0,
        line_height: 22.4,
        color: Color::WHITE,
        family: cosmic_text::Family::Name("DotGothic16"),
    };

    // Create layout with max width for wrapping
    let layout = TextLayout::with_max_width(
        &mut font_manager,
        Arc::from(long_text.clone()),
        position,
        style,
        200.0, // Max width: 200px
    );

    // Verify text wrapping created multiple lines
    assert!(
        layout.lines().len() >= 2,
        "Long text should wrap into multiple lines"
    );

    // Verify each line respects max width (approximately)
    for line in layout.lines() {
        assert!(
            line.width <= 210.0, // Allow small margin
            "Line width should not significantly exceed max width"
        );
    }
}

#[test]
fn test_mixed_text_layout() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    // Mix of Japanese, English, and symbols
    let mixed_text = "Hello世界！123 テスト".to_string();
    let position = Point::new(10.0, 20.0);
    let style = TextStyle {
        font_size: 18.0,
        line_height: 25.2,
        color: Color::new(1.0, 1.0, 1.0, 1.0),
        family: cosmic_text::Family::Name("DotGothic16"),
    };

    let layout = TextLayout::new(
        &mut font_manager,
        Arc::from(mixed_text.clone()),
        position,
        style,
    );

    assert_eq!(layout.text(), mixed_text);

    // Verify glyphs were generated for mixed content
    let glyph_count = layout.glyphs().count();
    assert!(glyph_count > 0, "Mixed text should generate glyphs");

    // Verify size is reasonable
    let size = layout.calculate().expect("Failed to calculate size");
    assert!(size.width > 0.0 && size.height > 0.0);
}

#[test]
fn test_typewriter_effect_with_japanese() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    let text = "ビジュアルノベル".to_string();
    let position = Point::new(0.0, 0.0);
    let style = TextStyle::default();

    let layout = TextLayout::new(&mut font_manager, Arc::from(text), position, style);

    // Test typewriter effect - show first 3 characters
    let visible_glyphs: Vec<_> = layout.visible_glyphs(3).collect();

    assert!(
        !visible_glyphs.is_empty(),
        "Should have visible glyphs for typewriter effect"
    );
    assert!(visible_glyphs.len() <= 3, "Should only show up to 3 glyphs");
}

#[test]
fn test_glyph_cache_with_atlas() {
    pollster::block_on(async {
        let (device, _queue) = create_test_device().await;

        let mut glyph_cache = GlyphCache::new(512).expect("Failed to create glyph cache");
        let mut atlas = TextureAtlas::new(&device, 1024, 1024).expect("Failed to create atlas");

        // Verify initial state
        assert_eq!(glyph_cache.capacity(), 512);
        assert_eq!(atlas.dimensions(), (1024, 1024));

        // Test atlas allocation
        let pos1 = atlas
            .allocate(64, 64)
            .expect("Should allocate space in atlas");
        assert_eq!(pos1, (0, 0));

        let pos2 = atlas
            .allocate(64, 64)
            .expect("Should allocate second space");
        assert_eq!(pos2, (64, 0));

        // Clear cache
        glyph_cache.clear();
    });
}

#[test]
fn test_multiline_japanese_text() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    let multiline_text = "一行目のテキスト\n二行目のテキスト\n三行目のテキスト".to_string();
    let position = Point::new(0.0, 0.0);
    let style = TextStyle {
        font_size: 16.0,
        line_height: 22.4,
        color: Color::WHITE,
        family: cosmic_text::Family::Name("DotGothic16"),
    };

    let layout = TextLayout::new(
        &mut font_manager,
        Arc::from(multiline_text),
        position,
        style,
    );

    // Should have multiple lines
    assert!(
        layout.lines().len() >= 3,
        "Should have at least 3 lines for explicit line breaks"
    );

    // Verify each line has content
    for line in layout.lines() {
        assert!(!line.glyphs.is_empty(), "Each line should have glyphs");
    }
}

#[test]
fn test_text_update() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    let initial_text = "初期テキスト".to_string();
    let position = Point::new(0.0, 0.0);
    let style = TextStyle::default();

    let mut layout = TextLayout::new(&mut font_manager, Arc::from(initial_text), position, style);

    assert_eq!(layout.text(), "初期テキスト");

    // Update text
    let new_text = "更新されたテキスト".to_string();
    layout.set_text(&mut font_manager, new_text.clone());

    assert_eq!(layout.text(), new_text);

    // Verify glyphs were regenerated
    let glyph_count = layout.glyphs().count();
    assert!(glyph_count > 0, "Updated text should have glyphs");
}

#[test]
fn test_empty_text_layout() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    let empty_text = String::new();
    let position = Point::new(0.0, 0.0);
    let style = TextStyle::default();

    let layout = TextLayout::new(&mut font_manager, Arc::from(empty_text), position, style);

    assert_eq!(layout.text(), "");

    // Empty text should have zero width
    // Height may still be non-zero due to default line height
    let size = layout.calculate().expect("Failed to calculate size");
    assert_eq!(size.width, 0.0, "Empty text should have zero width");
    // Note: cosmic-text may still report a line height even for empty text
}

#[test]
fn test_position_update() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    let text = "テスト".to_string();
    let initial_position = Point::new(10.0, 20.0);
    let style = TextStyle::default();

    let mut layout = TextLayout::new(&mut font_manager, Arc::from(text), initial_position, style);

    assert_eq!(layout.position(), initial_position);

    // Update position
    let new_position = Point::new(100.0, 200.0);
    layout.set_position(&mut font_manager, new_position);

    assert_eq!(layout.position(), new_position);

    // Verify glyphs are repositioned
    for glyph in layout.glyphs() {
        assert!(glyph.x >= new_position.x);
        assert!(glyph.y >= new_position.y);
    }
}

#[test]
fn test_large_text_performance() {
    let mut font_manager = FontManager::new().expect("Failed to create FontManager");
    let font_path = get_font_path();
    font_manager
        .load_japanese_font(&font_path)
        .expect("Failed to load Japanese font");

    // Create a larger text block
    let large_text = "日本語のテキストレンダリングのテストです。".repeat(10);
    let position = Point::new(0.0, 0.0);
    let style = TextStyle {
        font_size: 14.0,
        line_height: 20.0,
        color: Color::WHITE,
        family: cosmic_text::Family::Name("DotGothic16"),
    };

    // This should complete without panic or excessive time
    let layout = TextLayout::with_max_width(
        &mut font_manager,
        Arc::from(large_text.clone()),
        position,
        style,
        400.0,
    );

    // Verify it was processed
    assert_eq!(layout.text(), large_text);
    assert!(!layout.lines().is_empty());

    let size = layout.calculate().expect("Failed to calculate size");
    assert!(size.width > 0.0 && size.height > 0.0);
}
