//! Locate `Makefile.toml` on disk.

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Reasons [`MakefileLocation::find`] can fail.
#[derive(Debug, Error)]
pub enum FindError {
    /// No `Makefile.toml` was found between `start` (inclusive) and the
    /// filesystem root.
    #[error("no Makefile.toml found from {} upward to filesystem root", .start.display())]
    NotFound { start: PathBuf },
}

/// Directory + canonical filename of a `Makefile.toml`.
///
/// Constructed by [`MakefileLocation::find`] (upward search) or directly
/// via [`MakefileLocation::new`] from a known directory.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MakefileLocation {
    dir: PathBuf,
}

impl MakefileLocation {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Walk up from `start` looking for `Makefile.toml`.
    ///
    /// Stops at the filesystem root: returns [`FindError::NotFound`] when no
    /// `Makefile.toml` exists on the chain from `start` to `/`.
    pub fn find(start: &Path) -> Result<Self, FindError> {
        for dir in start.ancestors() {
            if dir.join("Makefile.toml").is_file() {
                return Ok(Self::new(dir.to_path_buf()));
            }
        }
        Err(FindError::NotFound {
            start: start.to_path_buf(),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn file(&self) -> PathBuf {
        self.dir.join("Makefile.toml")
    }
}

#[cfg(test)]
#[path = "location_tests.rs"]
mod tests;
