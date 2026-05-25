use std::vec::IntoIter;

use thiserror::Error;

/// Result of parsing makerz's own CLI flags.
///
/// `--version`, `--help`, `--init`, and `--extend` are consumed by makerz.
/// Everything else is collected into `Passthrough::args` to be forwarded to `makers`.
#[derive(Debug, PartialEq, Eq)]
pub enum Parsed {
    /// `--version`: print makerz's version.
    Version,
    /// `--help`: print makerz's help.
    Help,
    /// `--init [--extend <path>]`: generate a new `Makefile.toml`.
    Init { extend: Option<String> },
    /// Forward `args` to `makers` as-is.
    Passthrough { args: Vec<String> },
}

/// Distinct reasons a makerz CLI parse can fail.
///
/// Wrapped by [`crate::error::Error::ArgParse`]; surfaced directly to unit tests
/// so assertions can match on the variant instead of the rendered message.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum ParseError {
    /// `--extend` was the last token; no value followed.
    #[error("--extend requires a path argument")]
    ExtendMissingValue,
    /// `--extend=` or `--extend ""` produced an empty path.
    #[error("--extend requires a non-empty path argument")]
    ExtendEmptyValue,
    /// `--extend` was specified without `--init`.
    #[error("--extend requires --init (multi-parent or standalone extend is not supported)")]
    ExtendWithoutInit,
    /// `--extend` was specified more than once.
    #[error("--extend may be specified at most once")]
    ExtendDuplicated,
    /// `--init` was combined with positional or unknown tokens.
    #[error("--init does not accept other arguments (got: {})", .0.join(" "))]
    InitWithExtraArgs(Vec<String>),
}

/// Parse makerz's argv tail (program name already stripped).
pub fn parse(args: Vec<String>) -> Result<Parsed, ParseError> {
    Scan::run(args)?.classify()
}

/// Per-token accumulator. Owns the input iterator so per-token logic stays self-contained.
struct Scan {
    iter: IntoIter<String>,
    help: bool,
    version: bool,
    init: bool,
    extend: Option<String>,
    extend_count: u32,
    passthrough: Vec<String>,
}

impl Scan {
    fn new(args: Vec<String>) -> Self {
        Self {
            iter: args.into_iter(),
            help: false,
            version: false,
            init: false,
            extend: None,
            extend_count: 0,
            passthrough: Vec::new(),
        }
    }

    fn run(args: Vec<String>) -> Result<Self, ParseError> {
        let mut scan = Self::new(args);
        while let Some(arg) = scan.iter.next() {
            scan.consume(arg)?;
        }
        Ok(scan)
    }

    fn consume(&mut self, arg: String) -> Result<(), ParseError> {
        match arg.as_str() {
            "--help" => self.help = true,
            "--version" => self.version = true,
            "--init" => self.init = true,
            "--extend" => {
                let value = self.next_extend_value()?;
                self.set_extend(value)?;
            }
            s if s.starts_with("--extend=") => {
                let value = s["--extend=".len()..].to_string();
                self.set_extend(value)?;
            }
            _ => self.passthrough.push(arg),
        }
        Ok(())
    }

    fn next_extend_value(&mut self) -> Result<String, ParseError> {
        self.iter.next().ok_or(ParseError::ExtendMissingValue)
    }

    fn set_extend(&mut self, value: String) -> Result<(), ParseError> {
        if value.is_empty() {
            return Err(ParseError::ExtendEmptyValue);
        }
        self.extend_count += 1;
        self.extend = Some(value);
        Ok(())
    }

    fn classify(self) -> Result<Parsed, ParseError> {
        if self.help {
            return Ok(Parsed::Help);
        }
        if self.version {
            return Ok(Parsed::Version);
        }
        self.validate_extend()?;
        self.into_init_or_passthrough()
    }

    fn validate_extend(&self) -> Result<(), ParseError> {
        if self.extend_count > 1 {
            return Err(ParseError::ExtendDuplicated);
        }
        if self.extend.is_some() && !self.init {
            return Err(ParseError::ExtendWithoutInit);
        }
        Ok(())
    }

    fn into_init_or_passthrough(self) -> Result<Parsed, ParseError> {
        if self.init {
            if !self.passthrough.is_empty() {
                return Err(ParseError::InitWithExtraArgs(self.passthrough));
            }
            Ok(Parsed::Init {
                extend: self.extend,
            })
        } else {
            Ok(Parsed::Passthrough {
                args: self.passthrough,
            })
        }
    }
}

/// Text printed for `makerz --version`.
pub fn version_text() -> String {
    format!(
        "makerz {}\n(for cargo-make's version, run `makers --version`)",
        env!("CARGO_PKG_VERSION"),
    )
}

/// Text printed for `makerz --help`.
pub fn help_text() -> &'static str {
    "\
makerz - a thin wrapper around cargo-make (`makers`)

USAGE:
    makerz [makers args...]            Forward args to `makers`
    makerz --init                      Generate a new Makefile.toml
    makerz --init --extend <path>      Generate Makefile.toml extending <path>
    makerz --version                   Print makerz's version
    makerz --help                      Print this help

For cargo-make's own usage, run `makers --help`.
"
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
