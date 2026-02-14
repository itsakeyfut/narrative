//! 2D rendering module
//!
//! This module provides GPU-accelerated 2D rendering using wgpu,
//! including sprite rendering, batching, and render commands.

mod batch;
mod commands;
mod pipeline;
mod renderer;
mod sprite;
mod transition;

pub use batch::RenderBatch;
pub use commands::{RenderCommand, RenderLayer, TransitionKind};
pub use renderer::{LoadedTexture, Renderer, TextureId};
pub use sprite::{SpritePipeline, SpriteVertex};
pub use transition::{TransitionPipeline, TransitionVertex};
