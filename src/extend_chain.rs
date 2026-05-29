//! Walk a Makefile's top-level `extend` chain.
//!
//! Builds an ordered list of [`ParsedMakefile`] (ancestor → leaf), validating
//! that every referenced file exists and that the chain has no cycle.
//! Unsupported extend forms (`extend = [...]`, `relative`, `optional`) surface
//! as [`ChainError::Parse`] from `directive_parser`.

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
        let (parsed, canon) = load_node(&current)?;
        if !seen.insert(canon.clone()) {
            return Err(ChainError::Cycle { cycle_at: canon });
        }
        let next = next_location(&current, parsed.extend())?;
        chain.push(parsed);
        match next {
            Some(loc) => current = loc,
            None => break,
        }
    }

    chain.reverse();
    Ok(chain)
}

/// Parse the Makefile at `loc` and pair it with its canonical path.
///
/// Canonicalizing cannot fail here: a successful parse means the file was
/// read, so it exists.
fn load_node(loc: &MakefileLocation) -> Result<(ParsedMakefile, PathBuf), ChainError> {
    let parsed = parse_makefile(loc.clone()).map_err(|source| ChainError::Parse {
        location: loc.clone(),
        source,
    })?;
    let canon = fs::canonicalize(loc.file())
        .expect("Makefile.toml is canonicalizable after a successful parse");
    Ok((parsed, canon))
}

fn next_location(
    current: &MakefileLocation,
    extend: Option<&str>,
) -> Result<Option<MakefileLocation>, ChainError> {
    let Some(rel) = extend else {
        return Ok(None);
    };
    let target = current.dir().join(rel);
    let canon_target = canonicalize_extend_target(&target, &current.file())?;
    let next_dir = canon_target
        .parent()
        .expect("canonicalized Makefile.toml has a parent directory")
        .to_path_buf();
    Ok(Some(MakefileLocation::new(next_dir)))
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
