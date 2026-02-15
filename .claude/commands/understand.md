# understand - Quick Codebase Overview

## Overview

Provides a **concise, high-level overview** of specific files, structs, or features. Perfect for quick reference and getting oriented.

For comprehensive analysis with detailed examples and dependency graphs, use `understand-deep` instead.

## Usage

```bash
understand <target>
```

**target** can be any of:
- **File path**: `app/core/src/scenario.rs`
- **Module name**: `runtime`, `render`, `character`
- **Struct/Enum name**: `Scenario`, `CharacterAnimation`, `GameState`
- **Feature name**: `save-system`, `animation`, `text-rendering`

## Output Format

Generates concise documentation in Markdown format (1-2 pages) containing:

### 1. Quick Summary
- **Purpose**: One-liner description
- **Location**: File path
- **Phase**: Roadmap phase

### 2. Type Definition
- **Struct/Enum signature**: Just the type and field names (no doc comments)
- **Derived traits**: List only

### 3. Key Methods (Top 5-10)
- **Constructor**: Creation methods
- **Core methods**: Most commonly used 3-5 methods
- **Important helpers**: 2-3 utility methods
- Signature only, no implementation details

### 4. Dependencies (Simple List)
- **Direct imports**: 3-5 key dependencies
- **Used by**: 3-5 major usage locations
- Simple bullet points only (no diagrams)

### 5. Quick Example
- **One real usage example** from the codebase (5-10 lines)

### 6. Related Files
- **See also**: 2-3 related files to explore
- **Deep dive**: Link to `understand-deep` for full details

## Processing Steps

1. **Identify target** (same as understand-deep)
   - Quick Glob/Grep search
   - Confirm if multiple candidates

2. **Extract essentials**
   - Read target file
   - Extract type definition (structure only)
   - Identify 5-10 most important methods
   - Skip detailed doc comments

3. **Minimal dependency analysis**
   - List 3-5 direct imports
   - Find 3-5 main usage locations (not exhaustive)
   - Skip comprehensive search

4. **Single usage example**
   - Find ONE clear, simple usage example
   - Prefer test code or simple initialization

5. **Output concise Markdown**
   - Keep to 1-2 pages maximum
   - No diagrams (keep it simple)
   - Focus on "what" not "how"

6. **Save to file**
   - Create `.claude/tmp/` directory if needed
   - Save to `.claude/tmp/understand_<target>.md`
   - Display file path after completion

## Output File

After generating the documentation, save it to:
```
.claude/tmp/understand_<sanitized_target>.md
```

**Examples**:
- `understand Scenario` → `.claude/tmp/understand_scenario.md`
- `understand runtime` → `.claude/tmp/understand_runtime.md`
- `understand state_machine` → `.claude/tmp/understand_state_machine.md`

## Example Output

```markdown
# `Scenario` - Quick Overview

## Summary

**Purpose**: Core data structure for visual novel scenarios
**Location**: `app/core/src/scenario.rs`
**Phase**: Phase 0 (Core Types)

---

## Type Definition

```rust
pub struct Scenario {
    pub metadata: ScenarioMetadata,
    pub scenes: HashMap<SceneId, Scene>,
    pub characters: HashMap<CharacterId, CharacterDef>,
}
```

**Derived**: `Debug, Clone, Serialize, Deserialize`

---

## Key Methods

**Constructor**:
```rust
pub fn new(metadata: ScenarioMetadata, initial_scene: SceneId) -> Self
```

**Scene Management**:
```rust
pub fn add_scene(&mut self, id: SceneId, scene: Scene)
pub fn get_scene(&self, id: &SceneId) -> Option<&Scene>
```

**Character Management**:
```rust
pub fn add_character(&mut self, id: CharacterId, def: CharacterDef)
pub fn get_character(&self, id: &CharacterId) -> Option<&CharacterDef>
```

---

## Dependencies

**Imports**:
- `types::{SceneId, CharacterId}`
- `config::ScenarioMetadata`
- `serde::{Serialize, Deserialize}`

**Used by**:
- `engine/runtime/executor.rs` - Runtime execution
- `engine/save/data.rs` - Save/load system
- `tools/scenario_validator.rs` - Validation

---

## Quick Example

```rust
let metadata = ScenarioMetadata::new("chapter_01", "Chapter 1");
let mut scenario = Scenario::new(metadata, "scene_01");

let scene = Scene::new("scene_01", "Opening");
scenario.add_scene("scene_01", scene);
```

---

## Related Files

**See also**:
- `scenario/scene.rs` - Scene implementation
- `runtime/executor.rs` - Runtime usage

**For detailed analysis**: Run `understand-deep Scenario`

---

**Generated**: 2026-02-15
**Command**: `understand scenario`
```

## Important Notes

- **Brevity First**: Keep output to 1-2 pages maximum
- **Skip Details**: No comprehensive method lists, no full doc comments
- **Quick Reference**: Optimized for fast lookup, not learning
- **Direct Users**: When more detail is needed, suggest `understand-deep`
- **Save Output**: Always save to `.claude/tmp/understand_*.md`

## Technical Guidelines

### What to Include
- Type signatures (fields, enums)
- Top 5-10 methods only
- 3-5 key dependencies
- 1 simple usage example
- 2-3 related files

### What to Skip
- Detailed doc comments (use first sentence only)
- All methods (just the important ones)
- Exhaustive dependency search
- Multiple code examples
- Diagrams (ASCII or otherwise)
- Test code details
- Implementation explanations

### Selection Criteria for Methods

**Include**:
1. Constructor (`new`, `default`, `from_*`)
2. Most commonly called methods (check usage count)
3. Core API methods (public, non-helper)
4. Methods mentioned in doc comments
5. Methods used in examples

**Skip**:
- Internal helpers
- Rarely used utilities
- Simple getters/setters
- Private methods

---

**Purpose**: Provide quick orientation for developers who just need to know "what is this" without deep diving into "how it works".
