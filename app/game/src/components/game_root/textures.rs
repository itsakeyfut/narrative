//! Texture loading and management for GameRootElement

use super::element::GameRootElement;
use narrative_engine::asset::TextureHandle;

impl GameRootElement {
    /// Check if background has changed and schedule loading if necessary
    ///
    /// Returns: true if background changed (for children_dirty flag)
    pub(super) fn update_background_if_changed(&mut self) -> bool {
        let Some(runtime) = &self.scenario_runtime else {
            return false;
        };

        let runtime_background = runtime.current_background();

        // Check if background has changed
        let background_changed = match (&self.displayed_background, runtime_background) {
            (None, None) => false,
            (Some(_), None) => true,
            (None, Some(_)) => true,
            (Some(old), Some(new)) => old != new,
        };

        if !background_changed {
            return false;
        }

        tracing::debug!(
            "Background changed: {:?} -> {:?}",
            self.displayed_background.as_ref().map(|bg| bg.path()),
            runtime_background.map(|bg| bg.path())
        );

        // Save current background as previous for crossfade transitions
        self.previous_background_texture_id = self.current_background_texture_id;

        // Update displayed background
        self.displayed_background = runtime_background.cloned();

        tracing::debug!(
            "Cache size: {}, new_bg: {:?}",
            self.background_texture_cache.len(),
            runtime_background.map(|bg| bg.path())
        );

        if let Some(new_bg) = runtime_background {
            tracing::debug!(
                "Checking cache for: {}, current_texture_id: {:?}, displayed_background: {:?}",
                new_bg.path(),
                self.current_background_texture_id,
                self.displayed_background.as_ref().map(|bg| bg.path())
            );

            // Special case: First background change (None -> Some)
            // The initial background was loaded by Window, so register it in cache
            if self.displayed_background.is_none()
                && self.background_texture_id.is_some()
                && let Some(initial_bg_id) = self.background_texture_id
            {
                tracing::debug!(
                    "First background change - registering initial background in cache: {} (id: {})",
                    new_bg.path(),
                    initial_bg_id
                );
                self.background_texture_cache
                    .insert(new_bg.clone(), initial_bg_id);
                // Background is already displayed, no need to load
            }

            // Check cache
            if let Some(&cached_id) = self.background_texture_cache.get(new_bg) {
                tracing::debug!(
                    "Using cached background: {} (id: {})",
                    new_bg.path(),
                    cached_id
                );
                self.current_background_texture_id = Some(cached_id);
            } else {
                // Not in cache - schedule load for next frame
                tracing::debug!("Scheduling background load: {}", new_bg.path());
                self.pending_background = Some(new_bg.clone());
                // Keep current texture until new one is loaded (shows old background during load)
                // Don't set to None - this prevents flickering to fallback color
            }
        } else {
            // HideBackground command
            self.current_background_texture_id = None;
            self.pending_background = None;
        }

        true
    }

    /// Update CG if changed
    ///
    /// Returns: true if CG changed (for children_dirty flag)
    pub(super) fn update_cg_if_changed(&mut self) -> bool {
        let Some(runtime) = &self.scenario_runtime else {
            return false;
        };

        let runtime_cg = runtime.current_cg();

        // Check if CG has changed
        let cg_changed = match (&self.displayed_cg, runtime_cg) {
            (None, None) => false,
            (Some(_), None) => true,
            (None, Some(_)) => true,
            (Some(old), Some(new)) => old != new,
        };

        if !cg_changed {
            return false;
        }

        tracing::debug!(
            "CG changed: {:?} -> {:?}",
            self.displayed_cg.as_ref().map(|cg| cg.path()),
            runtime_cg.map(|cg| cg.path())
        );

        // Save current CG as previous for crossfade transitions
        self.previous_cg_texture_id = self.current_cg_texture_id;
        self.previous_cg_texture_size = self.current_cg_texture_size;

        // Update displayed CG
        self.displayed_cg = runtime_cg.cloned();

        if let Some(new_cg) = runtime_cg {
            // Check cache
            if let Some(&(cached_id, cached_size)) = self.cg_texture_cache.get(new_cg) {
                tracing::debug!("Using cached CG: {} (id: {})", new_cg.path(), cached_id);
                self.current_cg_texture_id = Some(cached_id);
                self.current_cg_texture_size = Some(cached_size);
            } else {
                // Not in cache - schedule load for next frame
                tracing::debug!("Scheduling CG load: {}", new_cg.path());
                self.pending_cg = Some(new_cg.clone());
                // Keep current texture until new one is loaded
            }
        } else {
            // HideCG command
            self.current_cg_texture_id = None;
            self.current_cg_texture_size = None;
            self.pending_cg = None;
        }

        true
    }

    /// Load pending background and character textures
    ///
    /// This is called by the Window/Element system when textures need to be loaded.
    /// Returns true if any textures were loaded (triggers redraw).
    pub fn load_pending_background_texture(
        &mut self,
        renderer: &mut narrative_gui::framework::renderer::Renderer,
    ) -> bool {
        let mut needs_redraw = false;

        // Load pending background texture
        if let Some(pending_bg) = self.pending_background.clone() {
            tracing::debug!(
                "load_pending_background_texture called for: {}",
                pending_bg.path()
            );

            match renderer.load_texture_from_path(std::path::Path::new(pending_bg.path())) {
                Ok(texture_id) => {
                    tracing::debug!(
                        "Loaded background texture: {} (id: {})",
                        pending_bg.path(),
                        texture_id
                    );
                    self.background_texture_cache.insert(pending_bg, texture_id);
                    self.current_background_texture_id = Some(texture_id);
                    self.pending_background = None;
                    needs_redraw = true;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to load background '{}': {}. Using fallback color.",
                        pending_bg.path(),
                        e
                    );
                    self.pending_background = None;
                }
            }
        }

        // Load pending CG texture
        if let Some(pending_cg) = self.pending_cg.clone() {
            tracing::debug!("Loading pending CG texture: {}", pending_cg.path());

            match renderer.load_texture_from_path(std::path::Path::new(pending_cg.path())) {
                Ok(texture_id) => {
                    tracing::debug!(
                        "Loaded CG texture: {} (id: {})",
                        pending_cg.path(),
                        texture_id
                    );

                    // Get texture size for aspect ratio calculation
                    let texture_size = match renderer.get_texture_size(texture_id) {
                        Some(size) => size,
                        None => {
                            tracing::warn!(
                                "Failed to get texture size for CG '{}' (id: {}). Using fallback HD resolution (1280x720).",
                                pending_cg.path(),
                                texture_id
                            );
                            (1280, 720)
                        }
                    };
                    self.cg_texture_cache
                        .insert(pending_cg, (texture_id, texture_size));
                    self.current_cg_texture_id = Some(texture_id);
                    self.current_cg_texture_size = Some(texture_size);
                    self.pending_cg = None;
                    needs_redraw = true;
                }
                Err(e) => {
                    tracing::error!("Failed to load CG '{}': {}", pending_cg.path(), e);
                    self.pending_cg = None;
                }
            }
        }

        // Load pending character textures
        while let Some((character_id, sprite_ref)) = self.pending_character_textures.pop() {
            match renderer.load_texture_from_path(std::path::Path::new(sprite_ref.path())) {
                Ok(texture_id) => {
                    tracing::info!(
                        "Loaded character texture: character='{}', sprite='{}', texture_id={}",
                        character_id,
                        sprite_ref.0,
                        texture_id
                    );
                    self.character_texture_cache
                        .insert(sprite_ref, TextureHandle::new(texture_id));
                    needs_redraw = true;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to load character texture: character='{}', sprite='{}', error: {}",
                        character_id,
                        sprite_ref.0,
                        e
                    );
                }
            }
        }

        // Load CG thumbnails if in CG Gallery state
        if matches!(
            &self.app_state,
            narrative_engine::runtime::AppState::InGame(
                narrative_engine::runtime::InGameState::CgGallery(_)
            )
        ) {
            // Check if we need to load thumbnails
            let unlock_data = match self.unlock_data.lock() {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to lock unlock_data: {}", e);
                    return needs_redraw;
                }
            };

            let sorted_cgs = self.cg_registry.get_all_sorted();
            let unlocked_count = unlock_data.unlocked_cg_count();
            let cached_count = self.cg_thumbnail_cache.len();

            tracing::debug!(
                "CG Gallery: {} total CGs, {} unlocked, {} cached thumbnails",
                sorted_cgs.len(),
                unlocked_count,
                cached_count
            );

            let mut loaded_any = false;

            for cg in sorted_cgs {
                // Only load thumbnails for unlocked CGs that aren't cached
                if unlock_data.is_cg_unlocked(&cg.id)
                    && !self.cg_thumbnail_cache.contains_key(&cg.id)
                {
                    let mut texture_id_opt = None;

                    // Priority 1: Try thumbnail path from config (if specified)
                    if let Some(thumb_path) = &cg.thumbnail_path {
                        match renderer.load_texture_from_path(std::path::Path::new(thumb_path)) {
                            Ok(texture_id) => {
                                tracing::debug!(
                                    "Loaded config thumbnail: {} -> {} (id: {})",
                                    cg.id,
                                    thumb_path,
                                    texture_id
                                );
                                texture_id_opt = Some(texture_id);
                            }
                            Err(_) => {
                                // Continue to fallback
                            }
                        }
                    }

                    // Priority 2: Fallback to full-size CG
                    if texture_id_opt.is_none() {
                        match renderer.load_texture_from_path(std::path::Path::new(&cg.asset_path))
                        {
                            Ok(texture_id) => {
                                tracing::debug!(
                                    "Loaded full-size CG as thumbnail: {} -> {} (id: {})",
                                    cg.id,
                                    cg.asset_path,
                                    texture_id
                                );
                                texture_id_opt = Some(texture_id);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load CG '{}': {}", cg.id, e);
                            }
                        }
                    }

                    // Cache the loaded texture
                    if let Some(texture_id) = texture_id_opt {
                        self.cg_thumbnail_cache.insert(cg.id.clone(), texture_id);
                        loaded_any = true;
                    }
                }
            }

            if loaded_any {
                needs_redraw = true;
                tracing::debug!("children_dirty set at line {}", line!());
                self.children_dirty = true; // Rebuild CgGalleryElement with new thumbnails
            }
        }

        needs_redraw
    }
}
