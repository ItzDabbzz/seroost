# Seroost

Fast local document/code search in Rust. Seroost indexes supported files, stores a local search index, then ranks results with TF-IDF.

Credit: original idea comes from Tsoding Daily's XML search engine work. This project extends that idea into a multi-format CLI search tool.

## Features

- Parallel directory indexing with worker threads
- TF-IDF ranked search
- Regular, tree, and JSON search output
- Compact `--ai` index output for agent/model context
- Source-code indexing with line-number search metadata
- Default ignore list for secrets, dependencies, build output, logs, dumps, media, binaries, IDE files
- `.gitignore` support at indexed root
- Saved config/index under system config directory

## Supported Files

Documents:

- PDF
- TXT
- XML / XHTML
- HTML / HTM

Source code:

- Rust, Python, JavaScript, TypeScript
- Java, C, C++, headers
- Go, PHP, Ruby, Swift, Kotlin

## Install

```bash
git clone https://github.com/parado-xy/seroost.git
cd seroost
cargo build --release
```

Optional global command:

```bash
sudo ln -s "$(pwd)/target/release/seroost" /usr/local/bin/
```

## CLI

```bash
seroost [OPTIONS] [COMMAND]
```

Commands:

- `index`: index documents
- `search <term>`: search indexed documents
- `usage`: detailed examples

Options:

- `-i, --index-path <PATH>`: directory to index; saved for later searches
- `-f, --file-size <MB>`: max file size; default `25`
- `-m, --mode <regular|tree|code>`: search output mode; default `regular`
- `-e, --ignore <PATTERNS>`: comma-separated extra ignore patterns
- `--no-default-ignore`: disable built-in ignores
- `-a, --ai`: compact index output

## Index

```bash
seroost --index-path /path/to/documents index
```

With larger file limit:

```bash
seroost --index-path /path/to/documents --file-size 50 index
```

With extra ignores:

```bash
seroost --index-path /repo --ignore "*.lock,tmp,fixtures" index
```

AI-friendly index output:

```bash
seroost --index-path /repo --ai index
```

Example:

```text
index:/home/me/.config/seroost/index.json
indexed:2
I:/repo/src/main.rs
I:/repo/readme.md
ignored:2
G:/repo/node_modules
G:/repo/target
done
```

Normal index output lists ignored roots without spamming every nested ignored file:

```text
Saving index to: /home/me/.config/seroost/index.json
Ignored roots: 2
  /repo/node_modules
  /repo/target
Successfully indexed 12 documents
```

## Search

Regular ranked output:

```bash
seroost search "query terms"
```

Tree output for project structure:

```bash
seroost --mode tree search "query terms"
```

Example:

```text
Search tree for: query terms
└── /repo
    └── src
        ├── main.rs [#1 score=0.51234]
        └── interact.rs [#2 score=0.33120]
```

JSON/code output:

```bash
seroost --mode code search "query terms"
```

Use `code` mode when another tool should parse results or when line matches matter.

## Config

Seroost uses the system config directory when available:

- Config: `~/.config/seroost/config.json`
- Index: `~/.config/seroost/index.json`

Fallback paths:

- `./indeces/config.json`
- `./indeces/index.json`

## Development

Run from source:

```bash
cargo run -- --help
cargo run -- --index-path ./docs index
cargo run -- --mode tree search "example"
```

Quality checks:

```bash
cargo check --all-targets --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
```

## Project Structure

```text
seroost/
├── build.rs
├── Cargo.toml
├── readme.md
└── src/
    ├── main.rs          # CLI/config flow
    ├── lexer.rs         # tokenization
    ├── parsers.rs       # PDF/TXT/XML/HTML/code readers
    ├── interact.rs      # search/results output
    └── interactives.rs  # threaded indexing
```

## License

MIT
