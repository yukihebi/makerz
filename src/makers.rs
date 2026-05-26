use std::io;
use std::process::Command;

use crate::error::Error;

/// Name of the `makers` binary on PATH.
pub const MAKERS_BINARY: &str = "makers";

/// Build the argv that `makers` should be invoked with.
pub fn build_args(passthrough: &[String]) -> Vec<String> {
    passthrough.to_vec()
}

/// Spawn `binary` with `args`, inheriting stdio, and translate the result into [`Error`].
pub fn spawn(binary: &str, args: &[String]) -> Result<(), Error> {
    let status = match Command::new(binary).args(args).status() {
        Ok(s) => s,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Err(Error::MakersNotFound),
        Err(err) => return Err(Error::MakersSpawn(err)),
    };
    if status.success() {
        Ok(())
    } else {
        Err(match status.code() {
            Some(code) => Error::MakersExited(code),
            None => Error::MakersTerminatedBySignal,
        })
    }
}

#[cfg(test)]
#[path = "makers_tests.rs"]
mod tests;
