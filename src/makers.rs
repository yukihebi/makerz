use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use crate::error::Error;

/// Name of the `makers` binary on PATH.
const MAKERS_BINARY: &str = "makers";

/// One `--env KEY=VALUE` pair to be passed to `makers`.
///
/// Producers (`caller` now, future `env-resolution`) emit these and the
/// main flow pushes them onto an [`Invocation`]. A named type keeps the
/// producer/consumer contract explicit when several producers contribute
/// to the same env set.
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

/// One pending `makers` invocation: `--cwd` + accumulated env overrides +
/// passthrough args.
///
/// Built up by `main::passthrough` (and later flows) by `push_env` /
/// `extend_env` from each producer, then handed off to [`Invocation::run`].
/// Splitting [`Self::to_argv`] from [`Self::run`] keeps argv assembly
/// unit-testable without spawning the real binary.
#[derive(Debug)]
pub struct Invocation {
    cwd: PathBuf,
    env_entries: Vec<EnvEntry>,
    passthrough: Vec<String>,
}

impl Invocation {
    pub fn new(cwd: PathBuf, passthrough: Vec<String>) -> Self {
        Self {
            cwd,
            env_entries: Vec::new(),
            passthrough,
        }
    }

    pub fn push_env(&mut self, entry: EnvEntry) {
        self.env_entries.push(entry);
    }

    #[allow(dead_code)]
    pub fn extend_env<I: IntoIterator<Item = EnvEntry>>(&mut self, entries: I) {
        self.env_entries.extend(entries);
    }

    /// Render the full `makers` argv: `--cwd <dir>` then each env entry then
    /// passthrough. A `--cwd` on the passthrough side is left alone; if the
    /// user supplied one, `makers` itself decides which wins.
    pub fn to_argv(&self) -> Vec<OsString> {
        let mut args = Vec::with_capacity(self.passthrough.len() + 2 + self.env_entries.len() * 2);
        args.push(OsString::from("--cwd"));
        args.push(self.cwd.as_os_str().to_os_string());
        for entry in &self.env_entries {
            args.extend(entry.to_argv());
        }
        args.extend(self.passthrough.iter().map(OsString::from));
        args
    }

    /// Spawn `makers` with the rendered argv. Inherits stdio.
    pub fn run(&self) -> Result<(), Error> {
        spawn(MAKERS_BINARY, &self.to_argv())
    }
}

/// Spawn `binary` with `args`, inheriting stdio, and translate the result into [`Error`].
fn spawn(binary: &str, args: &[OsString]) -> Result<(), Error> {
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
