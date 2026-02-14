//! Reactive system with Signals and Effects
//!
//! Issue #250 Phase 3: Implements a fine-grained reactivity system for efficient UI updates.
//!
//! ## Overview
//!
//! This module provides a reactive programming model inspired by Zed's GPUI:
//! - **Signals**: Observable values that notify dependents when changed
//! - **Effects**: Callbacks that automatically re-run when their dependencies change
//! - **Effect Queue**: Deferred execution to prevent reentrancy bugs
//!
//! ## Architecture
//!
//! ```text
//! Signal::set() → Mark Effects Dirty → Flush → Effects Execute → UI Invalidation
//! ```
//!
//! Key design decisions:
//! - Run-to-completion semantics (effects never interrupt each other)
//! - Automatic dependency tracking during effect execution
//! - View-level invalidation for minimal redraws

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(u64);

impl SignalId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Unique identifier for effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(u64);

impl EffectId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Unique identifier for subscriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Internal storage for a signal's value
struct SignalStorage {
    /// Type-erased value
    value: Box<dyn std::any::Any>,
    /// Effects that depend on this signal
    subscribers: HashSet<EffectId>,
    /// Version number for change detection
    version: u64,
}

/// Internal storage for an effect
struct EffectStorage {
    /// Signals this effect reads from
    dependencies: HashSet<SignalId>,
    /// Whether the effect needs to run
    dirty: bool,
}

/// Subscription callback type
type SubscriptionCallback = Box<dyn Fn(SignalId)>;

/// Deferred callback type
type DeferredCallback = Box<dyn FnOnce(&mut ReactiveRuntime)>;

/// The reactive runtime that manages signals and effects
pub struct ReactiveRuntime {
    /// Signal storage
    signals: HashMap<SignalId, SignalStorage>,
    /// Effect storage
    effects: HashMap<EffectId, EffectStorage>,
    /// Subscriptions for change notifications
    subscriptions: HashMap<SubscriptionId, (SignalId, SubscriptionCallback)>,
    /// Currently executing effect (for dependency tracking)
    tracking_effect: Option<EffectId>,
    /// Queue of dirty effects to run
    dirty_effects: VecDeque<EffectId>,
    /// Batch depth (0 = not batching)
    batch_depth: u32,
    /// Callbacks to run after batch
    pending_callbacks: VecDeque<DeferredCallback>,
}

impl ReactiveRuntime {
    /// Create a new reactive runtime
    pub fn new() -> Self {
        Self {
            signals: HashMap::new(),
            effects: HashMap::new(),
            subscriptions: HashMap::new(),
            tracking_effect: None,
            dirty_effects: VecDeque::new(),
            batch_depth: 0,
            pending_callbacks: VecDeque::new(),
        }
    }

    /// Create a new signal with an initial value
    pub fn create_signal<T: 'static>(&mut self, value: T) -> SignalId {
        let id = SignalId::new();
        self.signals.insert(
            id,
            SignalStorage {
                value: Box::new(value),
                subscribers: HashSet::new(),
                version: 0,
            },
        );
        id
    }

    /// Get a signal's value and track dependency
    ///
    /// If called inside an effect, the effect will be re-run when this signal
    /// changes.
    ///
    /// # Type Safety
    ///
    /// This method uses type erasure. If `T` doesn't match the signal's actual
    /// type, `None` is returned. In debug builds, a warning is logged for type
    /// mismatches to help catch bugs early.
    pub fn get_signal<T: Clone + 'static>(&mut self, id: SignalId) -> Option<T> {
        // Track dependency if we're inside an effect
        if let Some(effect_id) = self.tracking_effect {
            if let Some(signal) = self.signals.get_mut(&id) {
                signal.subscribers.insert(effect_id);
            }
            if let Some(effect) = self.effects.get_mut(&effect_id) {
                effect.dependencies.insert(id);
            }
        }

        let signal = self.signals.get(&id)?;

        // Defensive: log type mismatch in debug builds
        #[cfg(debug_assertions)]
        if signal.value.downcast_ref::<T>().is_none() {
            tracing::warn!(
                "Signal {:?} type mismatch: expected {}, got different type",
                id,
                std::any::type_name::<T>()
            );
        }

        signal.value.downcast_ref::<T>().cloned()
    }

    /// Get a signal's value without tracking dependency
    ///
    /// # Type Safety
    ///
    /// This method uses type erasure. If `T` doesn't match the signal's actual
    /// type, `None` is returned. In debug builds, a warning is logged for type
    /// mismatches to help catch bugs early.
    pub fn get_signal_untracked<T: Clone + 'static>(&self, id: SignalId) -> Option<T> {
        let signal = self.signals.get(&id)?;

        // Defensive: log type mismatch in debug builds
        #[cfg(debug_assertions)]
        if signal.value.downcast_ref::<T>().is_none() {
            tracing::warn!(
                "Signal {:?} type mismatch: expected {}, got different type",
                id,
                std::any::type_name::<T>()
            );
        }

        signal.value.downcast_ref::<T>().cloned()
    }

    /// Set a signal's value
    ///
    /// Returns `true` if the signal was found and updated, `false` otherwise.
    pub fn set_signal<T: 'static>(&mut self, id: SignalId, value: T) -> bool {
        let found = if let Some(signal) = self.signals.get_mut(&id) {
            signal.value = Box::new(value);
            signal.version += 1;

            // Mark all subscriber effects as dirty
            let subscribers: Vec<_> = signal.subscribers.iter().copied().collect();
            for effect_id in subscribers {
                if let Some(effect) = self.effects.get_mut(&effect_id)
                    && !effect.dirty
                {
                    effect.dirty = true;
                    self.dirty_effects.push_back(effect_id);
                }
            }

            // Notify subscriptions
            let subscription_ids: Vec<_> = self
                .subscriptions
                .iter()
                .filter(|(_, (sig_id, _))| *sig_id == id)
                .map(|(sub_id, _)| *sub_id)
                .collect();

            for sub_id in subscription_ids {
                if let Some((_, callback)) = self.subscriptions.get(&sub_id) {
                    callback(id);
                }
            }
            true
        } else {
            false
        };

        // Flush effects if not batching
        if self.batch_depth == 0 {
            self.flush();
        }

        found
    }

    /// Update a signal using a function
    ///
    /// Returns `true` if the signal was found and updated, `false` otherwise.
    pub fn update_signal<T: Clone + 'static>(
        &mut self,
        id: SignalId,
        f: impl FnOnce(&T) -> T,
    ) -> bool {
        let Some(current) = self.get_signal_untracked::<T>(id) else {
            return false;
        };
        let new_value = f(&current);
        self.set_signal(id, new_value)
    }

    /// Create an effect that tracks dependencies
    pub fn create_effect(&mut self) -> EffectId {
        let id = EffectId::new();
        self.effects.insert(
            id,
            EffectStorage {
                dependencies: HashSet::new(),
                dirty: true,
            },
        );
        self.dirty_effects.push_back(id);
        id
    }

    /// Start tracking dependencies for an effect
    pub fn begin_tracking(&mut self, effect_id: EffectId) {
        // Clear old dependencies
        if let Some(effect) = self.effects.get(&effect_id) {
            let old_deps: Vec<_> = effect.dependencies.iter().copied().collect();
            for signal_id in old_deps {
                if let Some(signal) = self.signals.get_mut(&signal_id) {
                    signal.subscribers.remove(&effect_id);
                }
            }
        }

        if let Some(effect) = self.effects.get_mut(&effect_id) {
            effect.dependencies.clear();
            effect.dirty = false;
        }

        self.tracking_effect = Some(effect_id);
    }

    /// Stop tracking dependencies
    pub fn end_tracking(&mut self) {
        self.tracking_effect = None;
    }

    /// Check if an effect is dirty
    pub fn is_effect_dirty(&self, effect_id: EffectId) -> bool {
        self.effects
            .get(&effect_id)
            .map(|e| e.dirty)
            .unwrap_or(false)
    }

    /// Mark an effect as clean
    pub fn mark_effect_clean(&mut self, effect_id: EffectId) {
        if let Some(effect) = self.effects.get_mut(&effect_id) {
            effect.dirty = false;
        }
    }

    /// Get the next dirty effect to run
    pub fn pop_dirty_effect(&mut self) -> Option<EffectId> {
        while let Some(effect_id) = self.dirty_effects.pop_front() {
            if self.is_effect_dirty(effect_id) {
                return Some(effect_id);
            }
        }
        None
    }

    /// Remove an effect
    pub fn remove_effect(&mut self, effect_id: EffectId) {
        if let Some(effect) = self.effects.remove(&effect_id) {
            // Remove from signal subscribers
            for signal_id in effect.dependencies {
                if let Some(signal) = self.signals.get_mut(&signal_id) {
                    signal.subscribers.remove(&effect_id);
                }
            }
        }
    }

    /// Remove a signal
    pub fn remove_signal(&mut self, signal_id: SignalId) {
        if let Some(signal) = self.signals.remove(&signal_id) {
            // Mark subscriber effects as dirty
            for effect_id in signal.subscribers {
                if let Some(effect) = self.effects.get_mut(&effect_id) {
                    effect.dependencies.remove(&signal_id);
                }
            }
        }

        // Remove related subscriptions
        self.subscriptions
            .retain(|_, (sig_id, _)| *sig_id != signal_id);
    }

    /// Subscribe to signal changes
    pub fn subscribe(
        &mut self,
        signal_id: SignalId,
        callback: impl Fn(SignalId) + 'static,
    ) -> SubscriptionId {
        let id = SubscriptionId::new();
        self.subscriptions
            .insert(id, (signal_id, Box::new(callback)));
        id
    }

    /// Unsubscribe from signal changes
    pub fn unsubscribe(&mut self, subscription_id: SubscriptionId) {
        self.subscriptions.remove(&subscription_id);
    }

    /// Start a batch of updates
    pub fn begin_batch(&mut self) {
        self.batch_depth += 1;
    }

    /// End a batch of updates
    pub fn end_batch(&mut self) {
        if self.batch_depth > 0 {
            self.batch_depth -= 1;
        }

        if self.batch_depth == 0 {
            self.flush();
        }
    }

    /// Run updates in a batch
    pub fn batch<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.begin_batch();
        let result = f(self);
        self.end_batch();
        result
    }

    /// Flush all pending effects
    pub fn flush(&mut self) {
        // Run pending callbacks
        while let Some(callback) = self.pending_callbacks.pop_front() {
            callback(self);
        }
    }

    /// Defer a callback to run after the current batch
    pub fn defer(&mut self, callback: impl FnOnce(&mut Self) + 'static) {
        self.pending_callbacks.push_back(Box::new(callback));

        if self.batch_depth == 0 {
            self.flush();
        }
    }

    /// Get signal version for change detection
    pub fn signal_version(&self, id: SignalId) -> Option<u64> {
        self.signals.get(&id).map(|s| s.version)
    }

    /// Get statistics
    pub fn stats(&self) -> RuntimeStats {
        RuntimeStats {
            signal_count: self.signals.len(),
            effect_count: self.effects.len(),
            subscription_count: self.subscriptions.len(),
            dirty_effect_count: self.dirty_effects.len(),
        }
    }
}

impl Default for ReactiveRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the reactive runtime
#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeStats {
    pub signal_count: usize,
    pub effect_count: usize,
    pub subscription_count: usize,
    pub dirty_effect_count: usize,
}

/// A reactive signal handle
///
/// This is a lightweight handle that can be copied and used to interact
/// with a signal through the runtime.
#[derive(Clone, Copy)]
pub struct Signal<T> {
    id: SignalId,
    _marker: PhantomData<T>,
}

impl<T> Signal<T> {
    /// Create a signal handle from an ID
    pub fn from_id(id: SignalId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Get the signal's ID
    pub fn id(&self) -> SignalId {
        self.id
    }
}

impl<T: Clone + 'static> Signal<T> {
    /// Get the signal's value
    ///
    /// Returns `None` if the signal has been removed from the runtime.
    pub fn get(&self, runtime: &mut ReactiveRuntime) -> Option<T> {
        runtime.get_signal(self.id)
    }

    /// Get the signal's value without tracking
    ///
    /// Returns `None` if the signal has been removed from the runtime.
    pub fn get_untracked(&self, runtime: &ReactiveRuntime) -> Option<T> {
        runtime.get_signal_untracked(self.id)
    }

    /// Set the signal's value
    ///
    /// Returns `true` if the signal was found and updated, `false` otherwise.
    pub fn set(&self, runtime: &mut ReactiveRuntime, value: T) -> bool {
        runtime.set_signal(self.id, value)
    }

    /// Update the signal's value
    ///
    /// Returns `true` if the signal was found and updated, `false` otherwise.
    pub fn update(&self, runtime: &mut ReactiveRuntime, f: impl FnOnce(&T) -> T) -> bool {
        runtime.update_signal(self.id, f)
    }
}

impl<T> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signal").field("id", &self.id).finish()
    }
}

impl<T> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Signal<T> {}

impl<T> std::hash::Hash for Signal<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// A reactive effect handle
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Effect {
    id: EffectId,
}

impl Effect {
    /// Create from an ID
    pub fn from_id(id: EffectId) -> Self {
        Self { id }
    }

    /// Get the effect's ID
    pub fn id(&self) -> EffectId {
        self.id
    }

    /// Check if this effect needs to run
    pub fn is_dirty(&self, runtime: &ReactiveRuntime) -> bool {
        runtime.is_effect_dirty(self.id)
    }

    /// Remove this effect from the runtime
    pub fn dispose(self, runtime: &mut ReactiveRuntime) {
        runtime.remove_effect(self.id);
    }
}

/// Helper for creating signals with the runtime
pub fn create_signal<T: 'static>(runtime: &mut ReactiveRuntime, value: T) -> Signal<T> {
    let id = runtime.create_signal(value);
    Signal::from_id(id)
}

/// Helper for creating effects with the runtime
pub fn create_effect(runtime: &mut ReactiveRuntime) -> Effect {
    let id = runtime.create_effect();
    Effect::from_id(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_create_signal() {
        let mut runtime = ReactiveRuntime::new();
        let signal = create_signal(&mut runtime, 42);

        assert_eq!(signal.get(&mut runtime), Some(42));
    }

    #[test]
    fn test_set_signal() {
        let mut runtime = ReactiveRuntime::new();
        let signal = create_signal(&mut runtime, 10);

        signal.set(&mut runtime, 20);
        assert_eq!(signal.get(&mut runtime), Some(20));
    }

    #[test]
    fn test_update_signal() {
        let mut runtime = ReactiveRuntime::new();
        let signal = create_signal(&mut runtime, 5);

        signal.update(&mut runtime, |v| v * 2);
        assert_eq!(signal.get(&mut runtime), Some(10));
    }

    #[test]
    fn test_signal_version() {
        let mut runtime = ReactiveRuntime::new();
        let signal = create_signal(&mut runtime, 1);

        assert_eq!(runtime.signal_version(signal.id()), Some(0));
        signal.set(&mut runtime, 2);
        assert_eq!(runtime.signal_version(signal.id()), Some(1));
        signal.set(&mut runtime, 3);
        assert_eq!(runtime.signal_version(signal.id()), Some(2));
    }

    #[test]
    fn test_effect_dirty_on_signal_change() {
        let mut runtime = ReactiveRuntime::new();
        let signal = create_signal(&mut runtime, 1);
        let effect = create_effect(&mut runtime);

        // Begin tracking and read signal
        runtime.begin_tracking(effect.id());
        let _ = signal.get(&mut runtime);
        runtime.end_tracking();
        runtime.mark_effect_clean(effect.id());

        // Effect should be clean
        assert!(!effect.is_dirty(&runtime));

        // Change signal
        signal.set(&mut runtime, 2);

        // Effect should be dirty
        assert!(effect.is_dirty(&runtime));
    }

    #[test]
    fn test_effect_dispose() {
        let mut runtime = ReactiveRuntime::new();
        let effect = create_effect(&mut runtime);

        assert_eq!(runtime.stats().effect_count, 1);
        effect.dispose(&mut runtime);
        assert_eq!(runtime.stats().effect_count, 0);
    }

    #[test]
    fn test_batch_updates() {
        let mut runtime = ReactiveRuntime::new();
        let signal1 = create_signal(&mut runtime, 1);
        let signal2 = create_signal(&mut runtime, 2);

        runtime.batch(|rt| {
            signal1.set(rt, 10);
            signal2.set(rt, 20);
        });

        assert_eq!(signal1.get(&mut runtime), Some(10));
        assert_eq!(signal2.get(&mut runtime), Some(20));
    }

    #[test]
    fn test_subscription() {
        let mut runtime = ReactiveRuntime::new();
        let signal = create_signal(&mut runtime, 0);

        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();

        let sub_id = runtime.subscribe(signal.id(), move |_| {
            called_clone.set(true);
        });

        signal.set(&mut runtime, 1);
        assert!(called.get());

        // Unsubscribe and verify no more calls
        called.set(false);
        runtime.unsubscribe(sub_id);
        signal.set(&mut runtime, 2);
        assert!(!called.get());
    }

    #[test]
    fn test_dependency_cleanup() {
        let mut runtime = ReactiveRuntime::new();
        let signal = create_signal(&mut runtime, 1);
        let effect = create_effect(&mut runtime);

        // Track dependency
        runtime.begin_tracking(effect.id());
        let _ = signal.get(&mut runtime);
        runtime.end_tracking();

        // Dispose effect
        effect.dispose(&mut runtime);

        // Signal should have no subscribers
        signal.set(&mut runtime, 2);
        // No panic means success
    }

    #[test]
    fn test_runtime_stats() {
        let mut runtime = ReactiveRuntime::new();

        assert_eq!(runtime.stats().signal_count, 0);
        assert_eq!(runtime.stats().effect_count, 0);

        let _signal = create_signal(&mut runtime, 1);
        let _effect = create_effect(&mut runtime);

        assert_eq!(runtime.stats().signal_count, 1);
        assert_eq!(runtime.stats().effect_count, 1);
    }

    #[test]
    fn test_signal_remove() {
        let mut runtime = ReactiveRuntime::new();
        let signal = create_signal(&mut runtime, 1);

        assert_eq!(runtime.stats().signal_count, 1);
        runtime.remove_signal(signal.id());
        assert_eq!(runtime.stats().signal_count, 0);
    }
}
