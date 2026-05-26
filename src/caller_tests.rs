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
fn missing_fallback_path_is_error() {
    let tmp = tempfile::tempdir().unwrap();
    let parsed = parse_at(
        tmp.path(),
        "[env]\n# @makerz = \"caller\"\nCALLER_DIR = \"does/not/exist\"\n",
    );
    let err = resolve_caller_env(&parsed, tmp.path()).unwrap_err();
    let expected_missing = tmp.path().join("does/not/exist");
    assert!(
        matches!(
            err,
            CallerError::FallbackPathMissing { ref path } if path == &expected_missing
        ),
        "got {err:?}"
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
    std::fs::create_dir(&caller_cwd).unwrap();

    let (key, value) = resolve_caller_env(&parsed, &caller_cwd).unwrap().unwrap();
    assert_eq!(key, "CALLER_DIR");
    assert_eq!(value, OsString::from(&caller_cwd));
}

#[test]
fn relative_fallback_resolves_against_makefile_dir() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("nested")).unwrap();
    let parsed = parse_at(
        tmp.path(),
        "[env]\n# @makerz = \"caller\"\nCALLER_DIR = \"nested\"\n",
    );
    let caller_cwd = tmp.path().join("nested");
    let (key, value) = resolve_caller_env(&parsed, &caller_cwd).unwrap().unwrap();
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
    std::fs::create_dir(&caller_cwd).unwrap();

    let (key, value) = resolve_caller_env(&parsed, &caller_cwd).unwrap().unwrap();
    assert_eq!(key, "MY_CWD");
    assert_eq!(value, OsString::from(&caller_cwd));
}
