use std::fmt;
use std::io;

use crate::cli::ParseError;

/// Common error type for makerz.
///
/// Variants here are the minimum required by the cli-skeleton scope.
/// Subsequent PRs add variants for discovery, parsing, and resolution failures.
#[derive(Debug)]
pub enum Error {
    /// Failure to parse makerz's own CLI arguments. Wraps the per-reason variant.
    ArgParse(ParseError),
    /// `makers` binary missing on PATH.
    MakersNotFound,
    /// Spawn IO error other than NotFound.
    MakersSpawn(io::Error),
    /// `makers` ran but exited non-zero.
    MakersFailed { code: Option<i32> },
}

impl Error {
    /// Exit code makerz itself should return for this error.
    /// For MakersFailed, the inner code is forwarded (unknown -> 1).
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::ArgParse(_) => 2,
            Error::MakersNotFound => 127,
            Error::MakersSpawn(_) => 1,
            Error::MakersFailed { code } => code.and_then(|c| u8::try_from(c).ok()).unwrap_or(1),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ArgParse(inner) => write!(f, "argument error: {inner}"),
            Error::MakersNotFound => write!(
                f,
                "`makers` not found on PATH; install cargo-make (e.g., `cargo install cargo-make`)"
            ),
            Error::MakersSpawn(err) => write!(f, "failed to spawn `makers`: {err}"),
            Error::MakersFailed { code: Some(c) } => write!(f, "`makers` exited with code {c}"),
            Error::MakersFailed { code: None } => write!(f, "`makers` terminated by signal"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ArgParse(inner) => Some(inner),
            Error::MakersSpawn(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::ArgParse(err)
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
