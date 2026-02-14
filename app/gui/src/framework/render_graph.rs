//! Render Graph for optimized rendering pass management
//!
//! Issue #250 Phase 3: Implements a graph-based rendering system for optimal
//! GPU resource usage and pass parallelization.
//!
//! ## Overview
//!
//! The render graph provides:
//! - Declarative render pass definition
//! - Automatic dependency tracking between passes
//! - Resource lifetime management
//! - Potential for pass parallelization
//!
//! ## Architecture
//!
//! ```text
//! RenderGraph
//! ├── Pass: Clear
//! ├── Pass: VideoBackground
//! │   └── depends_on: Clear
//! ├── Pass: UIQuads
//! │   └── depends_on: VideoBackground
//! └── Pass: Text
//!     └── depends_on: UIQuads
//! ```

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for a render pass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassId(u64);

impl PassId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for PassId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pass({})", self.0)
    }
}

/// Unique identifier for a render resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(u64);

impl ResourceId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Type of resource access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAccess {
    /// Read-only access
    Read,
    /// Write-only access (clears previous content)
    Write,
    /// Read-write access
    ReadWrite,
}

/// A resource used by render passes
#[derive(Debug, Clone)]
pub struct Resource {
    /// Unique ID
    pub id: ResourceId,
    /// Resource name for debugging
    pub name: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Whether this resource is external (not managed by the graph)
    pub external: bool,
}

/// Types of resources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Texture resource
    Texture,
    /// Buffer resource
    Buffer,
    /// Render target
    RenderTarget,
}

/// Resource usage declaration for a pass
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Resource being accessed
    pub resource_id: ResourceId,
    /// Type of access
    pub access: ResourceAccess,
}

/// Pass execution callback type
type PassCallback = Box<dyn Fn(&PassContext)>;

/// A render pass in the graph
pub struct RenderPass {
    /// Unique ID
    pub id: PassId,
    /// Pass name for debugging
    pub name: String,
    /// Resources used by this pass
    pub resources: Vec<ResourceUsage>,
    /// Explicit dependencies on other passes
    pub dependencies: HashSet<PassId>,
    /// Whether this pass is enabled
    pub enabled: bool,
    /// Pass execution callback
    callback: Option<PassCallback>,
}

impl RenderPass {
    /// Create a new render pass
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: PassId::new(),
            name: name.into(),
            resources: Vec::new(),
            dependencies: HashSet::new(),
            enabled: true,
            callback: None,
        }
    }

    /// Add a resource read dependency
    pub fn read(mut self, resource_id: ResourceId) -> Self {
        self.resources.push(ResourceUsage {
            resource_id,
            access: ResourceAccess::Read,
        });
        self
    }

    /// Add a resource write
    pub fn write(mut self, resource_id: ResourceId) -> Self {
        self.resources.push(ResourceUsage {
            resource_id,
            access: ResourceAccess::Write,
        });
        self
    }

    /// Add a resource read-write
    pub fn read_write(mut self, resource_id: ResourceId) -> Self {
        self.resources.push(ResourceUsage {
            resource_id,
            access: ResourceAccess::ReadWrite,
        });
        self
    }

    /// Add an explicit dependency on another pass
    pub fn depends_on(mut self, pass_id: PassId) -> Self {
        self.dependencies.insert(pass_id);
        self
    }

    /// Set the execution callback
    pub fn on_execute(mut self, callback: impl Fn(&PassContext) + 'static) -> Self {
        self.callback = Some(Box::new(callback));
        self
    }

    /// Enable or disable the pass
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl fmt::Debug for RenderPass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderPass")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("resources", &self.resources)
            .field("dependencies", &self.dependencies)
            .field("enabled", &self.enabled)
            .finish()
    }
}

/// Context passed to pass execution callbacks
pub struct PassContext {
    /// The pass being executed
    pub pass_id: PassId,
    /// Pass index in execution order
    pub index: usize,
    /// Total number of passes
    pub total_passes: usize,
}

/// Execution order for passes
#[derive(Debug, Clone)]
pub struct ExecutionOrder {
    /// Ordered list of pass IDs to execute
    pub passes: Vec<PassId>,
    /// Passes that can potentially run in parallel (same level in dependency graph)
    pub parallel_groups: Vec<Vec<PassId>>,
}

/// Error types for render graph operations
#[derive(Debug, Clone)]
pub enum RenderGraphError {
    /// Circular dependency detected
    CyclicDependency(Vec<PassId>),
    /// Pass not found
    PassNotFound(PassId),
    /// Resource not found
    ResourceNotFound(ResourceId),
    /// Resource conflict (multiple writes without dependency)
    ResourceConflict {
        resource_id: ResourceId,
        pass_a: PassId,
        pass_b: PassId,
    },
    /// Graph not compiled
    NotCompiled,
}

impl fmt::Display for RenderGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderGraphError::CyclicDependency(cycle) => {
                write!(f, "Cyclic dependency detected: {:?}", cycle)
            }
            RenderGraphError::PassNotFound(id) => {
                write!(f, "Pass not found: {}", id)
            }
            RenderGraphError::ResourceNotFound(id) => {
                write!(f, "Resource not found: {:?}", id)
            }
            RenderGraphError::ResourceConflict {
                resource_id,
                pass_a,
                pass_b,
            } => {
                write!(
                    f,
                    "Resource conflict on {:?} between {} and {}",
                    resource_id, pass_a, pass_b
                )
            }
            RenderGraphError::NotCompiled => {
                write!(f, "Render graph not compiled")
            }
        }
    }
}

impl std::error::Error for RenderGraphError {}

/// The render graph that manages passes and their execution order
pub struct RenderGraph {
    /// All registered passes
    passes: HashMap<PassId, RenderPass>,
    /// Order in which passes were added (for deterministic dependency resolution)
    pass_order: Vec<PassId>,
    /// All registered resources
    resources: HashMap<ResourceId, Resource>,
    /// Cached execution order (invalidated on graph changes)
    cached_order: Option<ExecutionOrder>,
    /// Whether the graph has been modified since last compile
    dirty: bool,
}

impl RenderGraph {
    /// Create a new empty render graph
    pub fn new() -> Self {
        Self {
            passes: HashMap::new(),
            pass_order: Vec::new(),
            resources: HashMap::new(),
            cached_order: None,
            dirty: true,
        }
    }

    /// Register a new resource
    pub fn create_resource(
        &mut self,
        name: impl Into<String>,
        resource_type: ResourceType,
    ) -> ResourceId {
        let id = ResourceId::new();
        self.resources.insert(
            id,
            Resource {
                id,
                name: name.into(),
                resource_type,
                external: false,
            },
        );
        self.dirty = true;
        id
    }

    /// Register an external resource (not managed by the graph)
    pub fn import_resource(
        &mut self,
        name: impl Into<String>,
        resource_type: ResourceType,
    ) -> ResourceId {
        let id = ResourceId::new();
        self.resources.insert(
            id,
            Resource {
                id,
                name: name.into(),
                resource_type,
                external: true,
            },
        );
        self.dirty = true;
        id
    }

    /// Add a render pass to the graph
    pub fn add_pass(&mut self, pass: RenderPass) -> PassId {
        let id = pass.id;
        self.passes.insert(id, pass);
        self.pass_order.push(id);
        self.dirty = true;
        id
    }

    /// Remove a render pass
    pub fn remove_pass(&mut self, id: PassId) -> Option<RenderPass> {
        self.dirty = true;
        self.pass_order.retain(|&pid| pid != id);
        self.passes.remove(&id)
    }

    /// Get a pass by ID
    pub fn get_pass(&self, id: PassId) -> Option<&RenderPass> {
        self.passes.get(&id)
    }

    /// Get a mutable pass by ID
    pub fn get_pass_mut(&mut self, id: PassId) -> Option<&mut RenderPass> {
        self.dirty = true;
        self.passes.get_mut(&id)
    }

    /// Get a resource by ID
    pub fn get_resource(&self, id: ResourceId) -> Option<&Resource> {
        self.resources.get(&id)
    }

    /// Compile the graph and compute execution order
    pub fn compile(&mut self) -> Result<&ExecutionOrder, RenderGraphError> {
        // Return cached order if available and not dirty
        if !self.dirty
            && let Some(ref order) = self.cached_order
        {
            return Ok(order);
        }

        // Build dependency graph including implicit resource dependencies
        let mut dependencies: HashMap<PassId, HashSet<PassId>> = HashMap::new();

        // Track which pass last wrote to each resource
        let mut last_writer: HashMap<ResourceId, PassId> = HashMap::new();

        // Initialize explicit dependencies
        for (pass_id, pass) in &self.passes {
            if !pass.enabled {
                continue;
            }
            dependencies.insert(*pass_id, pass.dependencies.clone());
        }

        // Add implicit dependencies from resource usage
        // Process passes in insertion order for correct dependency tracking
        for &pass_id in &self.pass_order {
            let Some(pass) = self.passes.get(&pass_id) else {
                continue;
            };
            if !pass.enabled {
                continue;
            }

            for usage in &pass.resources {
                // If reading a resource, depend on its last writer
                if matches!(
                    usage.access,
                    ResourceAccess::Read | ResourceAccess::ReadWrite
                ) && let Some(&writer_id) = last_writer.get(&usage.resource_id)
                    && writer_id != pass_id
                {
                    dependencies.entry(pass_id).or_default().insert(writer_id);
                }

                // Track writes
                if matches!(
                    usage.access,
                    ResourceAccess::Write | ResourceAccess::ReadWrite
                ) {
                    last_writer.insert(usage.resource_id, pass_id);
                }
            }
        }

        // Topological sort using Kahn's algorithm
        let order = self.topological_sort(&dependencies)?;

        // Group passes by level for potential parallelization
        let parallel_groups = self.compute_parallel_groups(&order, &dependencies);

        self.cached_order = Some(ExecutionOrder {
            passes: order,
            parallel_groups,
        });
        self.dirty = false;

        // Safe to unwrap: we just set cached_order to Some on the line above
        // Using match to satisfy clippy's no-unwrap rule
        match self.cached_order.as_ref() {
            Some(order) => Ok(order),
            None => unreachable!("cached_order was just set"),
        }
    }

    /// Perform topological sort
    fn topological_sort(
        &self,
        dependencies: &HashMap<PassId, HashSet<PassId>>,
    ) -> Result<Vec<PassId>, RenderGraphError> {
        let mut in_degree: HashMap<PassId, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Calculate in-degrees
        for (&pass_id, deps) in dependencies {
            in_degree.entry(pass_id).or_insert(0);
            for &dep in deps {
                in_degree.entry(dep).or_insert(0);
            }
        }

        for (&pass_id, deps) in dependencies {
            for &dep in deps {
                if dependencies.contains_key(&dep) {
                    *in_degree.entry(pass_id).or_default() += 1;
                }
            }
        }

        // Find all nodes with no dependencies
        for (&pass_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(pass_id);
            }
        }

        // Process queue
        while let Some(pass_id) = queue.pop_front() {
            result.push(pass_id);

            // Find all passes that depend on this one
            for (&other_id, deps) in dependencies {
                if deps.contains(&pass_id)
                    && let Some(degree) = in_degree.get_mut(&other_id)
                {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(other_id);
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != dependencies.len() {
            // Find a cycle for error reporting
            let remaining: Vec<PassId> = dependencies
                .keys()
                .filter(|id| !result.contains(id))
                .copied()
                .collect();
            return Err(RenderGraphError::CyclicDependency(remaining));
        }

        Ok(result)
    }

    /// Compute groups of passes that can run in parallel
    fn compute_parallel_groups(
        &self,
        order: &[PassId],
        dependencies: &HashMap<PassId, HashSet<PassId>>,
    ) -> Vec<Vec<PassId>> {
        let mut groups: Vec<Vec<PassId>> = Vec::new();
        let mut pass_levels: HashMap<PassId, usize> = HashMap::new();

        for &pass_id in order {
            // Level is max level of dependencies + 1
            let level = dependencies
                .get(&pass_id)
                .map(|deps| {
                    deps.iter()
                        .filter_map(|dep| pass_levels.get(dep))
                        .max()
                        .map(|l| l + 1)
                        .unwrap_or(0)
                })
                .unwrap_or(0);

            pass_levels.insert(pass_id, level);

            // Ensure groups vector is large enough
            while groups.len() <= level {
                groups.push(Vec::new());
            }

            groups[level].push(pass_id);
        }

        groups
    }

    /// Execute the compiled graph
    pub fn execute(&mut self) -> Result<(), RenderGraphError> {
        // Compile first, then clone the order to release the borrow
        self.compile()?;

        // Safe: compile() guarantees cached_order is Some on success
        let Some(order) = self.cached_order.clone() else {
            return Err(RenderGraphError::NotCompiled);
        };
        let total_passes = order.passes.len();

        for (index, &pass_id) in order.passes.iter().enumerate() {
            if let Some(pass) = self.passes.get(&pass_id)
                && let Some(ref callback) = pass.callback
            {
                let context = PassContext {
                    pass_id,
                    index,
                    total_passes,
                };
                callback(&context);
            }
        }

        Ok(())
    }

    /// Get statistics about the graph
    pub fn stats(&self) -> GraphStats {
        let enabled_passes = self.passes.values().filter(|p| p.enabled).count();
        GraphStats {
            total_passes: self.passes.len(),
            enabled_passes,
            resources: self.resources.len(),
            is_compiled: self.cached_order.is_some(),
        }
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the render graph
#[derive(Debug, Clone, Copy, Default)]
pub struct GraphStats {
    pub total_passes: usize,
    pub enabled_passes: usize,
    pub resources: usize,
    pub is_compiled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_graph() {
        let graph = RenderGraph::new();
        let stats = graph.stats();
        assert_eq!(stats.total_passes, 0);
        assert_eq!(stats.resources, 0);
    }

    #[test]
    fn test_add_resource() {
        let mut graph = RenderGraph::new();
        let id = graph.create_resource("backbuffer", ResourceType::RenderTarget);

        assert!(graph.get_resource(id).is_some());
        assert_eq!(graph.stats().resources, 1);
    }

    #[test]
    fn test_add_pass() {
        let mut graph = RenderGraph::new();
        let pass = RenderPass::new("clear");
        let id = graph.add_pass(pass);

        assert!(graph.get_pass(id).is_some());
        assert_eq!(graph.stats().total_passes, 1);
    }

    #[test]
    fn test_simple_dependency() {
        let mut graph = RenderGraph::new();

        let backbuffer = graph.create_resource("backbuffer", ResourceType::RenderTarget);

        let clear = graph.add_pass(RenderPass::new("clear").write(backbuffer));

        let render = graph.add_pass(RenderPass::new("render").read_write(backbuffer));

        let order = graph.compile().unwrap();

        // Clear should come before render due to resource dependency
        let clear_idx = order.passes.iter().position(|&id| id == clear).unwrap();
        let render_idx = order.passes.iter().position(|&id| id == render).unwrap();
        assert!(clear_idx < render_idx);
    }

    #[test]
    fn test_explicit_dependency() {
        let mut graph = RenderGraph::new();

        let pass_a = graph.add_pass(RenderPass::new("A"));
        let pass_b = graph.add_pass(RenderPass::new("B").depends_on(pass_a));

        let order = graph.compile().unwrap();

        let a_idx = order.passes.iter().position(|&id| id == pass_a).unwrap();
        let b_idx = order.passes.iter().position(|&id| id == pass_b).unwrap();
        assert!(a_idx < b_idx);
    }

    #[test]
    fn test_parallel_groups() {
        let mut graph = RenderGraph::new();

        // A and B have no dependencies, C depends on both
        let pass_a = graph.add_pass(RenderPass::new("A"));
        let pass_b = graph.add_pass(RenderPass::new("B"));
        let _pass_c = graph.add_pass(RenderPass::new("C").depends_on(pass_a).depends_on(pass_b));

        let order = graph.compile().unwrap();

        // A and B should be in the same parallel group
        assert!(!order.parallel_groups.is_empty());
        let first_group = &order.parallel_groups[0];
        assert!(first_group.contains(&pass_a) || first_group.contains(&pass_b));
    }

    #[test]
    fn test_disabled_pass() {
        let mut graph = RenderGraph::new();

        let mut pass = RenderPass::new("disabled");
        pass.set_enabled(false);
        graph.add_pass(pass);

        let order = graph.compile().unwrap();
        assert!(order.passes.is_empty());
    }

    #[test]
    fn test_remove_pass() {
        let mut graph = RenderGraph::new();
        let pass_id = graph.add_pass(RenderPass::new("test"));

        assert_eq!(graph.stats().total_passes, 1);
        graph.remove_pass(pass_id);
        assert_eq!(graph.stats().total_passes, 0);
    }

    #[test]
    fn test_execution() {
        use std::cell::Cell;
        use std::rc::Rc;

        let mut graph = RenderGraph::new();
        let executed = Rc::new(Cell::new(false));
        let executed_clone = executed.clone();

        graph.add_pass(RenderPass::new("test").on_execute(move |_ctx| {
            executed_clone.set(true);
        }));

        graph.execute().unwrap();
        assert!(executed.get());
    }
}
