//! Typewriter effect for dialogue

use std::time::Duration;

/// Typewriter effect state
#[derive(Debug, Clone)]
pub struct TypewriterEffect {
    /// Full text to display
    full_text: String,
    /// Current visible character count
    visible_chars: usize,
    /// Time elapsed since last character
    elapsed: Duration,
    /// Time between characters
    char_delay: Duration,
    /// Whether the effect is complete
    complete: bool,
}

impl TypewriterEffect {
    /// Create a new typewriter effect
    pub fn new(text: String, chars_per_second: f32) -> Self {
        let millis_per_char = (1000.0 / chars_per_second) as u64;
        let char_delay = Duration::from_millis(millis_per_char);
        let complete = text.is_empty();
        Self {
            full_text: text,
            visible_chars: 0,
            elapsed: Duration::ZERO,
            char_delay,
            complete,
        }
    }

    /// Update the typewriter effect
    pub fn update(&mut self, delta: Duration) {
        if self.complete {
            return;
        }

        self.elapsed = self.elapsed.saturating_add(delta);

        while self.elapsed >= self.char_delay && !self.complete {
            self.elapsed = self.elapsed.saturating_sub(self.char_delay);
            self.visible_chars = self.visible_chars.saturating_add(1);

            if self.visible_chars >= self.full_text.chars().count() {
                self.complete = true;
            }
        }
    }

    /// Get the currently visible text
    pub fn visible_text(&self) -> String {
        if self.complete {
            self.full_text.clone()
        } else {
            self.full_text
                .chars()
                .take(self.visible_chars)
                .collect::<String>()
        }
    }

    /// Skip to the end
    pub fn skip(&mut self) {
        self.visible_chars = self.full_text.chars().count();
        self.complete = true;
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Get the full text
    pub fn full_text(&self) -> &str {
        &self.full_text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typewriter() {
        let mut effect = TypewriterEffect::new("Hello".to_string(), 10.0);

        assert!(!effect.is_complete());
        assert_eq!(effect.visible_chars, 0);

        // Update with 100ms - should display 1 character at 10 chars/sec
        effect.update(Duration::from_millis(100));
        assert_eq!(effect.visible_chars, 1);

        // Update with another 400ms - should display 4 more characters
        effect.update(Duration::from_millis(400));
        assert_eq!(effect.visible_chars, 5);
        assert!(effect.is_complete()); // "Hello" has 5 characters

        // Skip should work even when already complete
        effect.skip();
        assert!(effect.is_complete());
        assert_eq!(effect.visible_text(), "Hello".to_string());
    }

    #[test]
    fn test_typewriter_slow_speed() {
        let mut effect = TypewriterEffect::new("AB".to_string(), 2.0); // 2 chars/sec = 500ms/char

        effect.update(Duration::from_millis(499));
        assert_eq!(effect.visible_chars, 0);

        effect.update(Duration::from_millis(1));
        assert_eq!(effect.visible_chars, 1);

        effect.update(Duration::from_millis(500));
        assert_eq!(effect.visible_chars, 2);
        assert!(effect.is_complete());
    }

    #[test]
    fn test_typewriter_fast_speed() {
        let mut effect = TypewriterEffect::new("Test".to_string(), 100.0); // 100 chars/sec

        effect.update(Duration::from_millis(40));
        assert_eq!(effect.visible_chars, 4);
        assert!(effect.is_complete());
    }

    #[test]
    fn test_typewriter_skip() {
        let mut effect = TypewriterEffect::new("Long text here".to_string(), 10.0);

        assert!(!effect.is_complete());

        effect.skip();

        assert!(effect.is_complete());
        assert_eq!(effect.visible_text(), "Long text here".to_string());
    }

    #[test]
    fn test_typewriter_empty_string() {
        let effect = TypewriterEffect::new(String::new(), 10.0);

        assert!(effect.is_complete()); // Empty string is immediately complete
        assert_eq!(effect.visible_text(), "".to_string());
    }

    #[test]
    fn test_typewriter_unicode() {
        let mut effect = TypewriterEffect::new("こんにちは".to_string(), 10.0);

        effect.update(Duration::from_millis(100));
        assert_eq!(effect.visible_chars, 1);

        effect.update(Duration::from_millis(400));
        assert_eq!(effect.visible_chars, 5);
        assert!(effect.is_complete());
    }

    #[test]
    fn test_typewriter_full_text() {
        let text = "Hello, World!".to_string();
        let effect = TypewriterEffect::new(text.clone(), 10.0);

        assert_eq!(effect.full_text(), &text);
    }

    #[test]
    fn test_typewriter_partial_update() {
        let mut effect = TypewriterEffect::new("12345".to_string(), 10.0); // 10 chars/sec = 100ms/char

        effect.update(Duration::from_millis(250));
        assert_eq!(effect.visible_chars, 2); // 250ms / 100ms = 2.5 = 2 chars

        effect.update(Duration::from_millis(250));
        assert_eq!(effect.visible_chars, 5); // Total 500ms / 100ms = 5 chars = complete
        assert!(effect.is_complete());
    }

    #[test]
    fn test_typewriter_complete_no_more_updates() {
        let mut effect = TypewriterEffect::new("Hi".to_string(), 10.0);

        effect.skip();
        assert!(effect.is_complete());

        let chars_before = effect.visible_chars;
        effect.update(Duration::from_millis(1000));
        assert_eq!(effect.visible_chars, chars_before); // Should not change
    }
}
