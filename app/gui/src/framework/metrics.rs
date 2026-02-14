//! Performance metrics for GUI framework
//!
//! Provides frame time measurement, FPS calculation, and performance statistics
//! for optimizing the rendering pipeline.
//!
//! Issue #250: Optimize GUI framework for 60+ FPS (target: 120 FPS)

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Frame timing metrics for a single frame
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameTiming {
    /// Total frame time (ms)
    pub total_ms: f32,
    /// Layout computation time (ms)
    pub layout_ms: f32,
    /// Paint phase time (ms)
    pub paint_ms: f32,
    /// GPU submission time (ms)
    pub gpu_submit_ms: f32,
}

/// Aggregated performance statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct PerformanceStats {
    /// Current frames per second
    pub fps: f32,
    /// Average frame time in milliseconds
    pub avg_frame_time_ms: f32,
    /// Maximum frame time in the sample window
    pub max_frame_time_ms: f32,
    /// Minimum frame time in the sample window
    pub min_frame_time_ms: f32,
    /// 95th percentile frame time (Issue #111: acceptance criteria)
    pub p95_frame_time_ms: f32,
    /// Average layout time in milliseconds
    pub avg_layout_ms: f32,
    /// Average paint time in milliseconds
    pub avg_paint_ms: f32,
    /// Average GPU submit time in milliseconds
    pub avg_gpu_submit_ms: f32,
    /// Number of draw calls in last frame
    pub draw_calls: u32,
    /// Number of quads rendered in last frame
    pub quad_count: u32,
    /// Number of glyphs rendered in last frame
    pub glyph_count: u32,
    /// Number of dirty elements that needed repaint
    pub dirty_elements: u32,
}

impl PerformanceStats {
    /// Format stats as a display string for overlay
    pub fn display_string(&self) -> String {
        format!(
            "FPS: {:.0} | Frame: {:.2}ms | Layout: {:.2}ms | Paint: {:.2}ms | Draws: {}",
            self.fps,
            self.avg_frame_time_ms,
            self.avg_layout_ms,
            self.avg_paint_ms,
            self.draw_calls
        )
    }

    /// Format detailed stats for performance testing (Issue #111)
    pub fn detailed_string(&self) -> String {
        format!(
            "FPS: {:.1} | Avg: {:.2}ms | P95: {:.2}ms | Max: {:.2}ms | Layout: {:.2}ms | Paint: {:.2}ms | GPU: {:.2}ms | Draws: {} | Quads: {}",
            self.fps,
            self.avg_frame_time_ms,
            self.p95_frame_time_ms,
            self.max_frame_time_ms,
            self.avg_layout_ms,
            self.avg_paint_ms,
            self.avg_gpu_submit_ms,
            self.draw_calls,
            self.quad_count
        )
    }

    /// Check if we're meeting the 60 FPS target (Issue #111)
    /// Uses 95th percentile as per acceptance criteria: "P95 < 16.67ms"
    pub fn meets_60fps(&self) -> bool {
        self.p95_frame_time_ms <= 16.67
    }

    /// Check if we're meeting the 60 FPS target based on average
    pub fn meets_60fps_avg(&self) -> bool {
        self.avg_frame_time_ms <= 16.67
    }

    /// Check if we're meeting the 120 FPS target
    pub fn meets_120fps(&self) -> bool {
        self.avg_frame_time_ms <= 8.33
    }

    /// Check if frame time breakdown is within targets (Issue #111)
    /// Target: Layout < 2ms, Paint < 3ms, GPU < 11ms
    pub fn meets_breakdown_targets(&self) -> bool {
        self.avg_layout_ms < 2.0 && self.avg_paint_ms < 3.0 && self.avg_gpu_submit_ms < 11.0
    }
}

/// Performance metrics collector and analyzer
pub struct FrameMetrics {
    /// History of frame timings for averaging
    frame_history: VecDeque<FrameTiming>,
    /// Maximum number of frames to keep in history
    max_history_size: usize,
    /// Start time of current frame
    frame_start: Option<Instant>,
    /// Layout phase timing
    layout_start: Option<Instant>,
    layout_end: Option<Instant>,
    /// Paint phase timing
    paint_start: Option<Instant>,
    paint_end: Option<Instant>,
    /// GPU submit timing
    gpu_submit_start: Option<Instant>,
    gpu_submit_end: Option<Instant>,
    /// Last computed FPS (updated periodically)
    last_fps: f32,
    /// Frame count since last FPS update
    frames_since_fps_update: u32,
    /// Time of last FPS update
    last_fps_update: Instant,
    /// Draw calls in current frame
    current_draw_calls: u32,
    /// Quads in current frame
    current_quad_count: u32,
    /// Glyphs in current frame
    current_glyph_count: u32,
    /// Dirty elements in current frame
    current_dirty_elements: u32,
    /// Total frames rendered since start
    total_frame_count: u64,
}

impl FrameMetrics {
    /// Create new metrics collector with default 60-frame history
    pub fn new() -> Self {
        Self::with_history_size(60)
    }

    /// Create with custom history size
    pub fn with_history_size(size: usize) -> Self {
        Self {
            frame_history: VecDeque::with_capacity(size),
            max_history_size: size,
            frame_start: None,
            layout_start: None,
            layout_end: None,
            paint_start: None,
            paint_end: None,
            gpu_submit_start: None,
            gpu_submit_end: None,
            last_fps: 0.0,
            frames_since_fps_update: 0,
            last_fps_update: Instant::now(),
            current_draw_calls: 0,
            current_quad_count: 0,
            current_glyph_count: 0,
            current_dirty_elements: 0,
            total_frame_count: 0,
        }
    }

    /// Start timing a new frame
    pub fn begin_frame(&mut self) {
        self.frame_start = Some(Instant::now());
        self.layout_start = None;
        self.layout_end = None;
        self.paint_start = None;
        self.paint_end = None;
        self.gpu_submit_start = None;
        self.gpu_submit_end = None;
        self.current_draw_calls = 0;
        self.current_quad_count = 0;
        self.current_glyph_count = 0;
        self.current_dirty_elements = 0;
    }

    /// Start timing layout phase
    pub fn begin_layout(&mut self) {
        self.layout_start = Some(Instant::now());
    }

    /// End timing layout phase
    pub fn end_layout(&mut self) {
        self.layout_end = Some(Instant::now());
    }

    /// Start timing paint phase
    pub fn begin_paint(&mut self) {
        self.paint_start = Some(Instant::now());
    }

    /// End timing paint phase
    pub fn end_paint(&mut self) {
        self.paint_end = Some(Instant::now());
    }

    /// Start timing GPU submission
    pub fn begin_gpu_submit(&mut self) {
        self.gpu_submit_start = Some(Instant::now());
    }

    /// End timing GPU submission
    pub fn end_gpu_submit(&mut self) {
        self.gpu_submit_end = Some(Instant::now());
    }

    /// Record draw call count for current frame
    pub fn record_draw_calls(&mut self, count: u32) {
        self.current_draw_calls = count;
    }

    /// Record quad count for current frame
    pub fn record_quad_count(&mut self, count: u32) {
        self.current_quad_count = count;
    }

    /// Record glyph count for current frame
    pub fn record_glyph_count(&mut self, count: u32) {
        self.current_glyph_count = count;
    }

    /// Record dirty element count for current frame
    pub fn record_dirty_elements(&mut self, count: u32) {
        self.current_dirty_elements = count;
    }

    /// End frame and record timing data
    pub fn end_frame(&mut self) {
        let Some(frame_start) = self.frame_start else {
            return;
        };

        let now = Instant::now();
        let total = now.duration_since(frame_start);

        let layout_duration = match (self.layout_start, self.layout_end) {
            (Some(start), Some(end)) => end.duration_since(start),
            _ => Duration::ZERO,
        };

        let paint_duration = match (self.paint_start, self.paint_end) {
            (Some(start), Some(end)) => end.duration_since(start),
            _ => Duration::ZERO,
        };

        let gpu_submit_duration = match (self.gpu_submit_start, self.gpu_submit_end) {
            (Some(start), Some(end)) => end.duration_since(start),
            _ => Duration::ZERO,
        };

        let timing = FrameTiming {
            total_ms: total.as_secs_f32() * 1000.0,
            layout_ms: layout_duration.as_secs_f32() * 1000.0,
            paint_ms: paint_duration.as_secs_f32() * 1000.0,
            gpu_submit_ms: gpu_submit_duration.as_secs_f32() * 1000.0,
        };

        // Add to history, removing oldest if at capacity
        if self.frame_history.len() >= self.max_history_size {
            self.frame_history.pop_front();
        }
        self.frame_history.push_back(timing);

        // Update FPS counter (every 0.5 seconds)
        self.frames_since_fps_update += 1;
        self.total_frame_count += 1;
        let fps_update_interval = Duration::from_millis(500);
        if now.duration_since(self.last_fps_update) >= fps_update_interval {
            let elapsed_secs = now.duration_since(self.last_fps_update).as_secs_f32();
            self.last_fps = self.frames_since_fps_update as f32 / elapsed_secs;
            self.frames_since_fps_update = 0;
            self.last_fps_update = now;
        }
    }

    /// Get total number of frames rendered since start
    pub fn total_frames(&self) -> u64 {
        self.total_frame_count
    }

    /// Get current performance statistics
    pub fn get_stats(&self) -> PerformanceStats {
        if self.frame_history.is_empty() {
            return PerformanceStats::default();
        }

        let count = self.frame_history.len() as f32;

        let total_time: f32 = self.frame_history.iter().map(|f| f.total_ms).sum();
        let total_layout: f32 = self.frame_history.iter().map(|f| f.layout_ms).sum();
        let total_paint: f32 = self.frame_history.iter().map(|f| f.paint_ms).sum();
        let total_gpu: f32 = self.frame_history.iter().map(|f| f.gpu_submit_ms).sum();

        let max_frame = self
            .frame_history
            .iter()
            .map(|f| f.total_ms)
            .fold(0.0f32, f32::max);
        let min_frame = self
            .frame_history
            .iter()
            .map(|f| f.total_ms)
            .fold(f32::MAX, f32::min);

        // Calculate 95th percentile (Issue #111)
        let p95_frame = self.calculate_percentile(95.0);

        PerformanceStats {
            fps: self.last_fps,
            avg_frame_time_ms: total_time / count,
            max_frame_time_ms: max_frame,
            min_frame_time_ms: min_frame,
            p95_frame_time_ms: p95_frame,
            avg_layout_ms: total_layout / count,
            avg_paint_ms: total_paint / count,
            avg_gpu_submit_ms: total_gpu / count,
            draw_calls: self.current_draw_calls,
            quad_count: self.current_quad_count,
            glyph_count: self.current_glyph_count,
            dirty_elements: self.current_dirty_elements,
        }
    }

    /// Calculate percentile value from frame history (Issue #111)
    ///
    /// Uses linear interpolation for accurate percentile calculation.
    fn calculate_percentile(&self, percentile: f32) -> f32 {
        if self.frame_history.is_empty() {
            return 0.0;
        }

        // Collect and sort frame times
        let mut times: Vec<f32> = self.frame_history.iter().map(|f| f.total_ms).collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate percentile index with linear interpolation
        let len = times.len();
        let index = (percentile / 100.0) * (len.saturating_sub(1)) as f32;
        let lower_index = index.floor() as usize;
        let upper_index = (lower_index.saturating_add(1)).min(len.saturating_sub(1));

        // Linear interpolation between lower and upper values
        if lower_index == upper_index {
            times[lower_index]
        } else {
            let lower_value = times[lower_index];
            let upper_value = times[upper_index];
            let fraction = index - lower_index as f32;
            lower_value + (upper_value - lower_value) * fraction
        }
    }

    /// Get current FPS
    pub fn fps(&self) -> f32 {
        self.last_fps
    }

    /// Get last frame timing
    pub fn last_frame(&self) -> Option<FrameTiming> {
        self.frame_history.back().copied()
    }
}

impl Default for FrameMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped timer for measuring code sections
pub struct ScopedTimer {
    start: Instant,
    name: &'static str,
}

impl ScopedTimer {
    pub fn new(name: &'static str) -> Self {
        Self {
            start: Instant::now(),
            name,
        }
    }

    pub fn elapsed_ms(&self) -> f32 {
        self.start.elapsed().as_secs_f32() * 1000.0
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        tracing::trace!("{}: {:.3}ms", self.name, self.elapsed_ms());
    }
}

/// Macro for creating scoped timers
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _timer = $crate::framework::metrics::ScopedTimer::new($name);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_frame_metrics_basic() {
        let mut metrics = FrameMetrics::new();

        metrics.begin_frame();
        thread::sleep(Duration::from_millis(1));
        metrics.begin_layout();
        thread::sleep(Duration::from_micros(100));
        metrics.end_layout();
        metrics.begin_paint();
        thread::sleep(Duration::from_micros(100));
        metrics.end_paint();
        metrics.end_frame();

        let timing = metrics.last_frame().unwrap();
        assert!(timing.total_ms > 1.0);
        assert!(timing.layout_ms > 0.0);
        assert!(timing.paint_ms > 0.0);
    }

    #[test]
    fn test_performance_stats() {
        let mut metrics = FrameMetrics::with_history_size(10);

        // Record a few frames
        for _ in 0..5 {
            metrics.begin_frame();
            thread::sleep(Duration::from_millis(1));
            metrics.end_frame();
        }

        let stats = metrics.get_stats();
        assert!(stats.avg_frame_time_ms >= 1.0);
        assert!(stats.max_frame_time_ms >= stats.min_frame_time_ms);
    }

    #[test]
    fn test_meets_fps_targets() {
        // Issue #111: Test uses P95 for 60 FPS validation
        let stats = PerformanceStats {
            fps: 60.0,
            avg_frame_time_ms: 10.0,
            p95_frame_time_ms: 15.0,
            ..Default::default()
        };
        assert!(stats.meets_60fps());
        assert!(!stats.meets_120fps());

        let stats = PerformanceStats {
            fps: 120.0,
            avg_frame_time_ms: 8.0,
            p95_frame_time_ms: 8.0,
            ..Default::default()
        };
        assert!(stats.meets_60fps());
        assert!(stats.meets_120fps());

        let stats = PerformanceStats {
            fps: 30.0,
            avg_frame_time_ms: 33.0,
            p95_frame_time_ms: 35.0,
            ..Default::default()
        };
        assert!(!stats.meets_60fps());
        assert!(!stats.meets_120fps());

        // Edge case: P95 above target but average below
        let stats = PerformanceStats {
            fps: 60.0,
            avg_frame_time_ms: 14.0,
            p95_frame_time_ms: 18.0,
            ..Default::default()
        };
        assert!(!stats.meets_60fps()); // P95 check should fail
        assert!(stats.meets_60fps_avg()); // But average is OK
    }

    #[test]
    fn test_percentile_calculation() {
        let mut metrics = FrameMetrics::with_history_size(100);

        // Add frames with known values
        for i in 0..100 {
            metrics.begin_frame();
            // Simulate varying frame times: 10ms base + 0.1ms per frame
            let sleep_time = Duration::from_micros(10000 + i * 100);
            thread::sleep(sleep_time);
            metrics.end_frame();
        }

        let p95 = metrics.calculate_percentile(95.0);
        assert!(p95 > 0.0);
        assert!(p95 < 100.0); // Should be reasonable

        // P95 should be higher than average but lower than max
        let stats = metrics.get_stats();
        assert!(stats.p95_frame_time_ms >= stats.avg_frame_time_ms);
        assert!(stats.p95_frame_time_ms <= stats.max_frame_time_ms);
    }

    #[test]
    fn test_breakdown_targets() {
        // Meeting all targets
        let stats = PerformanceStats {
            avg_layout_ms: 1.5,
            avg_paint_ms: 2.5,
            avg_gpu_submit_ms: 10.0,
            ..Default::default()
        };
        assert!(stats.meets_breakdown_targets());

        // Layout too slow
        let stats = PerformanceStats {
            avg_layout_ms: 2.5,
            avg_paint_ms: 2.5,
            avg_gpu_submit_ms: 10.0,
            ..Default::default()
        };
        assert!(!stats.meets_breakdown_targets());

        // Paint too slow
        let stats = PerformanceStats {
            avg_layout_ms: 1.5,
            avg_paint_ms: 3.5,
            avg_gpu_submit_ms: 10.0,
            ..Default::default()
        };
        assert!(!stats.meets_breakdown_targets());

        // GPU too slow
        let stats = PerformanceStats {
            avg_layout_ms: 1.5,
            avg_paint_ms: 2.5,
            avg_gpu_submit_ms: 12.0,
            ..Default::default()
        };
        assert!(!stats.meets_breakdown_targets());
    }
}
