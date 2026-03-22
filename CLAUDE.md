You are building Prytty — a Rust syntax highlighter and pretty-printer for terminal output. The third child of the YTT dynasty (Hermytt → Crytter → Prytty).

Prytty makes terminal output beautiful. Code highlighting, structured data formatting, log colorization, diff rendering — anything that deserves better than monochrome stdout.

## The YTT Family

- **Hermytt** — transport-agnostic terminal multiplexer (PTY over REST/MQTT/Telegram/WS)
- **Crytter** — WASM terminal emulator (37KB, xterm.js replacement)
- **Prytty** — syntax highlighting and pretty-printing for terminal output

## The Idea

```
raw output → prytty → beautiful terminal output

# Pipe-friendly
cat main.rs | prytty
kubectl get pods -o json | prytty
git diff | prytty
tail -f /var/log/syslog | prytty

# As a library
use prytty::highlight;
let colored = highlight("fn main() {}", "rust");
```

## Architecture

```
prytty/
├── prytty-core/          # Language detection, theme engine, ANSI output
├── prytty-syntax/        # Syntax grammars (tree-sitter or custom)
│   ├── rust.rs
│   ├── python.rs
│   ├── json.rs
│   ├── yaml.rs
│   ├── toml.rs
│   ├── diff.rs
│   ├── log.rs            # syslog, journald, nginx, etc.
│   └── generic.rs        # fallback: strings, numbers, brackets
├── prytty-formats/       # Structured data pretty-printing
│   ├── json.rs           # jq-style colorized + formatted
│   ├── xml.rs
│   ├── csv.rs
│   └── table.rs          # aligned columns
├── prytty-cli/           # CLI: pipe mode, file mode, stdin detection
└── prytty-wasm/          # WASM build for Crytter integration
```

## Key Features

### Syntax Highlighting
- Auto-detect language from file extension, shebang, or content
- tree-sitter grammars for accuracy (or lightweight custom parsers for speed)
- 16-color, 256-color, and truecolor output
- Themes: dracula, monokai, solarized, catppuccin, nord

### Structured Data
- JSON: colorized keys/values + pretty-printed (like jq but Rust)
- YAML/TOML: same treatment
- CSV: auto-aligned columns with header highlighting
- Tables: detect tabular output and align

### Log Colorization
- Auto-detect log format (syslog, journald, nginx, Apache)
- Timestamp in one color, level in another (ERROR=red, WARN=yellow, INFO=green)
- Highlight IPs, URLs, paths, error codes

### Diff Rendering
- Git diff: red/green with +/- gutter
- Side-by-side mode
- Word-level diff highlighting

### Pipe Integration
- Detect if stdin is a pipe → auto-highlight
- Detect if stdout is a terminal → output ANSI, else strip colors
- Stream-friendly: process line by line, don't buffer everything
- Fast: must not noticeably slow down `cat file | prytty`

## Integration with Siblings

- **Crytter**: Prytty compiles to WASM, Crytter uses it for in-browser syntax highlighting
- **Hermytt**: Prytty can be a middleware — terminal output passes through Prytty before reaching the transport

## Tech Stack

- `tree-sitter` — syntax parsing (optional, for accuracy)
- `syntect` — alternative highlighter (simpler, uses Sublime Text syntaxes)
- `crossterm` — terminal color output
- `clap` — CLI
- `serde` — theme config
- No heavy dependencies — binary should stay under 2MB

## Design Choice: tree-sitter vs syntect vs custom

- **syntect**: proven, uses .tmLanguage grammars, but 4MB+ binary with all grammars
- **tree-sitter**: accurate parsing, but large grammar files, complex build
- **Custom**: lightweight regex-based, fast, tiny binary, covers 80% of cases
- **Recommendation**: start with custom for the common languages + syntect as optional feature flag

## Cali's Preferences

- Tiny binary, aggressive release profile
- Pipe-friendly by default (`cat file | prytty` must just work)
- Auto-detection over explicit flags
- Beautiful defaults, no config needed for 90% of use
- Part of the YTT family — consistent naming, consistent quality
- The name is Prytty. Because terminals deserve to be pretty.
