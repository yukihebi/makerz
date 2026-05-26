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
    assert!(resolve_caller_env(&parsed, &caller_cwd).is_none());
}

#[test]
fn caller_directive_emits_env_entry() {
    let tmp = tempfile::tempdir().unwrap();
    let parsed = parse_at(
        tmp.path(),
        "[env]\n# @makerz = \"caller\"\nCALLER_DIR = \".\"\n",
    );
    let caller_cwd = tmp.path().join("sub");

    let (key, value) = resolve_caller_env(&parsed, &caller_cwd).unwrap();
    assert_eq!(key, "CALLER_DIR");
    assert_eq!(value, OsString::from(&caller_cwd));
}

#[test]
fn file_directive_alongside_caller_is_silently_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let parsed = parse_at(
        tmp.path(),
        "[config]\nskip_core_tasks = true\n\n\
         [env]\n# @makerz = \"file\"\nDEMO_DIR = \".\"\n\
         # @makerz = \"caller\"\nCALLER_DIR = \".\"\n\n\
         [tasks.default]\ncwd = \"${DEMO_DIR}\"\nscript = \"echo ${CALLER_DIR}\"\n",
    );
    let caller_cwd = tmp.path().join("from-here");

    let (key, value) = resolve_caller_env(&parsed, &caller_cwd).unwrap();
    assert_eq!(key, "CALLER_DIR");
    assert_eq!(value, OsString::from(&caller_cwd));
}

#[test]
fn user_chosen_var_name_is_emitted_as_is() {
    let tmp = tempfile::tempdir().unwrap();
    let parsed = parse_at(
        tmp.path(),
        "[env]\n# @makerz = \"caller\"\nMY_CWD = \".\"\n",
    );
    let caller_cwd = tmp.path().join("from-here");

    let (key, value) = resolve_caller_env(&parsed, &caller_cwd).unwrap();
    assert_eq!(key, "MY_CWD");
    assert_eq!(value, OsString::from(&caller_cwd));
}
