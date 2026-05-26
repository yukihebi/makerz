use std::io;

use thiserror::Error;

use crate::cli::ParseError;
use crate::directive_parser::ParseMakefileError;
use crate::discovery::DiscoveryError;

/// Common error type for makerz.
#[derive(Debug, Error)]
pub enum Error {
    /// Failure to parse makerz's own CLI arguments.
    #[error("argument error: {0}")]
    ArgParse(#[from] ParseError),

    /// Failure to locate a `Makefile.toml` upward from cwd.
    #[error("discovery error: {0}")]
    Discovery(#[from] DiscoveryError),

    /// Failure while parsing the discovered Makefile.toml.
    #[error("makefile parse error: {0}")]
    ParseMakefile(#[from] ParseMakefileError),

    /// Failure to obtain the current working directory.
    #[error("failed to read current directory: {0}")]
    Cwd(#[source] io::Error),

    /// `makers` binary missing on PATH.
    #[error("`makers` not found on PATH; install cargo-make (e.g., `cargo install cargo-make`)")]
    MakersNotFound,

    /// Spawn IO error other than NotFound.
    #[error("failed to spawn `makers`: {0}")]
    MakersSpawn(#[source] io::Error),

    /// Exit code from `makers`. Non-zero by construction.
    #[error("`makers` exited with code {0}")]
    MakersExited(i32),

    /// `makers` exited without a code. Unix-only in practice (signal-killed);
    /// Windows always surfaces a code.
    #[error("`makers` terminated by signal")]
    MakersTerminatedBySignal,
}

impl Error {
    /// Exit code makerz itself should return for this error.
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::ArgParse(_) => 2,
            Error::Discovery(_) => 1,
            Error::ParseMakefile(_) => 1,
            Error::Cwd(_) => 1,
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
