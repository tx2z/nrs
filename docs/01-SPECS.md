# nrs - npm Run Scripts

## Technical Specification Document

**Version:** 1.0.0  
**Last Updated:** December 2024  
**Status:** Draft

---

## Table of Contents

1. [Overview](#1-overview)
2. [Commands & CLI Interface](#2-commands--cli-interface)
3. [Terminal User Interface (TUI)](#3-terminal-user-interface-tui)
4. [Package Manager Detection](#4-package-manager-detection)
5. [Script Discovery & Parsing](#5-script-discovery--parsing)
6. [Description Sources](#6-description-sources)
7. [Sorting & Filtering](#7-sorting--filtering)
8. [History & Rerun](#8-history--rerun)
9. [Configuration System](#9-configuration-system)
10. [Monorepo & Workspaces Support](#10-monorepo--workspaces-support)
11. [Keyboard Shortcuts](#11-keyboard-shortcuts)
12. [Error Handling](#12-error-handling)
13. [Performance Requirements](#13-performance-requirements)
14. [File Structure](#14-file-structure)
15. [Dependencies](#15-dependencies)

---

## 1. Overview

### 1.1 Purpose

`nrs` (npm Run Scripts) is a fast, interactive terminal user interface (TUI) for discovering and executing npm/yarn/pnpm/bun scripts defined in `package.json` files.

### 1.2 Goals

- **Fast**: Sub-50ms startup time (Rust native binary)
- **Intuitive**: Number keys for quick execution, fuzzy search, visual grid
- **Smart**: Auto-detect package manager, remember history, show descriptions
- **Cross-platform**: Linux and macOS support
- **Zero-config**: Works out of the box, optional configuration for power users

### 1.3 Target Users

- JavaScript/TypeScript developers
- Teams with many npm scripts
- Developers who frequently switch between projects
- Anyone tired of typing `npm run <script>` repeatedly

### 1.4 Non-Goals

- Windows support (initially)
- Package installation features
- Script editing/creation
- Dependency management

---

## 2. Commands & CLI Interface

### 2.1 Primary Command

| Command | Description |
|---------|-------------|
| `nrs` | Launch interactive TUI to select and run a script |

**Rerun last script:** Use `nrs -l` (or `nrs --last`) to quickly rerun the last executed script without opening the TUI.

### 2.2 Command-Line Arguments

```
nrs [OPTIONS] [PATH]

ARGUMENTS:
    [PATH]    Path to project directory (default: current directory)

OPTIONS:
    -h, --help              Show help message
    -v, --version           Show version number
    -L, --last              Rerun last executed script (no TUI)
    -l, --list              List scripts non-interactively (no TUI)
    -e, --exclude <PATTERN> Exclude scripts matching pattern (can be repeated)
    -s, --sort <MODE>       Initial sort mode: recent|alpha|category (default: recent)
    -r, --runner <RUNNER>   Override package manager: npm|yarn|pnpm|bun
    -a, --args <ARGS>       Arguments to pass to the selected script
    -n, --script <NAME>     Run script directly without TUI
    -d, --dry-run           Show command without executing
    -c, --config <PATH>     Path to config file
        --no-config         Ignore config files
        --debug             Enable debug output
```

### 2.3 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | No package.json found |
| 3 | No scripts defined |
| 4 | Script execution failed |
| 5 | Invalid configuration |
| 130 | Interrupted (Ctrl+C) |

### 2.4 Examples

```bash
# Launch TUI in current directory
nrs

# Launch TUI for specific project
nrs ./my-project

# Rerun last script
nrs --last

# List scripts without TUI
nrs --list

# Run specific script with arguments
nrs -n test -- --coverage --watch

# Exclude test scripts
nrs --exclude "test*" --exclude "lint*"

# Use yarn regardless of detection
nrs --runner yarn

# Dry run to see command
nrs -n build --dry-run
```

---

## 3. Terminal User Interface (TUI)

### 3.1 Layout Structure

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📦 project-name                                              pnpm ▪ [?]     │  <- Header
├──────────────────────────────────────────────────────────────────────────────┤
│  🔍 filter-text_                                                             │  <- Filter Bar
├──────────────────────────────────────────────────────────────────────────────┤
│  SCRIPTS                                                    [Recent ▾]       │  <- Section Header
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  1 ❯dev          2  build        3  test         4  lint              │  │
│  │  5  format       6  typecheck    7  serve        8  deploy            │  │  <- Scripts Grid
│  │  9  clean           docs            release         publish           │  │
│  │     start           start:dev       start:prod      e2e               │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│  DESCRIPTION                                                                 │  <- Description Header
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  Start the development server with hot module replacement              │  │  <- Description Text
│  │  ─────────────────────────────────────────────────────────────────────│  │
│  │  $ vite --mode development --host                                      │  │  <- Command Preview
│  └────────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│  ↑↓←→ move  ⏎ run  1-9 quick  / filter  s sort  a args  m multi  ? help    │  <- Footer
└──────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Layout Components

#### 3.2.1 Header
- Project name (from `package.json` `name` field, or directory name)
- Package manager indicator with icon
- Help indicator `[?]`

#### 3.2.2 Filter Bar
- Shows current filter text
- Blinking cursor when in filter mode
- Placeholder text when empty: "Type to filter..."

#### 3.2.3 Scripts Grid
- Multi-column responsive grid (horizontal-first reading order)
- Numbers 1-9 on first 9 visible items
- Cursor indicator `❯` on selected item
- Dimmed style for items without numbers (10+)
- Selected item highlighted with background color

#### 3.2.4 Description Panel
- Script description (if available)
- Separator line
- Actual command preview (from scripts object)
- Scrollable if content exceeds panel height

#### 3.2.5 Footer
- Context-sensitive keybinding hints
- Changes based on current mode (normal, filter, multi-select, etc.)

### 3.3 Responsive Column Layout

| Terminal Width | Columns | Min Item Width |
|----------------|---------|----------------|
| < 60 chars | 1 | 58 |
| 60-89 chars | 2 | 28 |
| 90-119 chars | 3 | 28 |
| 120-159 chars | 4 | 28 |
| ≥ 160 chars | 5 | 30 |

**Column Direction Option:**
- `horizontal` (default): 1 2 3 4 / 5 6 7 8 / 9 ...
- `vertical`: 1 4 7 / 2 5 8 / 3 6 9 ...

### 3.4 Color Scheme

| Element | Default Color |
|---------|--------------|
| Header background | Blue |
| Header text | White bold |
| Filter text | Yellow |
| Script number | Cyan bold |
| Script name | White |
| Selected script | White on Blue background |
| Cursor (❯) | Green bold |
| Description | Gray |
| Command preview | Dim white |
| Footer | Dim white |
| Error messages | Red |
| Success messages | Green |

### 3.5 Minimum Terminal Size

- **Width**: 40 characters
- **Height**: 10 rows
- Show error message if terminal is too small

---

## 4. Package Manager Detection

### 4.1 Detection Priority

1. **CLI Override**: `--runner` flag takes highest priority
2. **Config File**: `runner` setting in config
3. **packageManager Field**: `package.json` `packageManager` field (e.g., `"pnpm@8.0.0"`)
4. **Lock File Detection** (in order):
   - `bun.lockb` → bun
   - `pnpm-lock.yaml` → pnpm
   - `yarn.lock` → yarn
   - `package-lock.json` → npm
5. **Fallback**: npm

### 4.2 Package Manager Commands

| Manager | Run Command | Run with Args |
|---------|-------------|---------------|
| npm | `npm run <script>` | `npm run <script> -- <args>` |
| yarn | `yarn <script>` | `yarn <script> <args>` |
| pnpm | `pnpm <script>` | `pnpm <script> <args>` |
| bun | `bun run <script>` | `bun run <script> <args>` |

### 4.3 Package Manager Display

| Manager | Icon | Display |
|---------|------|---------|
| npm | 📦 | `npm` |
| yarn | 🧶 | `yarn` |
| pnpm | 📀 | `pnpm` |
| bun | 🥟 | `bun` |

---

## 5. Script Discovery & Parsing

### 5.1 package.json Location

1. Check provided path argument
2. Search current directory for `package.json`
3. Traverse up to find nearest `package.json` (max 10 levels)

### 5.2 Script Extraction

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "test": "vitest"
  }
}
```

### 5.3 Script Filtering (Built-in)

By default, exclude npm lifecycle scripts unless `--all` flag:
- `preinstall`, `install`, `postinstall`
- `preuninstall`, `uninstall`, `postuninstall`
- `prepublish`, `prepublishOnly`, `publish`, `postpublish`
- `preversion`, `version`, `postversion`

### 5.4 Pre/Post Script Handling

Scripts with `pre` and `post` prefixes are shown but visually grouped:
- `prebuild`, `build`, `postbuild` shown together

---

## 6. Description Sources

### 6.1 Priority Order

1. **scripts-info** (highest priority)
2. **ntl.descriptions** 
3. **// comments**
4. **Fallback**: No description (show command only)

### 6.2 scripts-info Format

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  },
  "scripts-info": {
    "dev": "Start development server with HMR",
    "build": "Build for production"
  }
}
```

### 6.3 ntl.descriptions Format

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  },
  "ntl": {
    "descriptions": {
      "dev": "Start development server with HMR",
      "build": "Build for production"
    }
  }
}
```

### 6.4 Comment Format

```json
{
  "scripts": {
    "//dev": "Start development server with HMR",
    "dev": "vite",
    "//build": "Build for production",
    "build": "vite build"
  }
}
```

Also support:
- `"// dev"`: With space
- `"//dev//"`: Double slash suffix

---

## 7. Sorting & Filtering

### 7.1 Sort Modes

| Mode | Key | Description |
|------|-----|-------------|
| Recent | `r` | Most recently used first (from history) |
| Alphabetical | `a` | A-Z by script name |
| Category | `c` | Grouped by prefix (build:*, test:*, etc.) |

### 7.2 Fuzzy Filtering

- Filter searches both script names AND descriptions
- Case-insensitive
- Supports fuzzy matching (e.g., "bd" matches "build")
- Scoring algorithm: exact match > prefix match > contains > fuzzy
- Real-time filtering as user types

### 7.3 Filter Behavior

- Press `/` or just start typing to enter filter mode
- `Escape` clears filter and exits filter mode
- `Enter` runs first matching script
- `Backspace` deletes characters
- Numbers still work for quick selection while filtering

---

## 8. History & Rerun

### 8.1 History Storage

Location: `~/.config/nrs/history.json`

```json
{
  "version": 1,
  "projects": {
    "/path/to/project": {
      "last_script": "dev",
      "last_run": "2024-12-23T10:30:00Z",
      "scripts": {
        "dev": {
          "count": 42,
          "last_run": "2024-12-23T10:30:00Z",
          "last_args": "--host"
        },
        "build": {
          "count": 15,
          "last_run": "2024-12-22T14:00:00Z",
          "last_args": null
        }
      }
    }
  }
}
```

### 8.2 History Limits

- Max projects: 100 (LRU eviction)
- Max scripts per project: 50
- History file max size: 1MB

### 8.3 Rerun Command (`nrs --last`)

1. Read history for current project directory
2. Find last executed script
3. Execute with same arguments (if any)
4. If no history, show error message: "No previous script found for this project"

### 8.4 Recent Sort Algorithm

Score = (run_count × 0.3) + (recency_score × 0.7)

Where recency_score = 1.0 for today, decaying by 0.1 per day

---

## 9. Configuration System

### 9.1 Config File Locations (Priority Order)

1. `--config` CLI argument
2. `.nrsrc.toml` in current directory
3. `.nrsrc.toml` in project root (where package.json is)
4. `~/.config/nrs/config.toml`

### 9.2 Config File Format (TOML)

```toml
# ~/.config/nrs/config.toml

# General settings
[general]
# Default package manager (overrides auto-detection)
runner = "pnpm"

# Default sort mode: "recent", "alpha", "category"
default_sort = "recent"

# Column direction: "horizontal", "vertical"
column_direction = "horizontal"

# Show command preview in description panel
show_command_preview = true

# Maximum items to show (0 = unlimited)
max_items = 0

[filter]
# Search in descriptions too
search_descriptions = true

# Fuzzy matching
fuzzy = true

# Case sensitive search
case_sensitive = false

[history]
# Enable history tracking
enabled = true

# Max projects to remember
max_projects = 100

# Max scripts per project
max_scripts = 50

[exclude]
# Global patterns to exclude
patterns = [
  "pre*",
  "post*",
]

[appearance]
# Color theme: "default", "minimal", "none"
theme = "default"

# Show icons
icons = true

# Show help footer
show_footer = true

# Compact mode (less padding)
compact = false

[keybindings]
# Custom keybindings (advanced)
# quit = ["q", "Ctrl+c"]
# run = ["Enter", "o"]
# filter = ["/", "Ctrl+f"]
```

### 9.3 Project-Level Config

`.nrsrc.toml` in project directory:

```toml
# Project-specific settings
[general]
runner = "yarn"

[exclude]
patterns = ["internal:*"]

[scripts]
# Custom descriptions (override package.json)
[scripts.descriptions]
dev = "Start dev server on port 3000"
build = "Production build with minification"

# Script aliases (show alternative names)
[scripts.aliases]
d = "dev"
b = "build"
t = "test"
```

---

## 10. Monorepo & Workspaces Support

### 10.1 Workspace Detection

Detect workspaces from:
- `package.json` `workspaces` field
- `pnpm-workspace.yaml`
- `lerna.json`

### 10.2 Workspace Mode

When in monorepo root, show option to:
1. Run scripts from root `package.json`
2. Select a workspace first, then show its scripts
3. Show all scripts from all packages (grouped)

### 10.3 Workspace UI

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📦 monorepo-name                                             pnpm ▪ [?]     │
├──────────────────────────────────────────────────────────────────────────────┤
│  WORKSPACES                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  1 ❯root          2  @app/web      3  @app/api       4  @lib/ui       │  │
│  │  5  @lib/utils    6  @tools/cli                                       │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│  ⏎ select workspace  / filter  Esc back                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10.4 Running Workspace Scripts

Use package manager's workspace command:
- npm: `npm run -w <package> <script>`
- yarn: `yarn workspace <package> <script>`
- pnpm: `pnpm --filter <package> <script>`
- bun: `bun run --filter <package> <script>`

---

## 11. Keyboard Shortcuts

### 11.1 Navigation

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `←` / `h` | Move left |
| `→` / `l` | Move right |
| `Home` / `g` | Go to first item |
| `End` / `G` | Go to last item |
| `Page Up` | Move up one page |
| `Page Down` | Move down one page |

### 11.2 Actions

| Key | Action |
|-----|--------|
| `Enter` / `o` | Run selected script |
| `1-9` | Quick run numbered script |
| `a` | Run with arguments (prompts for input) |
| `m` | Toggle multi-select mode |
| `Space` | Toggle selection (in multi-select) |

### 11.3 Filtering & Sorting

| Key | Action |
|-----|--------|
| `/` | Enter filter mode |
| `Escape` | Clear filter / Exit mode |
| `s` | Cycle sort mode |
| `S` | Open sort menu |

### 11.4 General

| Key | Action |
|-----|--------|
| `q` / `Ctrl+c` | Quit |
| `?` | Show help |
| `r` | Refresh scripts |
| `c` | Copy script command to clipboard |
| `i` | Toggle description panel |

### 11.5 Multi-Select Mode

| Key | Action |
|-----|--------|
| `Space` | Toggle current item |
| `Enter` | Run all selected (in order) |
| `a` | Select all |
| `n` | Select none |
| `Escape` | Exit multi-select |

---

## 12. Error Handling

### 12.1 Error Types

| Error | Message | Recovery |
|-------|---------|----------|
| No package.json | "No package.json found in {path} or parent directories" | Exit with code 2 |
| Invalid JSON | "Failed to parse package.json: {error}" | Exit with code 2 |
| No scripts | "No scripts defined in package.json" | Exit with code 3 |
| Script not found | "Script '{name}' not found" | Show available scripts |
| Execution failed | "Script '{name}' failed with exit code {code}" | Exit with code 4 |
| Terminal too small | "Terminal too small (min: 40x10)" | Wait for resize |
| Permission denied | "Cannot read package.json: permission denied" | Exit with code 1 |
| Config error | "Invalid config at {path}: {error}" | Use defaults, warn |

### 12.2 Error Display

- Show errors in a dedicated error panel
- Red background/text for critical errors
- Yellow for warnings
- Auto-dismiss warnings after 3 seconds
- Press any key to dismiss errors

---

## 13. Performance Requirements

### 13.1 Targets

| Metric | Target |
|--------|--------|
| Startup time (cold) | < 50ms |
| Startup time (warm) | < 20ms |
| Filter response | < 16ms (60fps) |
| Memory usage | < 10MB |
| Binary size | < 5MB |

### 13.2 Optimization Strategies

- Lazy load history file
- Cache parsed package.json
- Debounce filter input (50ms)
- Use efficient fuzzy matching algorithm (skim/fuzzy-matcher)
- Minimize allocations in render loop
- Profile with `cargo flamegraph`

---

## 14. File Structure

### 14.1 Project Structure

```
nrs/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
├── src/
│   ├── main.rs              # Entry point, CLI parsing
│   ├── lib.rs               # Library root
│   ├── cli.rs               # CLI argument definitions
│   ├── config/
│   │   ├── mod.rs           # Config module
│   │   ├── file.rs          # Config file loading
│   │   └── types.rs         # Config structs
│   ├── package/
│   │   ├── mod.rs           # Package module
│   │   ├── manager.rs       # Package manager detection
│   │   ├── scripts.rs       # Script parsing
│   │   ├── descriptions.rs  # Description extraction
│   │   └── workspace.rs     # Monorepo support
│   ├── history/
│   │   ├── mod.rs           # History module
│   │   └── storage.rs       # History file operations
│   ├── tui/
│   │   ├── mod.rs           # TUI module
│   │   ├── app.rs           # Application state
│   │   ├── ui.rs            # UI rendering
│   │   ├── widgets/
│   │   │   ├── mod.rs
│   │   │   ├── header.rs
│   │   │   ├── filter.rs
│   │   │   ├── scripts.rs
│   │   │   └── description.rs
│   │   ├── layout.rs        # Layout calculations
│   │   ├── theme.rs         # Colors and styles
│   │   └── input.rs         # Input handling
│   ├── runner/
│   │   ├── mod.rs           # Runner module
│   │   └── executor.rs      # Script execution
│   ├── filter/
│   │   ├── mod.rs           # Filter module
│   │   └── fuzzy.rs         # Fuzzy matching
│   └── utils/
│       ├── mod.rs           # Utilities
│       ├── paths.rs         # Path utilities
│       └── terminal.rs      # Terminal utilities
├── tests/
│   ├── integration/
│   │   ├── cli_test.rs
│   │   ├── tui_test.rs
│   │   └── fixtures/
│   └── unit/
│       └── *.rs
└── benches/
    └── benchmarks.rs
```

### 14.2 User Data Locations

| Platform | Config | History | Cache |
|----------|--------|---------|-------|
| Linux | `~/.config/nrs/` | `~/.config/nrs/history.json` | `~/.cache/nrs/` |
| macOS | `~/.config/nrs/` | `~/.config/nrs/history.json` | `~/Library/Caches/nrs/` |

---

## 15. Dependencies

### 15.1 Core Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `clap` | CLI argument parsing | 4.x |
| `ratatui` | TUI framework | 0.26.x |
| `crossterm` | Terminal handling | 0.27.x |
| `serde` | Serialization | 1.x |
| `serde_json` | JSON parsing | 1.x |
| `toml` | TOML config parsing | 0.8.x |
| `dirs` | Platform directories | 5.x |
| `fuzzy-matcher` | Fuzzy string matching | 0.3.x |
| `chrono` | Date/time handling | 0.4.x |
| `anyhow` | Error handling | 1.x |
| `thiserror` | Error definitions | 1.x |

### 15.2 Development Dependencies

| Crate | Purpose |
|-------|---------|
| `criterion` | Benchmarking |
| `tempfile` | Test fixtures |
| `assert_cmd` | CLI testing |
| `predicates` | Test assertions |
| `insta` | Snapshot testing |

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| Script | An npm script defined in package.json scripts field |
| Runner | Package manager used to execute scripts (npm, yarn, pnpm, bun) |
| TUI | Terminal User Interface |
| Fuzzy matching | Approximate string matching that tolerates typos |
| Monorepo | A repository containing multiple packages/projects |
| Workspace | A package within a monorepo |

---

## Appendix B: Similar Tools Comparison

| Feature | nrs | ntl | npm-run |
|---------|-----|-----|---------|
| Language | Rust | Node.js | Node.js |
| TUI | ratatui | inquirer | inquirer |
| Startup | ~30ms | ~300ms | ~200ms |
| Binary | Yes | No | No |
| Descriptions | Yes | Yes | No |
| Fuzzy search | Yes | Yes | No |
| Multi-select | Yes | Yes | No |
| Monorepo | Yes | No | No |
| History | Yes | Yes | No |
| Config file | Yes | Limited | No |
