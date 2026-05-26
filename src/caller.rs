//! Resolve the `caller` directive on the active `Makefile.toml`.
//!
//! Emits the `--env` entry that records the makerz caller's true cwd for
//! use by `makers` tasks. The env var name is whatever the user bound the
//! directive to; `--init` defaults to `CALLER_DIR` but users may pick any
//! name. When no `caller` directive is present, no env entry is produced
//! and `makers` runs with `--cwd` only.

use std::path::Path;

use crate::directive_parser::ParsedMakefile;
use crate::makers::EnvEntry;

/// Inspect the active Makefile's `caller` binding and produce the
/// [`EnvEntry`] to inject as `--env`. Returns `None` when the Makefile
/// has no `caller` directive.
pub fn resolve_caller_env(parsed: &ParsedMakefile, caller_cwd: &Path) -> Option<EnvEntry> {
    let binding = parsed.env().caller()?;
    Some(EnvEntry::new(binding.name(), caller_cwd.as_os_str()))
}

#[cfg(test)]
#[path = "caller_tests.rs"]
mod tests;
