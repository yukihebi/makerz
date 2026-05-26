use std::path::PathBuf;

use super::*;
use crate::directive_parser;
use crate::location::MakefileLocation;

fn parse_at(dir: &std::path::Path, content: &str) -> directive_parser::ParsedMakefile {
    std::fs::write(dir.join("Makefile.toml"), content).unwrap();
    directive_parser::parse(MakefileLocation::new(dir.to_path_buf())).unwrap()
}

#[test]
fn no_caller_directive_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    let parsed = parse_at(tmp.path(), "[env]\nFOO = \"x\"\n");
    let caller_cwd = PathBuf::from("/whatever");
    assert!(resolve_caller_env(&parsed, &caller_cwd).unwrap().is_none());
}
