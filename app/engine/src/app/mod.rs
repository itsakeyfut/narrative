//! Application integration module
//!
//! This module provides the game loop and engine configuration.

mod config;
mod game_loop;

pub use config::{AudioConfig, EngineConfig};
pub use game_loop::GameLoop;
