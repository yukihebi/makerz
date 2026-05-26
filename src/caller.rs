//! Resolve the `caller` directive on the active `Makefile.toml`.
//!
//! Emits the `--env` entry that records the makerz caller's true cwd for
//! use by `makers` tasks. The env var name is whatever the user bound the
//! directive to; `--init` defaults to `CALLER_DIR` but users may pick any
//! name. When no `caller` directive is present, no env entry is produced
//! and `makers` runs with `--cwd` only.

use std::ffi::OsString;
use std::path::Path;

use crate::directive_parser::ParsedMakefile;

/// Inspect the active Makefile's `caller` binding and produce the
/// `(name, value)` pair to inject as `--env`. Returns `None` when the
/// Makefile has no `caller` directive.
pub fn resolve_caller_env(
    parsed: &ParsedMakefile,
    caller_cwd: &Path,
) -> Option<(String, OsString)> {
    let binding = parsed.env().caller()?;
    Some((
        binding.name().to_string(),
        caller_cwd.as_os_str().to_os_string(),
    ))
}

#[cfg(test)]
#[path = "caller_tests.rs"]
mod tests;
