//! Walk up from a directory looking for `Makefile.toml`.

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Reasons `find_makefile` can fail.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// No `Makefile.toml` was found between `start` (inclusive) and the
    /// filesystem root.
    #[error("no Makefile.toml found from {} upward to filesystem root", .start.display())]
    NotFound { start: PathBuf },
}

/// Walk up from `start` looking for `Makefile.toml`. Returns the full path to
/// the discovered file (not just the directory).
///
/// Stops at the filesystem root: returns [`DiscoveryError::NotFound`] when no
/// `Makefile.toml` exists on the chain from `start` to `/`.
pub fn find_makefile(start: &Path) -> Result<PathBuf, DiscoveryError> {
    for dir in start.ancestors() {
        let candidate = dir.join("Makefile.toml");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(DiscoveryError::NotFound {
        start: start.to_path_buf(),
    })
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
