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

/// Directory + canonical filename of a `Makefile.toml`.
///
/// Constructed by [`find_makefile`] or directly via [`MakefileLocation::new`].
/// The latter lets callers construct a location from a known directory — for
/// example when walking an extend chain to resolve parent Makefiles.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MakefileLocation {
    dir: PathBuf,
}

impl MakefileLocation {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn file(&self) -> PathBuf {
        self.dir.join("Makefile.toml")
    }
}

/// Walk up from `start` looking for `Makefile.toml`.
///
/// Stops at the filesystem root: returns [`DiscoveryError::NotFound`] when no
/// `Makefile.toml` exists on the chain from `start` to `/`.
pub fn find_makefile(start: &Path) -> Result<MakefileLocation, DiscoveryError> {
    for dir in start.ancestors() {
        if dir.join("Makefile.toml").is_file() {
            return Ok(MakefileLocation::new(dir.to_path_buf()));
        }
    }
    Err(DiscoveryError::NotFound {
        start: start.to_path_buf(),
    })
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
