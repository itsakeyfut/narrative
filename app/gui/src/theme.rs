//! Narrative Theme - Shared color palette and styling constants
//!
//! Modern dark theme with teal/cyan accents for visual novel editing.

use crate::framework::Color;

/// Dark theme color palette
pub mod colors {
    use super::Color;

    // Background colors - Dark gradient
    pub const BG_DARKEST: Color = Color::new(0.07, 0.07, 0.08, 1.0); // #121214
    pub const BG_DARK: Color = Color::new(0.10, 0.10, 0.12, 1.0); // #1a1a1e
    pub const BG_PANEL: Color = Color::new(0.12, 0.12, 0.14, 1.0); // #1f1f24
    pub const BG_ELEVATED: Color = Color::new(0.16, 0.16, 0.18, 1.0); // #29292e
    pub const BG_HOVER: Color = Color::new(0.20, 0.20, 0.22, 1.0); // #333338
    pub const BG_SELECTED: Color = Color::new(0.24, 0.24, 0.26, 1.0); // #3d3d42

    // Accent colors - Teal/Cyan gradient (Filmora style)
    pub const ACCENT_PRIMARY: Color = Color::new(0.00, 0.85, 0.75, 1.0); // #00D9C0 - Bright teal
    pub const ACCENT_SECONDARY: Color = Color::new(0.20, 0.75, 0.95, 1.0); // #33BFF2 - Cyan blue
    pub const ACCENT_GRADIENT_START: Color = Color::new(0.00, 0.90, 0.80, 1.0); // #00E6CC
    pub const ACCENT_GRADIENT_END: Color = Color::new(0.30, 0.70, 0.95, 1.0); // #4DB3F2
    pub const ACCENT_MUTED: Color = Color::new(0.00, 0.50, 0.45, 1.0); // #008073

    // Border colors
    pub const BORDER: Color = Color::new(0.18, 0.18, 0.20, 1.0); // #2e2e33
    pub const BORDER_LIGHT: Color = Color::new(0.25, 0.25, 0.28, 1.0); // #404047
    pub const BORDER_ACCENT: Color = Color::new(0.00, 0.60, 0.55, 1.0); // #00998C

    // Text colors
    pub const TEXT_PRIMARY: Color = Color::new(0.95, 0.95, 0.95, 1.0); // #F2F2F2
    pub const TEXT_SECONDARY: Color = Color::new(0.70, 0.70, 0.72, 1.0); // #B3B3B8
    pub const TEXT_MUTED: Color = Color::new(0.45, 0.45, 0.48, 1.0); // #73737A
    pub const TEXT_ACCENT: Color = Color::new(0.00, 0.85, 0.75, 1.0); // #00D9C0

    // Sidebar colors
    pub const SIDEBAR_BG: Color = Color::new(0.08, 0.08, 0.10, 1.0); // #14141a
    pub const SIDEBAR_ITEM_HOVER: Color = Color::new(0.12, 0.12, 0.14, 1.0); // #1f1f24
    pub const SIDEBAR_ITEM_ACTIVE: Color = Color::new(0.00, 0.30, 0.27, 1.0); // #004D45

    // Timeline colors
    pub const TIMELINE_BG: Color = Color::new(0.09, 0.09, 0.11, 1.0); // #17171c
    pub const TIMELINE_TRACK: Color = Color::new(0.14, 0.14, 0.16, 1.0); // #242429
    pub const TIMELINE_RULER: Color = Color::new(0.11, 0.11, 0.13, 1.0); // #1c1c21
    pub const TIMELINE_CLIP_VIDEO: Color = Color::new(0.25, 0.50, 0.75, 1.0); // #4080BF
    pub const TIMELINE_CLIP_AUDIO: Color = Color::new(0.30, 0.65, 0.40, 1.0); // #4DA666
    pub const TIMELINE_PLAYHEAD: Color = Color::new(0.95, 0.30, 0.30, 1.0); // #F24D4D

    // Button colors
    pub const BUTTON_PRIMARY: Color = Color::new(0.00, 0.75, 0.65, 1.0); // #00BFA6
    pub const BUTTON_PRIMARY_HOVER: Color = Color::new(0.00, 0.85, 0.75, 1.0); // #00D9C0
    pub const BUTTON_SECONDARY: Color = Color::new(0.18, 0.18, 0.20, 1.0); // #2e2e33
    pub const BUTTON_SECONDARY_HOVER: Color = Color::new(0.25, 0.25, 0.28, 1.0); // #404047

    // Card colors (for project cards, etc.)
    pub const CARD_BG: Color = Color::new(0.14, 0.14, 0.16, 1.0); // #242429
    pub const CARD_HOVER: Color = Color::new(0.18, 0.18, 0.20, 1.0); // #2e2e33
    pub const CARD_BORDER: Color = Color::new(0.20, 0.20, 0.22, 1.0); // #333338

    // Status colors
    pub const SUCCESS: Color = Color::new(0.30, 0.75, 0.45, 1.0); // #4DBF73
    pub const WARNING: Color = Color::new(0.95, 0.75, 0.25, 1.0); // #F2BF40
    pub const ERROR: Color = Color::new(0.90, 0.35, 0.35, 1.0); // #E65959
    pub const INFO: Color = Color::new(0.30, 0.70, 0.95, 1.0); // #4DB3F2
}

/// Spacing constants (in pixels)
pub mod spacing {
    pub const XS: f32 = 6.0;
    pub const SM: f32 = 10.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 20.0;
    pub const XL: f32 = 28.0;
    pub const XXL: f32 = 36.0;
    pub const XXXL: f32 = 56.0;
}

/// Border radius constants
pub mod radius {
    pub const XS: f32 = 2.0;
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 12.0;
    pub const XL: f32 = 16.0;
    pub const FULL: f32 = 9999.0; // For circular elements
}

/// Common constants shared across multiple components
pub mod common {
    /// Standard border/line thickness (1 pixel)
    pub const BORDER_THICKNESS: f32 = 1.0;

    /// Character width ratio for text width estimation (average width)
    /// Usage: `text.len() as f32 * font_size * CHAR_WIDTH_RATIO`
    pub const CHAR_WIDTH_RATIO: f32 = 0.6;

    /// Tighter character width ratio for condensed layouts
    /// Usage: `text.len() as f32 * font_size * CHAR_WIDTH_RATIO_TIGHT`
    pub const CHAR_WIDTH_RATIO_TIGHT: f32 = 0.9;

    /// Shadow layer count for drop shadow effects
    pub const SHADOW_LAYERS: usize = 3;

    /// Shadow offset step per layer
    pub const SHADOW_OFFSET_STEP: f32 = 2.0;

    /// Shadow base alpha value
    pub const SHADOW_BASE_ALPHA: f32 = 0.15;
}

/// Font size constants
pub mod font_size {
    pub const XS: f32 = 13.0;
    pub const SM: f32 = 14.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 18.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 28.0;
    pub const TITLE: f32 = 36.0;
    pub const HERO: f32 = 56.0;
}

/// Typography constants for text layout calculations
///
/// # Why baseline offset is needed
///
/// When vertically centering text, simply placing the text at the center Y coordinate
/// does not produce visually centered text. This is because fonts have different metrics:
///
/// ```text
/// ┌─────────────────────────┐
/// │      ascender line      │  ← Top of 'h', 'l', etc.
/// │  ─ ─ ─ x-height ─ ─ ─   │  ← Top of 'x', 'a', 'e', etc.
/// │  ═══ baseline ═══════   │  ← Where text "sits"
/// │      descender line     │  ← Bottom of 'g', 'y', 'p', etc.
/// └─────────────────────────┘
/// ```
///
/// The baseline (where most characters sit) is typically about 1/3 from the bottom
/// of the line height. To visually center text, we need to offset from the geometric
/// center by this amount.
///
/// # Ideal solution
///
/// Ideally, we would use actual font metrics from the text rendering system to calculate
/// the exact baseline position. However, our current framework does not expose font metrics,
/// so we use an empirical approximation (0.33 ≈ 1/3) that works well for most fonts.
///
/// # Future improvement
///
/// When font metrics become available, this should be replaced with:
/// ```ignore
/// let baseline_offset = font_metrics.ascender / (font_metrics.ascender + font_metrics.descender);
/// ```
pub mod typography {
    /// Text baseline offset ratio for vertical centering.
    ///
    /// This value (0.33 ≈ 1/3) approximates where the baseline sits relative to
    /// the font's line height. It is derived from typical font metrics where:
    /// - Ascender (above baseline): ~2/3 of line height
    /// - Descender (below baseline): ~1/3 of line height
    ///
    /// # Usage
    /// ```ignore
    /// let text_y = vertical_center + font_size * typography::BASELINE_OFFSET;
    /// ```
    pub const BASELINE_OFFSET: f32 = 0.33;
}

/// Layout constants
pub mod layout {
    pub const SIDEBAR_WIDTH: f32 = 240.0;
    pub const SIDEBAR_WIDTH_COLLAPSED: f32 = 72.0;
    pub const HEADER_HEIGHT: f32 = 56.0;
    pub const TOOLBAR_HEIGHT: f32 = 68.0;
    pub const TIMELINE_MIN_HEIGHT: f32 = 140.0;
    pub const TIMELINE_DEFAULT_HEIGHT: f32 = 220.0;
    pub const MEDIA_PANEL_WIDTH: f32 = 380.0;
    pub const INSPECTOR_WIDTH: f32 = 340.0;
    pub const TRACK_LABEL_WIDTH: f32 = 120.0;

    // Start screen card sizes
    pub const NEW_PROJECT_CARD_WIDTH: f32 = 560.0;
    pub const NEW_PROJECT_CARD_HEIGHT: f32 = 160.0;
    pub const QUICK_ACTION_CARD_WIDTH: f32 = 160.0;
    pub const QUICK_ACTION_CARD_HEIGHT: f32 = 120.0;
    pub const PROJECT_CARD_WIDTH: f32 = 200.0;
    pub const PROJECT_CARD_HEIGHT: f32 = 160.0;

    // Preview panel
    pub const PREVIEW_CONTROLS_HEIGHT: f32 = 72.0;
    pub const PREVIEW_TIMECODE_HEIGHT: f32 = 44.0;

    // Navigation item
    pub const NAV_ITEM_HEIGHT: f32 = 52.0;
    pub const NAV_ITEM_ACTIVE_HEIGHT: f32 = 48.0;

    // Logo and branding
    pub const LOGO_SIZE: f32 = 40.0;
    pub const LOGO_MARGIN_RIGHT: f32 = 12.0;
    pub const APP_NAME_WIDTH: f32 = 100.0;

    // Window controls
    pub const WINDOW_CONTROL_WIDTH: f32 = 52.0;
    pub const WINDOW_CONTROL_ICON_SIZE: f32 = 12.0;

    // Menu bar
    pub const MENU_CHAR_WIDTH: f32 = 11.0;
    pub const MENU_ITEM_PADDING: f32 = 16.0;

    // Thumbnail sizes
    pub const THUMBNAIL_WIDTH: f32 = 80.0;
    pub const THUMBNAIL_HEIGHT: f32 = 60.0;

    // Project card thumbnail area
    pub const PROJECT_CARD_THUMBNAIL_HEIGHT: f32 = 100.0;
}

/// Icon size constants (in pixels)
///
/// Use these for consistent icon sizing across the UI:
/// - XS (16px): Small inline icons, folder icons
/// - SM (20px): Navigation icons, small buttons
/// - MD (24px): Default icon size for most UI elements
/// - LG (32px): Larger emphasis icons, cards
/// - XL (40px): Extra large icons, feature highlights
/// - XXL (48px): Hero icons, splash screens
pub mod icon_size {
    pub const XS: f32 = 16.0;
    pub const SM: f32 = 20.0;
    pub const MD: f32 = 24.0;
    pub const LG: f32 = 32.0;
    pub const XL: f32 = 40.0;
    pub const XXL: f32 = 48.0;
}

/// Toolbar constants for the media type tabs (Media, Audio, Titles, etc.)
pub mod toolbar {
    /// Width of each toolbar item button
    pub const ITEM_WIDTH: f32 = 110.0;
    /// Height of each toolbar item button
    pub const ITEM_HEIGHT: f32 = 60.0;
    /// Size of icons within toolbar items
    pub const ITEM_ICON_SIZE: f32 = 28.0;
    /// Inset from item edges for the selected accent line
    pub const ACCENT_INSET: f32 = 10.0;
    /// Height of the selected item accent line
    pub const ACCENT_HEIGHT: f32 = 2.0;
}

/// Inspector panel constants
pub mod inspector {
    /// Height of collapsible section headers (Transform, Effects, Audio, etc.)
    pub const SECTION_HEADER_HEIGHT: f32 = 44.0;
    /// Height of each property row (label + value input)
    pub const PROPERTY_ROW_HEIGHT: f32 = 40.0;
    /// Height of the inspector header area
    pub const HEADER_HEIGHT: f32 = 60.0;
    /// Width of each tab button (Properties, Keyframes)
    pub const TAB_WIDTH: f32 = 100.0;
    /// Width of the active tab underline indicator
    pub const TAB_UNDERLINE_WIDTH: f32 = 85.0;
    /// Ratio of property row width allocated to the label (left side)
    pub const LABEL_WIDTH_RATIO: f32 = 0.4;
}

/// Media browser panel constants for the left sidebar media library
pub mod media_browser {
    /// Height of the header area (title + search + controls)
    pub const HEADER_HEIGHT: f32 = 140.0;
    /// Ratio of panel height allocated to the folder list (top section)
    pub const FOLDER_LIST_RATIO: f32 = 0.38;
    /// Height of each folder item in the list
    pub const FOLDER_HEIGHT: f32 = 36.0;
    /// Size of media item thumbnails in the grid
    pub const ITEM_SIZE: f32 = 80.0;
    /// Height of the item name label below thumbnails
    pub const ITEM_NAME_HEIGHT: f32 = 20.0;
    /// Height of the search input bar
    pub const SEARCH_BAR_HEIGHT: f32 = 36.0;
    /// Height of action buttons (Import, Filter)
    pub const BUTTON_HEIGHT: f32 = 32.0;
    /// Size of folder icons in the folder list
    pub const FOLDER_ICON_SIZE: f32 = 16.0;
    /// Height of type/duration badge overlays on thumbnails
    pub const BADGE_HEIGHT: f32 = 18.0;
}

/// Timeline panel constants for the video/audio track editor
pub mod timeline {
    /// Height of the time ruler at the top
    pub const RULER_HEIGHT: f32 = 28.0;
    /// Height of video tracks
    pub const VIDEO_TRACK_HEIGHT: f32 = 60.0;
    /// Height of audio tracks
    pub const AUDIO_TRACK_HEIGHT: f32 = 44.0;
    /// Height of major time markers (seconds, minutes)
    pub const MARKER_MAJOR_HEIGHT: f32 = 14.0;
    /// Height of minor time markers (frames, subdivisions)
    pub const MARKER_MINOR_HEIGHT: f32 = 8.0;
    /// Width of the playhead triangle indicator
    pub const PLAYHEAD_HEAD_WIDTH: f32 = 14.0;
    /// Height of the playhead triangle indicator
    pub const PLAYHEAD_HEAD_HEIGHT: f32 = 10.0;
    /// Border radius for clip rectangles
    pub const CLIP_BORDER_RADIUS: f32 = 5.0;
}

/// Preview panel constants for video playback controls
pub mod preview {
    /// Height of the playback controls area (from bottom of panel)
    pub const CONTROLS_AREA_HEIGHT: f32 = 72.0;
    /// Width of the control buttons container background
    pub const CONTROLS_BG_WIDTH: f32 = 280.0;
    /// Height of the control buttons container background
    pub const CONTROLS_BG_HEIGHT: f32 = 52.0;
    /// Vertical offset for control buttons container
    pub const CONTROLS_BG_OFFSET_Y: f32 = 6.0;
    /// Half-width offset for centering controls container
    pub const CONTROLS_BG_OFFSET_X: f32 = 140.0;

    /// Size of small control buttons (prev/next frame, loop)
    pub const BUTTON_SMALL_SIZE: f32 = 32.0;
    /// Size of play/pause button
    pub const BUTTON_PLAY_SIZE: f32 = 42.0;
    /// Vertical offset for small buttons
    pub const BUTTON_SMALL_OFFSET_Y: f32 = 16.0;
    /// Vertical offset for play button
    pub const BUTTON_PLAY_OFFSET_Y: f32 = 12.0;

    /// X offset for previous frame button (from center)
    pub const PREV_BUTTON_OFFSET_X: f32 = 115.0;
    /// X offset for play button (from center)
    pub const PLAY_BUTTON_OFFSET_X: f32 = 21.0;
    /// X offset for next frame button (from center)
    pub const NEXT_BUTTON_OFFSET_X: f32 = 83.0;
    /// X offset for loop button (from center)
    pub const LOOP_BUTTON_OFFSET_X: f32 = 123.0;

    /// Left/right padding for scrub bar
    pub const SCRUB_PADDING_X: f32 = 100.0;
    /// Vertical offset for scrub bar from timecode
    pub const SCRUB_OFFSET_Y: f32 = 10.0;
    /// Height of scrub bar track
    pub const SCRUB_TRACK_HEIGHT: f32 = 6.0;
    /// Border radius for scrub bar track
    pub const SCRUB_TRACK_RADIUS: f32 = 3.0;
    /// Size of playhead circle
    pub const PLAYHEAD_SIZE: f32 = 14.0;
    /// Playhead Y offset from track
    pub const PLAYHEAD_OFFSET_Y: f32 = 4.0;

    /// Timecode text vertical offset
    pub const TIMECODE_TEXT_OFFSET_Y: f32 = 22.0;

    /// Placeholder text vertical offset
    pub const PLACEHOLDER_OFFSET_Y: f32 = 10.0;
    /// Subtext vertical offset below center
    pub const SUBTEXT_OFFSET_Y: f32 = 24.0;

    /// Speed indicator X offset from right edge
    pub const SPEED_OFFSET_X: f32 = 44.0;
    /// Speed indicator Y offset from controls area
    pub const SPEED_OFFSET_Y: f32 = 36.0;

    /// Icon element sizes for control buttons
    pub const ICON_BAR_WIDTH_NARROW: f32 = 5.0;
    pub const ICON_BAR_WIDTH_WIDE: f32 = 9.0;
    pub const ICON_BAR_HEIGHT: f32 = 14.0;
    pub const ICON_OFFSET_SMALL: f32 = 9.0;
    pub const ICON_OFFSET_MEDIUM: f32 = 7.0;

    /// Pause icon dimensions
    pub const PAUSE_BAR_WIDTH: f32 = 6.0;
    pub const PAUSE_BAR_HEIGHT: f32 = 18.0;
    pub const PAUSE_BAR_OFFSET_X1: f32 = 13.0;
    pub const PAUSE_BAR_OFFSET_X2: f32 = 23.0;
    pub const PAUSE_BAR_OFFSET_Y: f32 = 12.0;

    /// Play icon dimensions
    pub const PLAY_ICON_WIDTH: f32 = 12.0;
    pub const PLAY_ICON_HEIGHT: f32 = 18.0;
    pub const PLAY_ICON_OFFSET_X: f32 = 16.0;
    pub const PLAY_ICON_OFFSET_Y: f32 = 12.0;

    /// Loop button text offset
    pub const LOOP_TEXT_OFFSET_X: f32 = 10.0;
    pub const LOOP_TEXT_OFFSET_Y: f32 = 21.0;
}

/// Sidebar navigation constants
pub mod sidebar {
    /// Height of sidebar header area (logo/brand)
    pub const HEADER_HEIGHT: f32 = 80.0;
    /// Total height allocated per navigation item (including spacing)
    pub const ITEM_SPACING: f32 = 44.0;
    /// Actual clickable height of navigation item
    pub const ITEM_HEIGHT: f32 = 40.0;
    /// Size of navigation item icons
    pub const ICON_SIZE: f32 = 20.0;
    /// Icon vertical offset from item top
    pub const ICON_OFFSET_Y: f32 = 10.0;
    /// Label X offset (after icon area)
    pub const LABEL_OFFSET_X: f32 = 40.0;
    /// Label Y offset from item top
    pub const LABEL_OFFSET_Y: f32 = 26.0;

    /// Selected item accent bar width
    pub const ACCENT_WIDTH: f32 = 3.0;
    /// Selected item accent bar height
    pub const ACCENT_HEIGHT: f32 = 24.0;
    /// Selected item accent bar Y offset from item top
    pub const ACCENT_OFFSET_Y: f32 = 8.0;

    /// Badge X offset from right edge
    pub const BADGE_OFFSET_X: f32 = 30.0;
    /// Badge width
    pub const BADGE_WIDTH: f32 = 24.0;
    /// Badge height
    pub const BADGE_HEIGHT: f32 = 20.0;
    /// Badge border radius
    pub const BADGE_RADIUS: f32 = 10.0;
    /// Badge text X offset within badge
    pub const BADGE_TEXT_OFFSET_X: f32 = 7.0;
    /// Badge text Y offset from item top
    pub const BADGE_TEXT_OFFSET_Y: f32 = 25.0;

    /// Brand text Y offset from header top
    pub const BRAND_OFFSET_Y: f32 = 45.0;
}

/// Button component constants
pub mod button {
    /// Text vertical offset ratio for centering
    /// Used as: `(bounds.height() + font_size * TEXT_VERTICAL_RATIO) / 2.0`
    pub const TEXT_VERTICAL_RATIO: f32 = 0.8;
}

/// Dropdown menu constants
pub mod dropdown {
    /// Height of each dropdown menu item
    pub const ITEM_HEIGHT: f32 = 40.0;
    /// Minimum width of the dropdown menu
    pub const MIN_WIDTH: f32 = 240.0;
    /// Height of separator lines between menu items
    pub const SEPARATOR_HEIGHT: f32 = 14.0;
    /// Estimated character width for keyboard shortcut text
    pub const SHORTCUT_CHAR_WIDTH: f32 = 9.0;
    /// Right padding reserved for keyboard shortcut display
    pub const SHORTCUT_PADDING: f32 = 56.0;
    /// Horizontal padding (left and right) for menu items
    pub const ITEM_PADDING_X: f32 = 4.0;
}

/// Audio level meter constants
pub mod audio_meter {
    use super::Color;

    // ==========================================================================
    // Layout dimensions
    // ==========================================================================

    /// Default meter bar width (for vertical mode)
    pub const METER_WIDTH: f32 = 8.0;
    /// Gap between stereo meter bars
    pub const STEREO_GAP: f32 = 2.0;
    /// Number of gradient segments for meter display
    pub const GRADIENT_SEGMENTS: i32 = 10;
    /// Gap between meter segments
    pub const SEGMENT_GAP: f32 = 1.0;
    /// Peak hold indicator thickness
    pub const PEAK_INDICATOR_THICKNESS: f32 = 2.0;
    /// Clip indicator height/width
    pub const CLIP_INDICATOR_SIZE: f32 = 4.0;
    /// Peak hold decay rate per update
    pub const PEAK_DECAY_RATE: f32 = 0.02;

    // ==========================================================================
    // Colors for meter gradient (from low to high level)
    // ==========================================================================

    /// Low level color (green) - below 60% threshold
    pub const COLOR_LOW: Color = Color::new(0.392, 0.784, 0.235, 1.0);

    /// Medium level color (yellow) - 60-80% threshold
    pub const COLOR_MEDIUM_START: Color = Color::new(0.392, 0.784, 0.235, 1.0);

    /// High level color (orange/yellow) - 80% threshold
    pub const COLOR_HIGH_START: Color = Color::new(1.0, 0.784, 0.235, 1.0);

    /// Warning level color (red) - above 80% threshold
    pub const COLOR_WARNING: Color = Color::new(1.0, 0.235, 0.235, 1.0);

    /// Clip indicator color (bright red)
    pub const COLOR_CLIP: Color = Color::new(1.0, 0.0, 0.0, 1.0);

    /// Clip indicator color (dimmed red for flash effect)
    pub const COLOR_CLIP_DIM: Color = Color::new(0.706, 0.0, 0.0, 1.0);

    // ==========================================================================
    // Level thresholds (as normalized values 0.0-1.0)
    // ==========================================================================

    /// Threshold where color starts transitioning to yellow (60%)
    pub const THRESHOLD_MEDIUM: f32 = 0.6;

    /// Threshold where color transitions to orange/red (80%)
    pub const THRESHOLD_HIGH: f32 = 0.8;

    /// Threshold for clipping (100%)
    pub const THRESHOLD_CLIP: f32 = 1.0;
}
