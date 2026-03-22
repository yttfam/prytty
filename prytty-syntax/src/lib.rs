mod generic;
mod rust;
mod python;
mod json;
mod yaml;
mod toml;
mod diff;
mod log;

use prytty_core::{Language, Token};

/// Trait for language-specific tokenizers.
pub trait Grammar {
    fn tokenize<'a>(&self, input: &'a str) -> Vec<Token<'a>>;
}

/// Tokenize input using the appropriate grammar for the detected language.
pub fn tokenize<'a>(lang: Language, input: &'a str) -> Vec<Token<'a>> {
    match lang {
        Language::Rust => rust::RustGrammar.tokenize(input),
        Language::Python => python::PythonGrammar.tokenize(input),
        Language::Json => json::JsonGrammar.tokenize(input),
        Language::Yaml => yaml::YamlGrammar.tokenize(input),
        Language::Toml => toml::TomlGrammar.tokenize(input),
        Language::Diff => diff::DiffGrammar.tokenize(input),
        Language::Log => log::LogGrammar.tokenize(input),
        Language::Generic => generic::GenericGrammar.tokenize(input),
    }
}
