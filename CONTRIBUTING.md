# Contributing to Prytty

Prytty welcomes contributions. Here's how to get started.

## Setup

```sh
git clone https://github.com/calibrae/prytty.git
cd prytty
cargo build
cargo test
```

Requires Rust 2024 edition (rustc 1.85+).

## Adding a new grammar

1. Create `prytty-syntax/src/<language>.rs` implementing the `Grammar` trait
2. Add the language variant to `Language` enum in `prytty-core/src/detect.rs`
3. Add detection rules (extension, shebang, content heuristics)
4. Wire it up in `prytty-syntax/src/lib.rs`
5. Add `style_for` mapping in `prytty-core/src/theme.rs` if new token kinds are needed
6. Add unit tests in the grammar file and integration tests

## Guidelines

- **Keep it light.** No heavy dependencies. The release binary should stay under 1MB.
- **Lossless tokenization.** Joining all `token.text` must reproduce the original input exactly. Test this.
- **UTF-8 safe.** All byte-level slicing must land on char boundaries. Use `advance_char()` for fallback paths.
- **No panics on bad input.** Every tokenizer processes untrusted data from pipes. Bounds-check everything.
- **ANSI sanitization.** Input is stripped of pre-existing escape sequences by default. Don't bypass this.
- **Tests.** Unit tests in each grammar file, integration tests for the pipeline, E2E tests for the CLI.

## Code style

- `cargo fmt` before committing
- `cargo clippy` should be clean
- Commit messages: short, descriptive, no AI attribution

## Reporting issues

Open an issue at https://github.com/calibrae/prytty/issues with:
- What you ran
- What you expected
- What happened instead
- Input that reproduces it (if possible)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
