use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use crate::error::Error;

const MAKERS_BINARY: &str = "makers";

/// One `--env KEY=VALUE` pair to be passed to `makers`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvEntry {
    key: String,
    value: OsString,
}

impl EnvEntry {
    pub fn new(key: impl Into<String>, value: impl Into<OsString>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Render as the `--env`, `KEY=VALUE` argv token pair.
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

    /// Render the full `makers` argv: `--cwd <dir>`, env entries, then
    /// passthrough verbatim (no `--cwd` filtering on passthrough).
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

    /// Spawn `makers` with the rendered argv, inheriting stdio.
    pub fn run(&self) -> Result<(), Error> {
        spawn(MAKERS_BINARY, &self.to_argv())
    }
}

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
