use std::path::PathBuf;

use super::*;
use crate::directive_parser;
use crate::location::MakefileLocation;
use crate::makers::EnvEntry;

fn parse_at(dir: &std::path::Path, content: &str) -> directive_parser::ParsedMakefile {
    std::fs::write(dir.join("Makefile.toml"), content).unwrap();
    directive_parser::parse(MakefileLocation::new(dir.to_path_buf())).unwrap()
}

fn absolutize(path: &std::path::Path) -> std::path::PathBuf {
    std::path::absolute(path).unwrap()
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

    assert_eq!(
        resolve_caller_env(&parsed, &caller_cwd).unwrap(),
        Some(EnvEntry::new("CALLER_DIR", absolutize(&caller_cwd))),
    );
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

    assert_eq!(
        resolve_caller_env(&parsed, &caller_cwd).unwrap(),
        Some(EnvEntry::new("CALLER_DIR", absolutize(&caller_cwd))),
    );
}

#[test]
fn user_chosen_var_name_is_emitted_as_is() {
    let tmp = tempfile::tempdir().unwrap();
    let parsed = parse_at(
        tmp.path(),
        "[env]\n# @makerz = \"caller\"\nMY_CWD = \".\"\n",
    );
    let caller_cwd = tmp.path().join("from-here");

    assert_eq!(
        resolve_caller_env(&parsed, &caller_cwd).unwrap(),
        Some(EnvEntry::new("MY_CWD", absolutize(&caller_cwd))),
    );
}

#[test]
fn relative_caller_cwd_is_absolutized() {
    let tmp = tempfile::tempdir().unwrap();
    let parsed = parse_at(
        tmp.path(),
        "[env]\n# @makerz = \"caller\"\nCALLER_DIR = \".\"\n",
    );
    let relative = PathBuf::from("some/relative/path");

    let entry = resolve_caller_env(&parsed, &relative).unwrap().unwrap();
    let expected_value = std::path::absolute(&relative).unwrap();
    assert_eq!(entry, EnvEntry::new("CALLER_DIR", expected_value));
}
