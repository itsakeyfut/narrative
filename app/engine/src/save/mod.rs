//! Save/load module
//!
//! This module provides save and load functionality with thumbnails.

mod data;
mod manager;
mod slot_info;
mod thumbnail;

pub use data::{SAVE_VERSION, SaveData, SavedCharacterDisplay};
pub use manager::SaveManager;
pub use slot_info::{SlotInfo, list_all_slots};
pub use thumbnail::generate_thumbnail;
