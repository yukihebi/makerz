//! Resolve the `caller` directive on the active `Makefile.toml`.
//!
//! When the active Makefile has a `caller` directive, emit an [`EnvEntry`]
//! binding the user-chosen variable to the makerz caller's cwd in
//! absolute form. Absent the directive, no entry is produced.

use std::io;
use std::path::{self, Path};

use crate::directive_parser::ParsedMakefile;
use crate::makers::EnvEntry;

pub fn resolve_caller_env(
    parsed: &ParsedMakefile,
    caller_cwd: &Path,
) -> io::Result<Option<EnvEntry>> {
    let Some(binding) = parsed.env().caller() else {
        return Ok(None);
    };
    let absolute = path::absolute(caller_cwd)?;
    Ok(Some(EnvEntry::new(binding.name(), absolute)))
}

#[cfg(test)]
#[path = "caller_tests.rs"]
mod tests;
