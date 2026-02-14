//! Async layout computation
//!
//! Issue #250 Phase 2: Background layout computation to avoid blocking the main thread.
//!
//! This module provides a mechanism to compute layouts asynchronously,
//! allowing the render loop to continue while layout is being calculated.
//!
//! ## Current Implementation (Synchronous)
//!
//! The current implementation uses **synchronous** layout computation via
//! `compute_layout_sync()`. This is structured to be easily converted to
//! true async/threaded computation in the future when needed.
//!
//! **Why synchronous for now:**
//! - Layout computation is typically fast enough (<1ms for most UIs)
//! - Avoids complexity of cross-thread taffy tree synchronization
//! - The API is already designed for async, making migration straightforward
//!
//! ## Future Async Implementation
//!
//! To enable true async layout computation:
//! 1. Use `tokio::spawn_blocking` or a dedicated layout thread
//! 2. Clone the taffy tree for background computation
//! 3. Use channels to communicate results back to main thread
//! 4. Implement `compute_layout_async()` method
//!
//! ## Architecture
//!
//! - `AsyncLayoutManager` coordinates between main thread and layout worker
//! - Layout requests are queued and processed in order
//! - Results are cached and the last valid layout is used while computing
//! - Generation counter prevents stale results from being applied

use super::error::FrameworkResult;
use super::layout::{LayoutEngine, Size};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// Status of an async layout computation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutStatus {
    /// No layout has been requested
    Idle,
    /// Layout is being computed
    Computing,
    /// Layout computation completed successfully
    Ready,
    /// Layout computation failed
    Failed,
}

/// Information about the last computed layout
#[derive(Debug, Clone)]
pub struct LayoutInfo {
    /// The size used for computation
    pub available_size: Size,
    /// When the layout was computed
    pub computed_at: Instant,
    /// Whether this result is stale (new computation pending)
    pub stale: bool,
    /// Generation counter
    pub generation: u64,
}

/// Manager for async layout computation
///
/// Provides a non-blocking interface for layout computation.
/// When a new layout is requested, it can be computed in the background
/// while the previous layout result is still used for rendering.
pub struct AsyncLayoutManager {
    /// Current layout status
    status: LayoutStatus,
    /// Latest valid layout info
    last_info: Option<LayoutInfo>,
    /// Flag indicating if computation is in progress
    computing: AtomicBool,
    /// Pending layout request
    pending_request: Option<LayoutRequest>,
    /// Current generation counter
    generation: u64,
}

/// Request for layout computation
#[derive(Debug, Clone)]
struct LayoutRequest {
    /// Available size for layout
    available_size: Size,
    /// Generation counter to detect stale requests
    generation: u64,
}

impl AsyncLayoutManager {
    /// Create a new async layout manager
    pub fn new() -> Self {
        Self {
            status: LayoutStatus::Idle,
            last_info: None,
            computing: AtomicBool::new(false),
            pending_request: None,
            generation: 0,
        }
    }

    /// Get current status
    pub fn status(&self) -> LayoutStatus {
        self.status
    }

    /// Check if layout computation is in progress
    pub fn is_computing(&self) -> bool {
        self.computing.load(Ordering::SeqCst)
    }

    /// Check if we have a valid layout result
    pub fn has_result(&self) -> bool {
        self.last_info.is_some()
    }

    /// Get the last valid layout info
    ///
    /// Returns info about the most recent successfully computed layout,
    /// even if a new computation is in progress.
    pub fn get_info(&self) -> Option<&LayoutInfo> {
        self.last_info.as_ref()
    }

    /// Check if the current layout is stale (new computation pending)
    pub fn is_stale(&self) -> bool {
        self.last_info.as_ref().is_some_and(|info| info.stale)
    }

    /// Get the current generation counter
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Request a new layout computation
    ///
    /// If a computation is already in progress, this marks the current
    /// result as stale and queues the request for when the current
    /// computation completes.
    pub fn request_layout(&mut self, available_size: Size) {
        self.generation += 1;

        self.pending_request = Some(LayoutRequest {
            available_size,
            generation: self.generation,
        });

        // Mark current result as stale
        if let Some(ref mut info) = self.last_info {
            info.stale = true;
        }
    }

    /// Check if there's a pending layout request
    pub fn has_pending_request(&self) -> bool {
        self.pending_request.is_some()
    }

    /// Get the pending request's available size
    pub fn pending_size(&self) -> Option<Size> {
        self.pending_request.as_ref().map(|r| r.available_size)
    }

    /// Start layout computation synchronously
    ///
    /// # Current Implementation
    ///
    /// This method runs **synchronously** on the calling thread. Despite the
    /// module name suggesting async operation, this is intentional:
    ///
    /// - Layout computation is typically <1ms for most UIs
    /// - The API and state management are designed for async conversion
    /// - When profiling shows layout as a bottleneck, migrate to true async
    ///
    /// # Future Async Migration
    ///
    /// To convert to async:
    /// ```ignore
    /// pub async fn compute_layout_async(...) -> FrameworkResult<bool> {
    ///     tokio::task::spawn_blocking(move || {
    ///         // Clone layout tree for thread safety
    ///         // Compute layout
    ///         // Send result via channel
    ///     }).await
    /// }
    /// ```
    ///
    /// Returns `true` if layout was computed, `false` if no pending request.
    pub fn compute_layout_sync(
        &mut self,
        engine: &mut LayoutEngine,
        root_node: taffy::NodeId,
    ) -> FrameworkResult<bool> {
        let Some(request) = self.pending_request.take() else {
            return Ok(false);
        };

        self.status = LayoutStatus::Computing;
        self.computing.store(true, Ordering::SeqCst);

        // Compute layout
        let result = engine.compute_layout(root_node, request.available_size);

        self.computing.store(false, Ordering::SeqCst);

        match result {
            Ok(()) => {
                self.status = LayoutStatus::Ready;
                self.last_info = Some(LayoutInfo {
                    available_size: request.available_size,
                    computed_at: Instant::now(),
                    stale: false,
                    generation: request.generation,
                });
                Ok(true)
            }
            Err(e) => {
                self.status = LayoutStatus::Failed;
                Err(e)
            }
        }
    }

    /// Mark that layout result has been consumed
    pub fn mark_consumed(&mut self) {
        if let Some(ref mut info) = self.last_info {
            info.stale = false;
        }
    }

    /// Reset the manager state
    pub fn reset(&mut self) {
        self.status = LayoutStatus::Idle;
        self.last_info = None;
        self.pending_request = None;
        self.computing.store(false, Ordering::SeqCst);
        self.generation = 0;
    }
}

impl Default for AsyncLayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for async layout configuration
pub struct AsyncLayoutConfig {
    /// Maximum time to wait for layout computation before using cached result
    pub max_wait_ms: u32,
    /// Whether to use cached layout while computing
    pub use_cached_during_compute: bool,
    /// Minimum time between layout computations (throttling)
    pub min_interval_ms: u32,
}

impl Default for AsyncLayoutConfig {
    fn default() -> Self {
        Self {
            max_wait_ms: 8, // ~1 frame at 120fps
            use_cached_during_compute: true,
            min_interval_ms: 16, // ~60fps
        }
    }
}

impl AsyncLayoutConfig {
    /// Create config optimized for 60fps
    pub fn for_60fps() -> Self {
        Self {
            max_wait_ms: 16,
            use_cached_during_compute: true,
            min_interval_ms: 16,
        }
    }

    /// Create config optimized for 120fps
    pub fn for_120fps() -> Self {
        Self {
            max_wait_ms: 8,
            use_cached_during_compute: true,
            min_interval_ms: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_layout_manager_new() {
        let manager = AsyncLayoutManager::new();
        assert_eq!(manager.status(), LayoutStatus::Idle);
        assert!(!manager.is_computing());
        assert!(!manager.has_result());
        assert_eq!(manager.generation(), 0);
    }

    #[test]
    fn test_request_layout() {
        let mut manager = AsyncLayoutManager::new();
        assert!(!manager.has_pending_request());

        manager.request_layout(Size::new(800.0, 600.0));
        assert!(manager.has_pending_request());
        assert_eq!(manager.generation(), 1);
        assert_eq!(manager.pending_size(), Some(Size::new(800.0, 600.0)));
    }

    #[test]
    fn test_multiple_requests_increment_generation() {
        let mut manager = AsyncLayoutManager::new();
        manager.request_layout(Size::new(800.0, 600.0));
        assert_eq!(manager.generation(), 1);

        manager.request_layout(Size::new(1024.0, 768.0));
        assert_eq!(manager.generation(), 2);
        assert_eq!(manager.pending_size(), Some(Size::new(1024.0, 768.0)));
    }

    #[test]
    fn test_reset() {
        let mut manager = AsyncLayoutManager::new();
        manager.request_layout(Size::new(800.0, 600.0));

        manager.reset();
        assert_eq!(manager.status(), LayoutStatus::Idle);
        assert!(!manager.has_pending_request());
        assert_eq!(manager.generation(), 0);
    }

    #[test]
    fn test_config_defaults() {
        let config = AsyncLayoutConfig::default();
        assert_eq!(config.min_interval_ms, 16);
        assert!(config.use_cached_during_compute);

        let config_60 = AsyncLayoutConfig::for_60fps();
        assert_eq!(config_60.max_wait_ms, 16);

        let config_120 = AsyncLayoutConfig::for_120fps();
        assert_eq!(config_120.max_wait_ms, 8);
    }
}
