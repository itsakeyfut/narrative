//! CG (Event Graphics) metadata and registry
//!
//! This module provides types for defining CG metadata and managing
//! a registry of all available CGs in the game.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Unique identifier for a CG
pub type CgId = String;

/// CG (Event Graphics) metadata
///
/// Defines information about a CG including its asset paths,
/// variations, and display metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CgMetadata {
    /// Unique CG identifier
    pub id: CgId,

    /// Display title for the CG (shown in gallery)
    pub title: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Main asset path for the CG
    pub asset_path: String,

    /// Thumbnail asset path (for gallery grid)
    /// If None, the main asset will be used scaled down
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_path: Option<String>,

    /// Variations of this CG (e.g., different expressions, times of day)
    #[serde(default)]
    pub variations: Vec<CgVariation>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Sort order for gallery display (lower = earlier)
    #[serde(default)]
    pub sort_order: u32,
}

impl CgMetadata {
    /// Create a new CG metadata entry
    pub fn new(
        id: impl Into<CgId>,
        title: impl Into<String>,
        asset_path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            asset_path: asset_path.into(),
            thumbnail_path: None,
            variations: Vec::new(),
            tags: Vec::new(),
            sort_order: 0,
        }
    }

    /// Set the thumbnail path
    pub fn with_thumbnail(mut self, thumbnail_path: impl Into<String>) -> Self {
        self.thumbnail_path = Some(thumbnail_path.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a variation
    pub fn with_variation(mut self, variation: CgVariation) -> Self {
        self.variations.push(variation);
        self
    }

    /// Add multiple variations
    pub fn with_variations(mut self, variations: Vec<CgVariation>) -> Self {
        self.variations = variations;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set the sort order
    pub fn with_sort_order(mut self, order: u32) -> Self {
        self.sort_order = order;
        self
    }

    /// Get the thumbnail path or fallback to main asset
    pub fn get_thumbnail_path(&self) -> &str {
        self.thumbnail_path.as_deref().unwrap_or(&self.asset_path)
    }

    /// Get the total number of images (main + variations)
    pub fn total_image_count(&self) -> usize {
        1 + self.variations.len()
    }
}

/// A variation of a CG
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CgVariation {
    /// Variation identifier (e.g., "happy", "night", "diff1")
    pub id: String,

    /// Display name for this variation
    pub name: String,

    /// Asset path for this variation
    pub asset_path: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CgVariation {
    /// Create a new CG variation
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        asset_path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            asset_path: asset_path.into(),
            description: None,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Registry of all available CGs in the game
#[derive(Debug, Clone, Default)]
pub struct CgRegistry {
    /// Map of CG ID to CG metadata
    cgs: HashMap<CgId, CgMetadata>,

    /// Sorted list of CG IDs (for ordered iteration)
    sorted_ids: Vec<CgId>,
}

impl CgRegistry {
    /// Create a new empty CG registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a CG
    pub fn register(&mut self, cg: CgMetadata) {
        let id = cg.id.clone();
        self.cgs.insert(id.clone(), cg);
        self.update_sorted_ids();
    }

    /// Register multiple CGs
    pub fn register_many(&mut self, cgs: Vec<CgMetadata>) {
        for cg in cgs {
            self.cgs.insert(cg.id.clone(), cg);
        }
        self.update_sorted_ids();
    }

    /// Get a CG by ID
    pub fn get(&self, id: &str) -> Option<&CgMetadata> {
        self.cgs.get(id)
    }

    /// Get all CGs in sorted order
    pub fn get_all_sorted(&self) -> Vec<&CgMetadata> {
        self.sorted_ids
            .iter()
            .filter_map(|id| self.cgs.get(id))
            .collect()
    }

    /// Get the total number of CGs
    pub fn total_count(&self) -> usize {
        self.cgs.len()
    }

    /// Check if a CG exists
    pub fn contains(&self, id: &str) -> bool {
        self.cgs.contains_key(id)
    }

    /// Update the sorted ID list based on sort_order
    fn update_sorted_ids(&mut self) {
        let mut ids: Vec<(String, u32)> = self
            .cgs
            .iter()
            .map(|(id, cg)| (id.clone(), cg.sort_order))
            .collect();

        ids.sort_by_key(|(_, order)| *order);
        self.sorted_ids = ids.into_iter().map(|(id, _)| id).collect();
    }

    /// Extract CG ID from asset path
    ///
    /// Converts "assets/cg/event_01.png" -> "event_01"
    /// This is a helper for automatically unlocking CGs when they're displayed
    pub fn extract_cg_id_from_path(asset_path: &str) -> Option<String> {
        // Use Path to properly handle both Unix and Windows path separators
        Path::new(asset_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cg_metadata_new() {
        let cg = CgMetadata::new("cg_01", "First Event", "assets/cg/event_01.png");
        assert_eq!(cg.id, "cg_01");
        assert_eq!(cg.title, "First Event");
        assert_eq!(cg.asset_path, "assets/cg/event_01.png");
        assert_eq!(cg.variations.len(), 0);
    }

    #[test]
    fn test_cg_metadata_builder() {
        let variation = CgVariation::new("happy", "Happy Version", "assets/cg/event_01_happy.png");

        let cg = CgMetadata::new("cg_01", "First Event", "assets/cg/event_01.png")
            .with_thumbnail("assets/cg/thumbs/event_01.png")
            .with_description("An important event")
            .with_variation(variation)
            .with_tag("chapter1")
            .with_sort_order(10);

        assert_eq!(
            cg.thumbnail_path,
            Some("assets/cg/thumbs/event_01.png".to_string())
        );
        assert_eq!(cg.description, Some("An important event".to_string()));
        assert_eq!(cg.variations.len(), 1);
        assert_eq!(cg.tags.len(), 1);
        assert_eq!(cg.sort_order, 10);
    }

    #[test]
    fn test_cg_get_thumbnail_path() {
        let cg1 = CgMetadata::new("cg_01", "Event 1", "assets/cg/event_01.png");
        assert_eq!(cg1.get_thumbnail_path(), "assets/cg/event_01.png");

        let cg2 = CgMetadata::new("cg_02", "Event 2", "assets/cg/event_02.png")
            .with_thumbnail("assets/cg/thumbs/event_02.png");
        assert_eq!(cg2.get_thumbnail_path(), "assets/cg/thumbs/event_02.png");
    }

    #[test]
    fn test_cg_total_image_count() {
        let mut cg = CgMetadata::new("cg_01", "Event 1", "assets/cg/event_01.png");
        assert_eq!(cg.total_image_count(), 1);

        cg.variations
            .push(CgVariation::new("var1", "Variation 1", "path1"));
        cg.variations
            .push(CgVariation::new("var2", "Variation 2", "path2"));
        assert_eq!(cg.total_image_count(), 3);
    }

    #[test]
    fn test_cg_variation_new() {
        let var = CgVariation::new("happy", "Happy Version", "assets/cg/event_happy.png")
            .with_description("A happy ending");

        assert_eq!(var.id, "happy");
        assert_eq!(var.name, "Happy Version");
        assert_eq!(var.asset_path, "assets/cg/event_happy.png");
        assert_eq!(var.description, Some("A happy ending".to_string()));
    }

    #[test]
    fn test_cg_registry_new() {
        let registry = CgRegistry::new();
        assert_eq!(registry.total_count(), 0);
    }

    #[test]
    fn test_cg_registry_register() {
        let mut registry = CgRegistry::new();

        let cg1 = CgMetadata::new("cg_01", "Event 1", "path1");
        let cg2 = CgMetadata::new("cg_02", "Event 2", "path2");

        registry.register(cg1);
        registry.register(cg2);

        assert_eq!(registry.total_count(), 2);
        assert!(registry.contains("cg_01"));
        assert!(registry.contains("cg_02"));
        assert!(!registry.contains("cg_03"));
    }

    #[test]
    fn test_cg_registry_get() {
        let mut registry = CgRegistry::new();
        let cg = CgMetadata::new("cg_01", "Event 1", "path1");
        registry.register(cg);

        let retrieved = registry.get("cg_01");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Event 1");

        let not_found = registry.get("cg_99");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_cg_registry_sorted() {
        let mut registry = CgRegistry::new();

        let cg1 = CgMetadata::new("cg_01", "Event 1", "path1").with_sort_order(20);
        let cg2 = CgMetadata::new("cg_02", "Event 2", "path2").with_sort_order(10);
        let cg3 = CgMetadata::new("cg_03", "Event 3", "path3").with_sort_order(30);

        registry.register(cg1);
        registry.register(cg2);
        registry.register(cg3);

        let sorted = registry.get_all_sorted();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].id, "cg_02"); // order 10
        assert_eq!(sorted[1].id, "cg_01"); // order 20
        assert_eq!(sorted[2].id, "cg_03"); // order 30
    }

    #[test]
    fn test_extract_cg_id_from_path() {
        assert_eq!(
            CgRegistry::extract_cg_id_from_path("assets/cg/event_01.png"),
            Some("event_01".to_string())
        );

        assert_eq!(
            CgRegistry::extract_cg_id_from_path("assets\\cg\\event_02.png"),
            Some("event_02".to_string())
        );

        assert_eq!(
            CgRegistry::extract_cg_id_from_path("event_03.jpg"),
            Some("event_03".to_string())
        );

        assert_eq!(CgRegistry::extract_cg_id_from_path(""), None);
    }
}
