use std::io;

use thiserror::Error;

use crate::cli::ParseError;

/// Common error type for makerz.
///
/// Variants here are the minimum required by the cli-skeleton scope.
/// Subsequent PRs add variants for discovery, parsing, and resolution failures.
#[derive(Debug, Error)]
pub enum Error {
    /// Failure to parse makerz's own CLI arguments.
    #[error("argument error: {0}")]
    ArgParse(#[from] ParseError),

    /// `makers` binary missing on PATH.
    #[error("`makers` not found on PATH; install cargo-make (e.g., `cargo install cargo-make`)")]
    MakersNotFound,

    /// Spawn IO error other than NotFound.
    #[error("failed to spawn `makers`: {0}")]
    MakersSpawn(#[source] io::Error),

    /// `makers` ran and exited with the given non-zero code.
    #[error("`makers` exited with code {0}")]
    MakersExited(i32),

    /// `makers` was terminated by a signal (no exit code).
    #[error("`makers` terminated by signal")]
    MakersTerminatedBySignal,
}

impl Error {
    /// Exit code makerz itself should return for this error.
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::ArgParse(_) => 2,
            Error::MakersNotFound => 127,
            Error::MakersSpawn(_) => 1,
            Error::MakersExited(c) => u8::try_from(*c).unwrap_or(1),
            Error::MakersTerminatedBySignal => 1,
        }
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
