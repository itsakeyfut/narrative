# understand-deep - Comprehensive Codebase Analysis

## Overview

Provides **comprehensive and detailed** analysis of specific files, structs, or features with extensive documentation, code examples, and dependency graphs. Use this for deep dives into complex implementations.

For quick overview, use the `understand` command instead.

## Usage

```bash
understand-deep <target>
```

**target** can be any of:
- **File path**: `app/core/src/scenario.rs`
- **Module name**: `runtime`, `render`, `character`
- **Struct/Enum name**: `Scenario`, `CharacterAnimation`, `GameState`
- **Feature name**: `save-system`, `animation`, `text-rendering`

## Output Format

Generates detailed documentation in Markdown format containing:

### 1. Overview Section
- **Purpose**: What this feature/module does
- **Responsibility**: Role within the system
- **Related Phase**: Which roadmap phase this belongs to

### 2. Type Definitions Section
- **Struct/Enum definitions**: All fields and their types
- **Doc comments**: Existing documentation comments
- **Derive traits**: List of derived traits
- **Visibility**: pub/pub(crate)/private distinction

### 3. impl Blocks Section
- **Method list**: All method signatures
- **Method classification**: Constructors, getters, setters, conversions, helpers
- **Important methods**: Highlighted methods to understand

### 4. Trait Implementations Section
- **Standard traits**: Debug, Clone, Serialize, Deserialize, etc.
- **Custom traits**: Project-specific traits
- **Trait bounds**: Traits required in generics

### 5. Dependencies Section
- **Imports**: What this file imports
- **Usage locations**: Where this file's types are used
  - File paths and context
  - Common usage patterns
- **Dependency graph**: ASCII art visualization

### 6. Usage Examples Section
- **Real code examples**: Actual usage extracted from codebase
- **Test code**: Usage patterns from unit tests
- **Scenario examples**: TOML file usage (when applicable)

### 7. Related Files Section
- **Related modules**: Files to understand together
- **Documentation**: Related docs in docs/ directory
- **Next files to read**: Recommended path to deepen understanding

## Processing Steps

1. **Identify target**
   - Search for target file using Glob/Grep
   - Display list for confirmation if multiple candidates exist

2. **Parse file**
   - Read target file with Read tool
   - Parse structs, enums, impl blocks, traits
   - Extract doc comments

3. **Analyze dependencies**
   - Search for type names across all files using Grep
   - Identify usage in use statements, field types, method arguments
   - List imports (what this file depends on)

4. **Extract usage examples**
   - Extract code examples from actual usage locations
   - Get usage patterns from test files
   - Search assets/scenarios/ for TOML examples (when applicable)

5. **Generate ASCII diagram**
   - Visualize dependencies in ASCII art graph format
   - Distinguish imports (dependencies) and usages
   - Use box drawing characters for clear visualization

6. **Output Markdown**
   - Detailed documentation organized by section
   - Proper syntax highlighting in code blocks
   - Embedded ASCII diagrams

7. **Save to file**
   - Create `.claude/tmp/` directory if it doesn't exist
   - Save output to `.claude/tmp/understand-deep_<target>.md`
   - Sanitize target name for filename (replace `/`, `::`, spaces with `_`)
   - Display file path to user after completion

## Example Output

```markdown
# `Scenario` - Scenario Data Structure

## Overview

**Purpose**: Core data structure representing an entire visual novel scenario

**Responsibilities**:
- Manage collection of scenes
- Store character definitions
- Maintain metadata (title, author, etc.)

**Phase**: Phase 0 (Core Types) / Phase 1 (Runtime)

---

## Type Definitions

### `Scenario` Struct

\`\`\`rust
pub struct Scenario {
    pub metadata: ScenarioMetadata,
    pub scenes: HashMap<SceneId, Scene>,
    pub characters: HashMap<CharacterId, CharacterDef>,
}
\`\`\`

**Fields**:
- `metadata: ScenarioMetadata` - Scenario metadata (title, author, version)
- `scenes: HashMap<SceneId, Scene>` - Collection of scenes keyed by scene ID
- `characters: HashMap<CharacterId, CharacterDef>` - Character definition map

**Derived Traits**: `Debug, Clone, Serialize, Deserialize`

---

## impl Blocks

### Constructors

\`\`\`rust
pub fn new(metadata: ScenarioMetadata, initial_scene: SceneId) -> Self
\`\`\`
Create a new scenario with initial_scene as the entry point.

### Scene Management

\`\`\`rust
pub fn add_scene(&mut self, id: SceneId, scene: Scene)
pub fn get_scene(&self, id: &SceneId) -> Option<&Scene>
pub fn scene_exists(&self, id: &SceneId) -> bool
\`\`\`

### Character Management

\`\`\`rust
pub fn add_character(&mut self, id: CharacterId, def: CharacterDef)
pub fn get_character(&self, id: &CharacterId) -> Option<&CharacterDef>
\`\`\`

---

## Trait Implementations

- **Serialize/Deserialize**: Conversion to/from TOML/RON files
- **Debug**: Debug output
- **Clone**: Deep copy

---

## Dependencies

### What This File Imports

\`\`\`rust
use crate::types::{SceneId, CharacterId};
use crate::config::ScenarioMetadata;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
\`\`\`

### Where This File's Types Are Used

- `app/engine/src/runtime/executor.rs`: Executed in ScenarioRuntime
- `app/engine/src/save/data.rs`: Scene position recorded in save data
- `app/tools/src/scenario_validator.rs`: Validation processing
- `assets/scenarios/*.toml`: Deserialized from TOML

### Dependency Graph

\`\`\`
┌─────────────────────────────────────────────────────────────┐
│                      scenario.rs                            │
│                   (Core Data Structure)                      │
└────────┬────────────────────────────────┬──────────────────┘
         │                                │
         │ Dependencies (imports)         │ Used by
         │                                │
    ┌────▼────────────────┐         ┌────▼──────────────────────┐
    │ types::SceneId      │         │ runtime/executor.rs       │
    │ types::CharacterId  │         │  - ScenarioRuntime        │
    │ ScenarioMetadata    │         └───────────────────────────┘
    │ serde               │
    └─────────────────────┘         ┌───────────────────────────┐
                                    │ save/data.rs              │
                                    │  - Save/load state        │
                                    └───────────────────────────┘

                                    ┌───────────────────────────┐
                                    │ tools/validator.rs        │
                                    │  - Validation             │
                                    └───────────────────────────┘

                                    ┌───────────────────────────┐
                                    │ assets/*.toml             │
                                    │  (deserialization)        │
                                    └───────────────────────────┘
\`\`\`

---

## Usage Examples

### Actual Usage

**Usage in runtime/executor.rs**:
\`\`\`rust
pub struct ScenarioRuntime {
    scenario: Scenario,
    current_scene: SceneId,
    // ...
}

impl ScenarioRuntime {
    pub fn new(scenario: Scenario) -> Self {
        let initial = scenario.metadata.initial_scene.clone();
        Self {
            scenario,
            current_scene: initial,
        }
    }

    pub fn execute_current(&mut self) -> Result<()> {
        let scene = self.scenario.get_scene(&self.current_scene)
            .ok_or(ScenarioError::SceneNotFound)?;
        // ...
    }
}
\`\`\`

### Usage in Tests

**app/core/src/scenario_tests.rs**:
\`\`\`rust
#[test]
fn test_scenario_creation() {
    let metadata = ScenarioMetadata::new("chapter_01", "Chapter 1");
    let mut scenario = Scenario::new(metadata, "scene_01");

    let scene = Scene::new("scene_01", "Opening");
    scenario.add_scene("scene_01", scene);

    assert!(scenario.scene_exists(&"scene_01"));
}
\`\`\`

### TOML File Usage

**assets/scenarios/chapter_01.toml**:
\`\`\`toml
[metadata]
id = "chapter_01"
title = "Chapter 1: The Beginning"
author = "Story Team"
version = "1.0.0"

[characters.alice]
name = "Alice"
sprite_path = "characters/alice"

[[scenes]]
id = "scene_01"
name = "Opening Scene"

[[scenes.commands]]
type = "Dialogue"
# ...
\`\`\`

---

## Related Files

### Related Modules to Understand

1. **Scene** (`scenario/scene.rs`) - Individual scenes that compose a scenario
2. **ScenarioCommand** (`scenario/command.rs`) - Execution commands within scenes
3. **ScenarioRuntime** (`engine/runtime/executor.rs`) - Scenario execution engine

### Related Documentation

- [docs/scenario-format.md](../../../docs/scenario-format.md) - TOML format specification
- [docs/design/roadmap.md](../../../docs/design/roadmap.md) - Phase 0/1 implementation plan

### Next Files to Read

Depending on developer's goal:
- **Learn scenario writing**: `docs/scenario-format.md` → `assets/scenarios/chapter_01.toml`
- **Understand runtime**: `runtime/executor.rs` → `runtime/state_machine.rs`
- **Add validation**: `tools/scenario_validator.rs`

---

**Generated**: 2026-02-15
**Command**: `understand-deep scenario`
```

## Output File

After generating the documentation, save it to:
```
.claude/tmp/understand-deep_<sanitized_target>.md
```

**Filename sanitization rules**:
- Replace `/` with `_` (e.g., `app/core/src/scenario.rs` → `understand-deep_app_core_src_scenario_rs.md`)
- Replace `::` with `_` (e.g., `runtime::executor` → `understand-deep_runtime_executor.md`)
- Replace spaces with `_` (e.g., `save system` → `understand-deep_save_system.md`)
- Convert to lowercase for consistency
- Remove special characters except `_` and `-`

**Examples**:
- `understand-deep Scenario` → `.claude/tmp/understand-deep_scenario.md`
- `understand-deep app/core/src/scenario.rs` → `.claude/tmp/understand-deep_app_core_src_scenario_rs.md`
- `understand-deep runtime::executor` → `.claude/tmp/understand-deep_runtime_executor.md`
- `understand-deep save-system` → `.claude/tmp/understand-deep_save-system.md`

**Directory creation**:
- If `.claude/tmp/` doesn't exist, create it using Bash: `mkdir -p .claude/tmp`
- Always notify the user of the output file path after completion

## Important Notes

- **Accuracy First**: Accurately reflect actual code behavior
- **Real Examples**: Learn from actual code, not theory
- **Comprehensive**: Don't miss any important usage locations
- **Readable**: Write in a way new developers can understand
- **Up-to-date**: Reflect the current state of the codebase
- **Save Output**: Always save the complete documentation to `.claude/tmp/understand-deep_*.md`

## Technical Implementation Guidelines

### Search Strategy

1. **Glob search**: Search by file name pattern
   - `app/**/src/**/${target}*.rs`
   - For module names, also search `**/mod.rs`

2. **Grep search**: Content-based search
   - `struct ${target}` - Struct definitions
   - `enum ${target}` - Enum definitions
   - `impl.*${target}` - impl blocks
   - `use.*${target}` - import statements
   - `${target}::` - Usage locations

3. **Identify dependencies**:
   ```bash
   # imports (this file's dependencies)
   Grep pattern="^use " file_path=target_file

   # usages (used by others)
   Grep pattern="TargetType" path=app/ output_mode=files_with_matches
   ```

### ASCII Diagram Generation Rules

```
┌─────────────────────────────────────────────────────────────┐
│                      target.rs                               │
│                   (Target Module)                            │
└────────┬────────────────────────────────┬──────────────────┘
         │                                │
         │ Dependencies (imports)         │ Used by
         │                                │
    ┌────▼────────────────┐         ┌────▼──────────────────────┐
    │ dependency1         │         │ user1.rs                  │
    │ dependency2         │         │  - Context                │
    │ dependency3         │         └───────────────────────────┘
    └─────────────────────┘
                                    ┌───────────────────────────┐
                                    │ user2.rs                  │
                                    │  - Context                │
                                    └───────────────────────────┘

                                    ┌───────────────────────────┐
                                    │ external.toml             │
                                    │  (data file reference)    │
                                    └───────────────────────────┘
```

**Format rules**:
- Use box drawing characters (┌─┐│└┘├┤┬┴┼)
- Target module at top in a box
- Dependencies on left side
- Users on right side
- Data files indicated with parentheses
- Keep layout clean and readable in VSCode

### Error Handling

- **File not found**: Suggest similar files
- **Multiple candidates**: Prompt user to select
- **Parse failure**: Output partial information

### Performance Optimization

- Parallelize Grep searches when possible
- For large files, use offset/limit for chunked reading
- Extract only necessary information (don't copy entire content)

## Extensibility

Future features to add:

- **Change history**: Display recent changes from git log
- **Metrics**: Code complexity, test coverage
- **Refactoring suggestions**: Improvement recommendations
- **Interactive mode**: Display details progressively
- **Export**: HTML, PDF output

---

**Purpose of this command**: Help new developers get started quickly and existing developers recall forgotten implementations.
