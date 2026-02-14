//! Game root element - main game UI container

mod audio;
mod children;
mod element;
mod in_game;
mod input;
mod rendering;
mod state;
mod textures;
mod transitions;

#[cfg(test)]
mod element_tests;
#[cfg(test)]
mod input_tests;

pub use element::GameRootElement;
