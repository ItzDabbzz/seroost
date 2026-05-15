# Changelog

All notable changes to Seroost will be documented in this file.

## [0.1.3] - 2025-05-15

### Added

- **Configurable code file extensions** via `--code-ext` / `-x` CLI flag
  - Pass comma-separated extensions to treat as code files (e.g. `-x nix,tf,graphql`)
  - Merges with built-in set and persists to config
  - Built-in list expanded from ~25 to ~80 extensions covering web (jsx, tsx, vue, svelte), shell (sh, bash, ps1, bat), config (toml, yaml, json, ini), and many more languages (zig, nim, dart, julia, crystal, ocaml, fsharp, raku, etc.)
- **Windows release packaging script** (`scripts/release.ps1`)
  - Builds release binary and zips to `releases/seroost-<version>-<target>.zip`
  - Includes binary + readme + changelog
- **Bash release packaging script** (`scripts/release.sh`)
  - Linux/macOS equivalent for cross-platform release builds

### Changed

- `is_code_file()` now accepts user-supplied extra extensions alongside built-in defaults
- Tests updated to reflect expanded code file extension coverage
- `.gitignore` updated to exclude release artifacts (`/dist`, `/releases`, `*.zip`, `*.tar.gz`)

## [0.1.2] - 2025-09-14

### Added

- **Code output mode** (`--mode code`) with structured JSON-friendly search results
- **Code file parsing support** with line number tracking via `get_code_line_info`
- Support for multiple programming languages (Rust, Python, JavaScript, TypeScript, Java, C/C++, Go, PHP, Ruby, Swift, Kotlin)
- Line-numbered content indexing for better code search accuracy

### Enhanced

- Extended file format support to include common programming language files
- Improved search precision for source code with contextual line information

## [0.1.1] - 2025-04-20

### Added

- **Multi-threaded architecture** with automatic CPU core detection
- **Color-coded CLI output** for better user feedback
- **System-aware configuration storage** (`~/.config/seroost/`)
- `CHANGELOG.md` and expanded `readme.md`

### Changed

- Complete rewrite of indexing system for parallel processing
- Enhanced error reporting with colored output
- Restructured project architecture: split `main.rs` into `interact.rs` and `interactives.rs`

## [0.1.0] - 2025-03-30

### Added

- Support for multiple file formats (PDF, TXT, XML, HTML)
- TF-IDF based search implementation
- Command-line interface with `index` and `search` commands
- Recursive directory traversal
- Simple configuration system
- `readme.md` with usage instructions
