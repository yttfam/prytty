# Integration spec from crytter

## What crytter needs from prytty-wasm

A single function:

```rust
/// Highlight text, return ANSI-colored string.
/// language: optional hint ("rust", "json", "diff", etc.)
/// If None, auto-detect from content.
pub fn highlight(text: &str, language: Option<&str>) -> String;
```

That's it. Crytter's VTE parser + grid already handles all ANSI escape sequences (SGR colors, 256-color, RGB). Prytty just needs to emit standard ANSI color codes.

## How crytter will use it

Two modes:

### 1. Client-side highlighting (WASM)
For paste preview, file viewing, or when the PTY output isn't already colored:
```javascript
import { highlight } from 'prytty-wasm';
const colored = highlight(rawText, 'rust');
term.write(colored);
```

### 2. Server-side middleware (hermytt integration)
Prytty sits between the PTY output and the transport. Not crytter's concern — that's between prytty and hermytt.

## Color format requirements

- Output standard ANSI SGR sequences: `\x1b[38;5;Nm` (256-color) or `\x1b[38;2;R;G;Bm` (RGB)
- Always reset at end of output: `\x1b[0m`
- No cursor movement, no erase sequences — just color
- Line-by-line streaming preferred (for large files)

## WASM build requirements

- Target: `wasm32-unknown-unknown` with `wasm-bindgen`
- Keep it small — crytter is 85KB, prytty-wasm should ideally be under 200KB
- Export via `#[wasm_bindgen]` with JS-friendly API
- Ship as npm-compatible pkg (same as crytter uses wasm-pack)

## Theme

Default theme should look good on dark backgrounds (#1e1e1e). Crytter's default palette:
- Background: `#1e1e1e`
- Foreground: `#d4d4d4`
- ANSI colors: xterm defaults

If prytty supports themes, a VS Code Dark+ style default would be ideal.

## No rush

Crytter works fine without prytty — it's an enhancement. Build prytty's CLI first, add WASM when the core is solid.
