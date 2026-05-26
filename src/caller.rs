//! Resolve the `caller` directive on the active `Makefile.toml`.
//!
//! Validates that the bound fallback path exists, then emits the `--env`
//! entry that records the makerz caller's true cwd for use by `makers`
//! tasks. The env var name is whatever the user bound the directive to;
//! `--init` defaults to `CALLER_DIR` but users may pick any name.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::directive_parser::ParsedMakefile;

#[derive(Debug, Error)]
pub enum CallerError {
    #[error(
        "fallback path for `caller` directive does not exist: `{}` (resolved from Makefile dir)",
        .path.display()
    )]
    FallbackPathMissing { path: PathBuf },
}

/// Inspect the active Makefile's `caller` binding and produce the
/// `(name, value)` pair to inject as `--env`. Returns `Ok(None)` when the
/// Makefile has no `caller` directive.
pub fn resolve_caller_env(
    parsed: &ParsedMakefile,
    caller_cwd: &Path,
) -> Result<Option<(String, OsString)>, CallerError> {
    let Some(binding) = parsed.env().caller() else {
        return Ok(None);
    };
    let resolved = parsed.location().dir().join(binding.fallback());
    if !resolved.exists() {
        return Err(CallerError::FallbackPathMissing { path: resolved });
    }
    Ok(Some((
        binding.name().to_string(),
        caller_cwd.as_os_str().to_os_string(),
    )))
}

#[cfg(test)]
#[path = "caller_tests.rs"]
mod tests;
