// Removed in Task 7 (main.rs wiring) once `parse` is called from the binary.
#![allow(dead_code)]

//! Parse a single `Makefile.toml` into structured directive info.
//!
//! Wired into the binary in this PR only to surface errors early; the parsed
//! data is consumed by later PRs (caller-injection, env-resolution).

use std::fs;
use std::path::PathBuf;

use thiserror::Error;

use crate::discovery::MakefileLocation;

#[derive(Debug, Error)]
pub enum ParseMakefileError {
    #[error("failed to read {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("TOML parse error in {}: {source}", path.display())]
    TomlParse {
        path: PathBuf,
        #[source]
        source: toml_edit::TomlError,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedMakefile {
    location: MakefileLocation,
    env: ParsedEnv,
    extend: Option<String>,
}

impl ParsedMakefile {
    pub fn location(&self) -> &MakefileLocation {
        &self.location
    }
    pub fn env(&self) -> &ParsedEnv {
        &self.env
    }
    pub fn extend(&self) -> Option<&str> {
        self.extend.as_deref()
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ParsedEnv {
    file: Option<EnvBinding>,
    caller: Option<EnvBinding>,
    inherit: Vec<EnvBinding>,
    plain_keys: Vec<String>,
}

impl ParsedEnv {
    pub fn file(&self) -> Option<&EnvBinding> {
        self.file.as_ref()
    }
    pub fn caller(&self) -> Option<&EnvBinding> {
        self.caller.as_ref()
    }
    pub fn inherit(&self) -> &[EnvBinding] {
        &self.inherit
    }
    pub fn plain_keys(&self) -> &[String] {
        &self.plain_keys
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnvBinding {
    name: String,
    fallback: String,
}

impl EnvBinding {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
}

pub fn parse(location: MakefileLocation) -> Result<ParsedMakefile, ParseMakefileError> {
    let path = location.file();
    let content = fs::read_to_string(&path).map_err(|source| ParseMakefileError::Read {
        path: path.clone(),
        source,
    })?;
    let _doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|source| ParseMakefileError::TomlParse {
            path: path.clone(),
            source,
        })?;
    Ok(ParsedMakefile {
        location,
        env: ParsedEnv::default(),
        extend: None,
    })
}

#[cfg(test)]
#[path = "directive_parser_tests.rs"]
mod tests;
