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

    #[error("top-level `extend` must be a string (got {kind})")]
    ExtendNotString { kind: &'static str },
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
    let doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|source| ParseMakefileError::TomlParse {
            path: path.clone(),
            source,
        })?;
    let extend = parse_extend(&doc)?;
    let env = parse_env(&content, &doc)?;
    Ok(ParsedMakefile {
        location,
        env,
        extend,
    })
}

fn parse_env(content: &str, doc: &toml_edit::DocumentMut) -> Result<ParsedEnv, ParseMakefileError> {
    let mut state = EnvScan::new(doc);
    for line in content.lines() {
        state.feed(line)?;
    }
    state.finish()
}

/// Per-line scanner. Tracks current section + pending directives and builds
/// the [`ParsedEnv`].
struct EnvScan<'a> {
    doc: &'a toml_edit::DocumentMut,
    current_section: Option<String>,
    pending: Vec<Directive>,
    env: ParsedEnv,
}

#[derive(Debug, Clone, Copy)]
enum Directive {
    File,
    Inherit,
    Caller,
}

impl<'a> EnvScan<'a> {
    fn new(doc: &'a toml_edit::DocumentMut) -> Self {
        Self {
            doc,
            current_section: None,
            pending: Vec::new(),
            env: ParsedEnv::default(),
        }
    }

    fn feed(&mut self, line: &str) -> Result<(), ParseMakefileError> {
        if let Some(section) = match_section_header(line) {
            self.current_section = Some(section);
            return Ok(());
        }
        if let Some(value) = match_directive_comment(line) {
            return self.consume_directive(&value);
        }
        if self.in_env()
            && let Some(key) = match_bare_key(line)
        {
            return self.bind_pending_to(&key);
        }
        Ok(())
    }

    fn finish(self) -> Result<ParsedEnv, ParseMakefileError> {
        Ok(self.env)
    }

    fn in_env(&self) -> bool {
        self.current_section.as_deref() == Some("env")
    }

    fn consume_directive(&mut self, value: &str) -> Result<(), ParseMakefileError> {
        let directive = match value {
            "file" => Directive::File,
            "inherit" => Directive::Inherit,
            "caller" => Directive::Caller,
            _ => return Ok(()),
        };
        if self.in_env() {
            self.pending.push(directive);
        }
        Ok(())
    }

    fn bind_pending_to(&mut self, key: &str) -> Result<(), ParseMakefileError> {
        let pending = std::mem::take(&mut self.pending);
        match pending.as_slice() {
            [] => {
                self.env.plain_keys.push(key.to_string());
            }
            [directive] => {
                let fallback = lookup_env_fallback(self.doc, key);
                let binding = EnvBinding {
                    name: key.to_string(),
                    fallback,
                };
                match directive {
                    Directive::File => self.env.file = Some(binding),
                    Directive::Caller => self.env.caller = Some(binding),
                    Directive::Inherit => self.env.inherit.push(binding),
                }
            }
            _ => {
                // Multiple directives on the same key — handled in Task 6.
            }
        }
        Ok(())
    }
}

fn lookup_env_fallback(doc: &toml_edit::DocumentMut, key: &str) -> String {
    doc.get("env")
        .and_then(|i| i.as_table())
        .and_then(|t| t.get(key))
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .to_string()
}

fn match_section_header(line: &str) -> Option<String> {
    let l = line.trim_start();
    let l = l.strip_prefix('[')?;
    let l = l.strip_prefix('[').unwrap_or(l);
    let end = l.find(']')?;
    Some(l[..end].trim().to_string())
}

fn match_directive_comment(line: &str) -> Option<String> {
    let l = line.trim_start();
    let l = l.strip_prefix('#')?;
    let l = l.trim_start();
    let l = l.strip_prefix("@makerz")?;
    let l = l.trim_start();
    let l = l.strip_prefix('=')?;
    let l = l.trim_start();
    let l = l.strip_prefix('"')?;
    let end = l.find('"')?;
    let value = l[..end].to_string();
    let rest = l[end + 1..].trim_start();
    if !rest.is_empty() && !rest.starts_with('#') {
        return None;
    }
    Some(value)
}

fn match_bare_key(line: &str) -> Option<String> {
    let l = line.trim_start();
    let first = l.chars().next()?;
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return None;
    }
    let end = l
        .char_indices()
        .take_while(|(_, c)| *c == '_' || c.is_ascii_alphanumeric())
        .map(|(i, c)| i + c.len_utf8())
        .last()?;
    let name = &l[..end];
    let rest = l[end..].trim_start();
    if !rest.starts_with('=') {
        return None;
    }
    Some(name.to_string())
}

fn parse_extend(doc: &toml_edit::DocumentMut) -> Result<Option<String>, ParseMakefileError> {
    let Some(item) = doc.get("extend") else {
        return Ok(None);
    };
    if let Some(s) = item.as_str() {
        return Ok(Some(s.to_string()));
    }
    Err(ParseMakefileError::ExtendNotString {
        kind: item_kind_label(item),
    })
}

fn item_kind_label(item: &toml_edit::Item) -> &'static str {
    use toml_edit::{Item, Value};
    match item {
        Item::None => "none",
        Item::Value(Value::String(_)) => "string",
        Item::Value(Value::Integer(_)) => "integer",
        Item::Value(Value::Float(_)) => "float",
        Item::Value(Value::Boolean(_)) => "boolean",
        Item::Value(Value::Datetime(_)) => "datetime",
        Item::Value(Value::Array(_)) => "array",
        Item::Value(Value::InlineTable(_)) => "inline-table",
        Item::Table(_) => "table",
        Item::ArrayOfTables(_) => "array-of-tables",
    }
}

#[cfg(test)]
#[path = "directive_parser_tests.rs"]
mod tests;
