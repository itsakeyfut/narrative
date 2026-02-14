//! Native menu bar management using muda
//!
//! Provides cross-platform native menu bar functionality for Narrative.
//! - Windows: Native Win32 menu bar
//! - macOS: Native app menu bar
//! - Linux: GTK menu bar (requires gtk feature)

use muda::{
    AboutMetadata, Menu, MenuEvent, MenuEventReceiver, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};
use std::sync::mpsc;

/// Menu item identifiers for handling menu events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuId {
    // File menu
    NewProject,
    OpenProject,
    Save,
    SaveAs,
    Export,
    Settings,
    Exit,

    // Edit menu
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,

    // View menu
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleInspector,
    ToggleMediaBrowser,
    ToggleFullscreen,

    // Timeline menu
    AddTrack,
    DeleteTrack,
    SplitClip,
    RippleDelete,
    SnapToGrid,
    ShowWaveforms,

    // Help menu
    Documentation,
    Shortcuts,
    CheckUpdates,
    About,
}

impl MenuId {
    /// Convert a string ID to MenuId
    pub fn from_id_str(s: &str) -> Option<Self> {
        match s {
            "new_project" => Some(Self::NewProject),
            "open_project" => Some(Self::OpenProject),
            "save" => Some(Self::Save),
            "save_as" => Some(Self::SaveAs),
            "export" => Some(Self::Export),
            "settings" => Some(Self::Settings),
            "exit" => Some(Self::Exit),

            "undo" => Some(Self::Undo),
            "redo" => Some(Self::Redo),
            "cut" => Some(Self::Cut),
            "copy" => Some(Self::Copy),
            "paste" => Some(Self::Paste),
            "delete" => Some(Self::Delete),
            "select_all" => Some(Self::SelectAll),

            "zoom_in" => Some(Self::ZoomIn),
            "zoom_out" => Some(Self::ZoomOut),
            "zoom_reset" => Some(Self::ZoomReset),
            "toggle_inspector" => Some(Self::ToggleInspector),
            "toggle_media_browser" => Some(Self::ToggleMediaBrowser),
            "toggle_fullscreen" => Some(Self::ToggleFullscreen),

            "add_track" => Some(Self::AddTrack),
            "delete_track" => Some(Self::DeleteTrack),
            "split_clip" => Some(Self::SplitClip),
            "ripple_delete" => Some(Self::RippleDelete),
            "snap_to_grid" => Some(Self::SnapToGrid),
            "show_waveforms" => Some(Self::ShowWaveforms),

            "documentation" => Some(Self::Documentation),
            "shortcuts" => Some(Self::Shortcuts),
            "check_updates" => Some(Self::CheckUpdates),
            "about" => Some(Self::About),
            _ => None,
        }
    }
}

/// Application menu bar
pub struct AppMenu {
    pub menu: Menu,
    // Store submenus for potential later modification
    pub file_menu: Submenu,
    pub edit_menu: Submenu,
    pub view_menu: Submenu,
    pub timeline_menu: Submenu,
    pub help_menu: Submenu,
}

impl AppMenu {
    /// Create a new application menu bar
    pub fn new() -> Self {
        let menu = Menu::new();

        // File menu
        let file_menu = Submenu::new("File", true);
        file_menu
            .append_items(&[
                &MenuItem::with_id(
                    "new_project",
                    "New Project",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyN)),
                ),
                &MenuItem::with_id(
                    "open_project",
                    "Open Project...",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyO)),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    "save",
                    "Save",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyS)),
                ),
                &MenuItem::with_id(
                    "save_as",
                    "Save As...",
                    true,
                    Some(Accelerator::new(
                        Some(Modifiers::CONTROL | Modifiers::SHIFT),
                        Code::KeyS,
                    )),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    "export",
                    "Export...",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyE)),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    "settings",
                    "Settings",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::Comma)),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    "exit",
                    "Exit",
                    true,
                    Some(Accelerator::new(Some(Modifiers::ALT), Code::F4)),
                ),
            ])
            .map_err(|e| tracing::error!("Failed to create file menu items: {}", e))
            .ok();

        // Edit menu
        let edit_menu = Submenu::new("Edit", true);
        edit_menu
            .append_items(&[
                &MenuItem::with_id(
                    "undo",
                    "Undo",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyZ)),
                ),
                &MenuItem::with_id(
                    "redo",
                    "Redo",
                    true,
                    Some(Accelerator::new(
                        Some(Modifiers::CONTROL | Modifiers::SHIFT),
                        Code::KeyZ,
                    )),
                ),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::cut(Some("Cut")),
                &PredefinedMenuItem::copy(Some("Copy")),
                &PredefinedMenuItem::paste(Some("Paste")),
                &MenuItem::with_id(
                    "delete",
                    "Delete",
                    true,
                    Some(Accelerator::new(None, Code::Delete)),
                ),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::select_all(Some("Select All")),
            ])
            .map_err(|e| tracing::error!("Failed to create edit menu items: {}", e))
            .ok();

        // View menu
        let view_menu = Submenu::new("View", true);
        view_menu
            .append_items(&[
                &MenuItem::with_id(
                    "zoom_in",
                    "Zoom In",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::Equal)),
                ),
                &MenuItem::with_id(
                    "zoom_out",
                    "Zoom Out",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::Minus)),
                ),
                &MenuItem::with_id(
                    "zoom_reset",
                    "Reset Zoom",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::Digit0)),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    "toggle_inspector",
                    "Toggle Inspector",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyI)),
                ),
                &MenuItem::with_id(
                    "toggle_media_browser",
                    "Toggle Media Browser",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyM)),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    "toggle_fullscreen",
                    "Toggle Fullscreen",
                    true,
                    Some(Accelerator::new(None, Code::F11)),
                ),
            ])
            .map_err(|e| tracing::error!("Failed to create view menu items: {}", e))
            .ok();

        // Timeline menu
        let timeline_menu = Submenu::new("Timeline", true);
        timeline_menu
            .append_items(&[
                &MenuItem::with_id(
                    "add_track",
                    "Add Track",
                    true,
                    Some(Accelerator::new(
                        Some(Modifiers::CONTROL | Modifiers::SHIFT),
                        Code::KeyT,
                    )),
                ),
                &MenuItem::with_id("delete_track", "Delete Track", true, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    "split_clip",
                    "Split Clip",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyB)),
                ),
                &MenuItem::with_id(
                    "ripple_delete",
                    "Ripple Delete",
                    true,
                    Some(Accelerator::new(Some(Modifiers::SHIFT), Code::Delete)),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    "snap_to_grid",
                    "Snap to Grid",
                    true,
                    Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyG)),
                ),
                &MenuItem::with_id("show_waveforms", "Show Waveforms", true, None),
            ])
            .map_err(|e| tracing::error!("Failed to create timeline menu items: {}", e))
            .ok();

        // Help menu
        let help_menu = Submenu::new("Help", true);
        help_menu
            .append_items(&[
                &MenuItem::with_id(
                    "documentation",
                    "Documentation",
                    true,
                    Some(Accelerator::new(None, Code::F1)),
                ),
                &MenuItem::with_id(
                    "shortcuts",
                    "Keyboard Shortcuts",
                    true,
                    Some(Accelerator::new(
                        Some(Modifiers::CONTROL | Modifiers::SHIFT),
                        Code::Slash,
                    )),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id("check_updates", "Check for Updates...", true, None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::about(
                    Some("About Narrative"),
                    Some(AboutMetadata {
                        name: Some("Narrative".to_string()),
                        version: Some(env!("CARGO_PKG_VERSION").to_string()),
                        copyright: Some("Copyright (c) 2024 Narrative Contributors".to_string()),
                        comments: Some(
                            "A wgpu-based visual novel engine"
                                .to_string(),
                        ),
                        ..Default::default()
                    }),
                ),
            ])
            .map_err(|e| tracing::error!("Failed to create help menu items: {}", e))
            .ok();

        // Add all submenus to main menu
        menu.append_items(&[
            &file_menu,
            &edit_menu,
            &view_menu,
            &timeline_menu,
            &help_menu,
        ])
        .map_err(|e| tracing::error!("Failed to create menu bar: {}", e))
        .ok();

        Self {
            menu,
            file_menu,
            edit_menu,
            view_menu,
            timeline_menu,
            help_menu,
        }
    }

    /// Initialize the menu for a window (Windows-specific)
    #[cfg(target_os = "windows")]
    pub fn init_for_window(&self, window: &winit::window::Window) {
        use raw_window_handle::HasWindowHandle;
        if let Ok(handle) = window.window_handle()
            && let raw_window_handle::RawWindowHandle::Win32(win32_handle) = handle.as_raw()
        {
            let hwnd = win32_handle.hwnd.get() as *mut std::ffi::c_void;
            // Safety: hwnd is obtained from a valid Window handle via raw-window-handle crate.
            // The pointer is guaranteed to be valid for the lifetime of the window.
            // This is safe because:
            // 1. window_handle() returns Ok only for valid windows
            // 2. Win32Handle contains a valid HWND
            // 3. muda's init_for_hwnd performs validation
            unsafe {
                if let Err(e) = self.menu.init_for_hwnd(hwnd as isize) {
                    tracing::error!("Failed to initialize menu for HWND: {}", e);
                }
            }
        }
    }

    /// Initialize the menu for a window (macOS-specific)
    #[cfg(target_os = "macos")]
    pub fn init_for_window(&self, _window: &winit::window::Window) {
        // On macOS, menus are app-global, not per-window
        if let Err(e) = self.menu.init_for_nsapp() {
            tracing::error!("Failed to initialize menu for NSApp: {}", e);
        }
    }

    /// Initialize the menu for a window (Linux-specific)
    #[cfg(target_os = "linux")]
    pub fn init_for_window(&self, window: &winit::window::Window) {
        use raw_window_handle::HasWindowHandle;
        // GTK menus need special handling
        // For now, this is a placeholder
        tracing::warn!("Native menu bar on Linux requires GTK integration");
    }

    /// Fallback for other platforms
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub fn init_for_window(&self, _window: &winit::window::Window) {
        tracing::warn!("Native menu bar not supported on this platform");
    }
}

impl Default for AppMenu {
    fn default() -> Self {
        Self::new()
    }
}

/// Menu event handler that processes menu item clicks
pub struct MenuEventHandler {
    receiver: MenuEventReceiver,
}

impl MenuEventHandler {
    /// Create a new menu event handler
    pub fn new() -> Self {
        Self {
            receiver: MenuEvent::receiver().clone(),
        }
    }

    /// Try to receive a menu event (non-blocking)
    pub fn try_recv(&self) -> Option<MenuId> {
        if let Ok(event) = self.receiver.try_recv() {
            let id_str = event.id().0.as_str();
            tracing::debug!("Menu event received: {}", id_str);
            MenuId::from_id_str(id_str)
        } else {
            None
        }
    }
}

impl Default for MenuEventHandler {
    fn default() -> Self {
        Self::new()
    }
}
