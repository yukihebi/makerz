//! Resolve the `caller` directive on the active `Makefile.toml`.
//!
//! Validates that the caller binding (if any) uses the canonical variable
//! name and that its fallback path exists, then emits the `--env` entry
//! that records the makerz caller's true cwd for use by `makers` tasks.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::directive_parser::ParsedMakefile;

/// The only var name a `caller` directive may bind.
pub const CALLER_VAR: &str = "CALLER_DIR";

#[derive(Debug, Error)]
pub enum CallerError {
    #[error("`caller` directive must bind variable `{expected}` (got `{actual}`)")]
    VarNameMismatch {
        expected: &'static str,
        actual: String,
    },

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
    if binding.name() != CALLER_VAR {
        return Err(CallerError::VarNameMismatch {
            expected: CALLER_VAR,
            actual: binding.name().to_string(),
        });
    }
    let resolved = parsed.location().dir().join(binding.fallback());
    if !resolved.exists() {
        return Err(CallerError::FallbackPathMissing { path: resolved });
    }
    Ok(Some((
        CALLER_VAR.to_string(),
        caller_cwd.as_os_str().to_os_string(),
    )))
}

#[cfg(test)]
#[path = "caller_tests.rs"]
mod tests;
