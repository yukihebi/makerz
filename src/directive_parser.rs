//! Parse a single `Makefile.toml` into structured directive info.
//!
//! Run on every invocation so directive errors surface before `makers` is
//! launched.

use std::fs;
use std::path::PathBuf;

use thiserror::Error;

use crate::location::MakefileLocation;

#[derive(Debug, Error)]
pub enum ParseMakefileError {
    #[error("failed to read {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml_edit::TomlError),

    #[error("top-level `extend` must be a string (got {kind})")]
    ExtendNotString { kind: &'static str },

    #[error("makerz directive `# @makerz = \"{value}\"` is not in any [env] section")]
    DirectiveOutsideEnv { value: String },

    #[error("makerz directive `# @makerz = \"{value}\"` is not bound to any env key")]
    DirectiveUnbound { value: String },

    #[error("multiple makerz directives bind to env key `{name}`")]
    DirectiveOnSameVar { name: String },

    #[error(
        "multiple `file` directives in the same Makefile (previous: `{previous}`, new: `{new}`)"
    )]
    DuplicateFile { previous: String, new: String },

    #[error(
        "multiple `caller` directives in the same Makefile (previous: `{previous}`, new: `{new}`)"
    )]
    DuplicateCaller { previous: String, new: String },

    #[error(
        "unknown makerz directive value `{value}` (expected \"file\", \"inherit\", or \"caller\")"
    )]
    DirectiveUnknownValue { value: String },

    #[error("env key `{name}` has a makerz directive but its fallback value is not a string")]
    FallbackNotString { name: String },

    #[error("malformed makerz directive: `{line}` (expected `# @makerz = \"<value>\"`)")]
    DirectiveMalformed { line: String },
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedMakefile {
    location: MakefileLocation,
    env: ParsedEnv,
    extend: Option<String>,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
    let (env, extend) = parse_content(&content)?;
    Ok(ParsedMakefile {
        location,
        env,
        extend,
    })
}

pub fn parse_content(content: &str) -> Result<(ParsedEnv, Option<String>), ParseMakefileError> {
    let doc = content.parse::<toml_edit::DocumentMut>()?;
    let extend = parse_extend(&doc)?;
    let env = parse_env(content, &doc)?;
    Ok((env, extend))
}

fn parse_env(content: &str, doc: &toml_edit::DocumentMut) -> Result<ParsedEnv, ParseMakefileError> {
    let mut state = EnvScan::new(doc);
    for line in content.lines() {
        state.feed(line)?;
    }
    state.finish()
}

#[derive(Debug, Clone, Copy)]
enum Directive {
    File,
    Inherit,
    Caller,
}

struct PendingDirective {
    directive: Directive,
    raw_value: String,
}

/// Per-line scanner. Tracks current section + pending directives and builds
/// the [`ParsedEnv`].
struct EnvScan<'a> {
    doc: &'a toml_edit::DocumentMut,
    current_section: Option<String>,
    pending: Vec<PendingDirective>,
    env: ParsedEnv,
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
            self.flush_unbound_on_section_change()?;
            self.current_section = Some(section);
            return Ok(());
        }
        if let Some(value) = match_directive_comment(line)? {
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
        if self.in_env()
            && let Some(first) = self.pending.into_iter().next()
        {
            return Err(ParseMakefileError::DirectiveUnbound {
                value: first.raw_value,
            });
        }
        Ok(self.env)
    }

    fn in_env(&self) -> bool {
        self.current_section.as_deref() == Some("env")
    }

    fn flush_unbound_on_section_change(&mut self) -> Result<(), ParseMakefileError> {
        if self.in_env()
            && let Some(first) = self.pending.first()
        {
            return Err(ParseMakefileError::DirectiveUnbound {
                value: first.raw_value.clone(),
            });
        }
        Ok(())
    }

    fn consume_directive(&mut self, value: &str) -> Result<(), ParseMakefileError> {
        let directive = match value {
            "file" => Directive::File,
            "inherit" => Directive::Inherit,
            "caller" => Directive::Caller,
            other => {
                return Err(ParseMakefileError::DirectiveUnknownValue {
                    value: other.to_string(),
                });
            }
        };
        if !self.in_env() {
            return Err(ParseMakefileError::DirectiveOutsideEnv {
                value: value.to_string(),
            });
        }
        self.pending.push(PendingDirective {
            directive,
            raw_value: value.to_string(),
        });
        Ok(())
    }

    fn bind_pending_to(&mut self, key: &str) -> Result<(), ParseMakefileError> {
        let pending = std::mem::take(&mut self.pending);
        match pending.as_slice() {
            [] => {
                self.env.plain_keys.push(key.to_string());
                Ok(())
            }
            [only] => self.bind_single(only.directive, key),
            _ => Err(ParseMakefileError::DirectiveOnSameVar {
                name: key.to_string(),
            }),
        }
    }

    fn bind_single(&mut self, directive: Directive, key: &str) -> Result<(), ParseMakefileError> {
        let fallback = lookup_env_fallback(self.doc, key)?;
        let binding = EnvBinding {
            name: key.to_string(),
            fallback,
        };
        match directive {
            Directive::File => {
                if let Some(prev) = &self.env.file {
                    return Err(ParseMakefileError::DuplicateFile {
                        previous: prev.name.clone(),
                        new: binding.name,
                    });
                }
                self.env.file = Some(binding);
            }
            Directive::Caller => {
                if let Some(prev) = &self.env.caller {
                    return Err(ParseMakefileError::DuplicateCaller {
                        previous: prev.name.clone(),
                        new: binding.name,
                    });
                }
                self.env.caller = Some(binding);
            }
            Directive::Inherit => {
                self.env.inherit.push(binding);
            }
        }
        Ok(())
    }
}

fn lookup_env_fallback(
    doc: &toml_edit::DocumentMut,
    key: &str,
) -> Result<String, ParseMakefileError> {
    doc.get("env")
        .and_then(|i| i.as_table())
        .and_then(|t| t.get(key))
        .and_then(|i| i.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| ParseMakefileError::FallbackNotString {
            name: key.to_string(),
        })
}

fn match_section_header(line: &str) -> Option<String> {
    let l = line.trim_start();
    let l = l.strip_prefix('[')?;
    let l = l.strip_prefix('[').unwrap_or(l);
    let end = l.find(']')?;
    Some(l[..end].trim().to_string())
}

fn match_directive_comment(line: &str) -> Result<Option<String>, ParseMakefileError> {
    let trimmed = line.trim_start();
    let Some(after_hash) = trimmed.strip_prefix('#') else {
        return Ok(None);
    };
    let body = after_hash.trim_start();
    let Some(tail) = body.strip_prefix("@makerz") else {
        return Ok(None);
    };
    if !is_directive_token_boundary(tail) {
        return Ok(None);
    }
    parse_directive_tail(tail)
        .map(Some)
        .ok_or_else(|| ParseMakefileError::DirectiveMalformed {
            line: line.to_string(),
        })
}

fn is_directive_token_boundary(tail: &str) -> bool {
    match tail.chars().next() {
        None => true,
        Some(c) => c == '=' || c.is_whitespace(),
    }
}

fn parse_directive_tail(tail: &str) -> Option<String> {
    let s = tail.trim_start();
    let s = s.strip_prefix('=')?;
    let s = s.trim_start();
    let s = s.strip_prefix('"')?;
    let end = s.find('"')?;
    let value = s[..end].to_string();
    let rest = s[end + 1..].trim_start();
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
