//! Resolve the `caller` directive on the active `Makefile.toml`.
//!
//! When the active Makefile has a `caller` directive, emit an [`EnvEntry`]
//! binding the user-chosen variable to the makerz caller's true cwd.
//! Absent the directive, no entry is produced.

use std::path::Path;

use crate::directive_parser::ParsedMakefile;
use crate::makers::EnvEntry;

pub fn resolve_caller_env(parsed: &ParsedMakefile, caller_cwd: &Path) -> Option<EnvEntry> {
    let binding = parsed.env().caller()?;
    Some(EnvEntry::new(binding.name(), caller_cwd.as_os_str()))
}

#[cfg(test)]
#[path = "caller_tests.rs"]
mod tests;
