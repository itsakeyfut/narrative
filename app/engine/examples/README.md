# Examples - narrative-engine

This directory contains sample programs demonstrating narrative-engine features.

## Available Samples

### wgpu_clear_color

Demonstrates basic wgpu setup and displays a window with a clear color (dark blue).

```bash
cargo run -p narrative-engine --example wgpu_clear_color
```

### text_rendering

**Verification for Issue #11: TextLayout Implementation**

Sample to verify Japanese text rendering implementation. Demonstrates the following features:

- Text layout calculation
- Text wrapping
- Line spacing and character spacing adjustment
- Japanese text rendering
- cosmic-text integration
- GlyphCache and TextureAtlas usage
- Integration with wgpu-based renderer

```bash
cargo run -p narrative-engine --example text_rendering
```

#### Expected Output

A window opens displaying the following text:

- Japanese text: "こんにちは、世界！" (Hello, World!)
- Japanese-English mixed text: "Hello 世界 123 テスト" (Hello World 123 Test)
- Long text: "ビジュアルノベルエンジン - Visual Novel Engine"
- Multi-line text (different color per line)
- Completion message: "Issue #11: テキストレイアウト実装完了" (Issue #11: Text layout implementation complete)

#### Technical Details

This sample uses the following components:

- `Renderer` - wgpu-based renderer
- `FontManager` - Font management via cosmic-text
- `GlyphCache` - Glyph caching and rasterization
- `TextureAtlas` - GPU management of glyph textures
- `TextLayout` - Text layout calculation
- `RenderCommand::DrawText` - Text drawing command

#### Dependencies

- DotGothic16 font (`assets/fonts/DotGothic16/DotGothic16-Regular.ttf`)

## Adding Samples

To add a new sample:

1. Create a file in `app/engine/examples/your_example.rs`
2. Implement the `main()` function
3. Run with `cargo run --example your_example`

Samples are self-contained, and required dependencies are already included in `app/engine/Cargo.toml`.
