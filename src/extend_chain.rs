//! Walk a Makefile's top-level `extend` chain.
//!
//! Produces an ordered list of [`ParsedMakefile`] (ancestor → leaf) for the
//! active Makefile, validating that every referenced file exists and that no
//! cycle is present. Scope-limit extend forms (`extend = [...]`, `relative`,
//! `optional`) surface as parse errors propagated from `directive_parser`.
//!
//! No semantic interpretation is applied here; env resolution is the next
//! layer up.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::directive_parser::{ParseMakefileError, ParsedMakefile, parse as parse_makefile};
use crate::location::MakefileLocation;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("failed to parse {}: {source}", location.file().display())]
    Parse {
        location: MakefileLocation,
        #[source]
        source: ParseMakefileError,
    },

    #[error(
        "extend target {} (referenced from {}) does not exist: {source}",
        target.display(),
        from.display()
    )]
    ExtendNotFound {
        target: PathBuf,
        from: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("extend chain cycle detected at {}", cycle_at.display())]
    Cycle { cycle_at: PathBuf },
}

#[allow(dead_code)]
pub fn build_chain(start: MakefileLocation) -> Result<Vec<ParsedMakefile>, ChainError> {
    let mut chain = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut current = start;

    loop {
        let parsed = parse_makefile(current.clone()).map_err(|source| ChainError::Parse {
            location: current.clone(),
            source,
        })?;
        let canon = fs::canonicalize(current.file())
            .expect("Makefile.toml is canonicalizable after a successful parse");
        if !seen.insert(canon.clone()) {
            return Err(ChainError::Cycle { cycle_at: canon });
        }
        let next_rel = parsed.extend().map(str::to_string);
        chain.push(parsed);

        let Some(rel) = next_rel else {
            break;
        };
        let target = current.dir().join(&rel);
        let canon_target = canonicalize_extend_target(&target, &current.file())?;
        let next_dir = canon_target
            .parent()
            .expect("canonicalized Makefile.toml has a parent directory")
            .to_path_buf();
        current = MakefileLocation::new(next_dir);
    }

    chain.reverse();
    Ok(chain)
}

fn canonicalize_extend_target(target: &Path, from: &Path) -> Result<PathBuf, ChainError> {
    fs::canonicalize(target).map_err(|source| ChainError::ExtendNotFound {
        target: target.to_path_buf(),
        from: from.to_path_buf(),
        source,
    })
}

#[cfg(test)]
#[path = "extend_chain_tests.rs"]
mod tests;
