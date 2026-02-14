//! UI Components for Narrative Novel Engine
//!
//! This module contains all UI components built on top of the narrative-gui framework.
//! All components implement the Element trait from narrative-gui.

// Core game UI components
pub mod backlog;
pub mod cg_gallery;
pub mod cg_viewer;
pub mod character_animation;
pub mod character_sprite;
pub mod character_transition;
pub mod choice_menu;
pub mod confirm_dialog;
pub mod dialogue_box;
pub mod game_root;
pub mod pause_menu;
pub mod quick_menu;
pub mod save_load_menu;
pub mod save_slot_card;
pub mod settings_menu;
pub mod title_screen;

// Re-exports
pub use backlog::BacklogElement;
pub use cg_gallery::{CgGalleryAction, CgGalleryElement};
pub use cg_viewer::{CgViewerAction, CgViewerElement};
pub use character_sprite::CharacterSpriteElement;
pub use choice_menu::ChoiceMenuElement;
pub use confirm_dialog::{ConfirmDialogElement, DialogResponse};
pub use dialogue_box::DialogueBoxElement;
pub use game_root::GameRootElement;
pub use pause_menu::{PauseMenuAction, PauseMenuElement};
pub use quick_menu::{QuickMenuAction, QuickMenuElement};
pub use save_load_menu::{SaveLoadMenuAction, SaveLoadMenuElement};
pub use save_slot_card::SaveSlotCard;
pub use settings_menu::SettingsMenuElement;
pub use title_screen::{TitleScreenAction, TitleScreenElement};
