use super::*;

impl ScenarioRuntime {
    /// Get the currently displayed characters
    pub fn displayed_characters(&self) -> &HashMap<String, DisplayedCharacter> {
        &self.displayed_characters
    }

    /// Check if displayed characters have changed since last check
    ///
    /// This method consumes the dirty flag (one-shot read).
    /// Returns true if characters were added, removed, moved, or had sprite changes.
    pub fn displayed_characters_changed(&mut self) -> bool {
        if self.displayed_characters_dirty {
            self.displayed_characters_dirty = false;
            true
        } else {
            false
        }
    }
}
