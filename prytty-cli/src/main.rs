use clap::Parser;
use is_terminal::IsTerminal;
use prytty_core::{detect_language, strip_ansi, AnsiWriter, ColorMode, Language};
use prytty_formats::format_json;
use prytty_syntax::tokenize;
use std::io::{self, Read, Write};
use std::path::PathBuf;

/// Maximum input size: 512 MiB. Prevents OOM from unbounded stdin/file reads.
const MAX_INPUT_SIZE: u64 = 512 * 1024 * 1024;

#[derive(Parser)]
#[command(name = "prytty", about = "Syntax highlighting for terminal output")]
struct Cli {
    /// File to highlight (reads stdin if omitted)
    file: Option<PathBuf>,

    /// Language hint (auto-detected if omitted)
    #[arg(short, long)]
    language: Option<String>,

    /// Color mode: true, 256, 16, none
    #[arg(long)]
    color: Option<String>,

    /// Format/pretty-print structured data (JSON)
    #[arg(short, long)]
    format: bool,

    /// Theme name
    #[arg(short, long, default_value = "dark+")]
    theme: String,

    /// Allow pre-existing ANSI escape sequences in input to pass through
    /// (by default they are stripped to prevent terminal injection)
    #[arg(long)]
    no_sanitize: bool,
}

fn main() {
    let cli = Cli::parse();

    // Read input with size limit to prevent OOM
    let input = match &cli.file {
        Some(path) => {
            // Check file size before reading
            match std::fs::metadata(path) {
                Ok(meta) if meta.len() > MAX_INPUT_SIZE => {
                    eprintln!(
                        "prytty: {}: file too large ({} bytes, max {})",
                        path.display(),
                        meta.len(),
                        MAX_INPUT_SIZE
                    );
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("prytty: {}: {}", path.display(), e);
                    std::process::exit(1);
                }
                _ => {}
            }
            std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("prytty: {}: {}", path.display(), e);
                std::process::exit(1);
            })
        }
        None => {
            let mut buf = String::new();
            io::stdin()
                .take(MAX_INPUT_SIZE)
                .read_to_string(&mut buf)
                .unwrap_or_else(|e| {
                    eprintln!("prytty: stdin: {}", e);
                    std::process::exit(1);
                });
            buf
        }
    };

    // Strip ANSI escape sequences from input to prevent terminal injection.
    // Untrusted input could contain OSC sequences to set terminal title,
    // write to clipboard (OSC 52), or trigger other terminal behavior.
    let input = if cli.no_sanitize { input } else { strip_ansi(&input) };

    // Detect color mode
    let color_mode = match cli.color.as_deref() {
        Some("true") | Some("truecolor") | Some("rgb") => ColorMode::TrueColor,
        Some("256") => ColorMode::Color256,
        Some("16") => ColorMode::Color16,
        Some("none") | Some("off") => ColorMode::None,
        _ => {
            if io::stdout().is_terminal() {
                ColorMode::detect()
            } else {
                ColorMode::None
            }
        }
    };

    // Detect language
    let lang = cli
        .language
        .as_deref()
        .and_then(Language::from_name)
        .unwrap_or_else(|| detect_language(cli.file.as_deref(), &input));

    // Format structured data if requested
    let input = if cli.format && lang == Language::Json {
        format_json(&input)
    } else {
        input
    };

    // Tokenize + render
    let tokens = tokenize(lang, &input);
    let writer = AnsiWriter::new(color_mode, Default::default());
    let output = writer.render(&tokens);

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(output.as_bytes());
}
