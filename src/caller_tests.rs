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

#[test]
fn caller_directive_emits_env_entry() {
    let tmp = tempfile::tempdir().unwrap();
    let parsed = parse_at(
        tmp.path(),
        "[env]\n# @makerz = \"caller\"\nCALLER_DIR = \".\"\n",
    );
    let caller_cwd = tmp.path().join("sub");
    std::fs::create_dir(&caller_cwd).unwrap();

    let (key, value) = resolve_caller_env(&parsed, &caller_cwd).unwrap().unwrap();
    assert_eq!(key, "CALLER_DIR");
    assert_eq!(value, OsString::from(&caller_cwd));
}

#[test]
fn wrong_var_name_for_caller_is_error() {
    let tmp = tempfile::tempdir().unwrap();
    let parsed = parse_at(
        tmp.path(),
        "[env]\n# @makerz = \"caller\"\nNOT_CALLER_DIR = \".\"\n",
    );
    let err = resolve_caller_env(&parsed, tmp.path()).unwrap_err();
    assert!(
        matches!(
            err,
            CallerError::VarNameMismatch { expected, ref actual }
            if expected == "CALLER_DIR" && actual == "NOT_CALLER_DIR"
        ),
        "got {err:?}"
    );
}
