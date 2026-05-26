use std::io;
use std::path::Path;
use std::process::Command;

use crate::error::Error;

/// Name of the `makers` binary on PATH.
pub const MAKERS_BINARY: &str = "makers";

/// Build the argv that `makers` should be invoked with.
///
/// `makefile_dir` is prepended as `--cwd <dir>` so that the discovered
/// `Makefile.toml` is resolved consistently regardless of the shell's cwd.
/// Any `--cwd` on the passthrough side is intentionally not parsed here; if
/// the user supplied one, `makers` itself decides which wins.
pub fn build_args(makefile_dir: &Path, passthrough: &[String]) -> Vec<String> {
    let mut args = Vec::with_capacity(passthrough.len() + 2);
    args.push("--cwd".to_string());
    args.push(makefile_dir.to_string_lossy().into_owned());
    args.extend(passthrough.iter().cloned());
    args
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
