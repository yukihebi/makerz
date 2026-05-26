use std::ffi::OsString;
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
/// Returns [`OsString`]s to preserve non-UTF-8 paths losslessly on Unix.
///
/// Any `--cwd` on the passthrough side is intentionally not parsed here; if
/// the user supplied one, `makers` itself decides which wins.
pub fn build_args(makefile_dir: &Path, passthrough: &[String]) -> Vec<OsString> {
    let mut args = Vec::with_capacity(passthrough.len() + 2);
    args.push(OsString::from("--cwd"));
    args.push(makefile_dir.as_os_str().to_os_string());
    args.extend(passthrough.iter().map(OsString::from));
    args
}

/// Spawn `binary` with `args`, inheriting stdio, and translate the result into [`Error`].
pub fn spawn(binary: &str, args: &[OsString]) -> Result<(), Error> {
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
