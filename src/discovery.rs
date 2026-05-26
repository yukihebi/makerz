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

/// Outcome of a successful upward search.
#[derive(Debug, PartialEq, Eq)]
pub struct Discovered {
    dir: PathBuf,
}

impl Discovered {
    /// Directory containing the discovered `Makefile.toml`.
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

/// Walk up from `start` looking for `Makefile.toml`.
///
/// Stops at the filesystem root: returns [`DiscoveryError::NotFound`] when no
/// `Makefile.toml` exists on the chain from `start` to `/`.
pub fn find_makefile(start: &Path) -> Result<Discovered, DiscoveryError> {
    for dir in start.ancestors() {
        if dir.join("Makefile.toml").is_file() {
            return Ok(Discovered {
                dir: dir.to_path_buf(),
            });
        }
    }
    Err(DiscoveryError::NotFound {
        start: start.to_path_buf(),
    })
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
