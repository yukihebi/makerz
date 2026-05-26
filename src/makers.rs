use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::error::Error;

/// Name of the `makers` binary on PATH.
pub const MAKERS_BINARY: &str = "makers";

/// One `--env KEY=VALUE` pair to be passed to `makers`.
///
/// Producers (`caller`, future `env-resolution`) emit these and the main
/// flow merges them into a single slice before handing the result to
/// [`build_args`], which renders each entry as a `--env`/`KEY=VALUE` pair
/// in the argv. Using a named type keeps the producer/consumer contract
/// explicit when several producers contribute to the same env set.
#[derive(Debug, Clone)]
pub struct EnvEntry {
    pub key: String,
    pub value: OsString,
}

impl EnvEntry {
    pub fn new(key: impl Into<String>, value: impl Into<OsString>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Render as the `--env`, `KEY=VALUE` argv token pair `makers` expects.
    pub fn to_argv(&self) -> [OsString; 2] {
        let mut kv = OsString::with_capacity(self.key.len() + 1 + self.value.len());
        kv.push(&self.key);
        kv.push("=");
        kv.push(&self.value);
        [OsString::from("--env"), kv]
    }
}

/// Build the argv that `makers` should be invoked with.
///
/// `makefile_dir` is prepended as `--cwd <dir>` so `makers` operates from that
/// directory regardless of the shell's cwd. Each [`EnvEntry`] is rendered
/// via [`EnvEntry::to_argv`], preserving order. [`OsString`] preserves
/// non-UTF-8 names losslessly on Unix.
///
/// A `--cwd` on the passthrough side is left alone; if the user supplied one,
/// `makers` itself decides which wins.
pub fn build_args(
    makefile_dir: &Path,
    env_entries: &[EnvEntry],
    passthrough: &[String],
) -> Vec<OsString> {
    let mut args = Vec::with_capacity(passthrough.len() + 2 + env_entries.len() * 2);
    args.push(OsString::from("--cwd"));
    args.push(makefile_dir.as_os_str().to_os_string());
    for entry in env_entries {
        args.extend(entry.to_argv());
    }
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
