use std::fs;

use tempfile::tempdir;

use super::*;
use crate::discovery::MakefileLocation;

fn write_makefile(dir: &std::path::Path, content: &str) -> MakefileLocation {
    fs::write(dir.join("Makefile.toml"), content).unwrap();
    MakefileLocation::new(dir.to_path_buf())
}

#[test]
fn invalid_toml_returns_toml_parse_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "this is = = not toml");

    let err = parse(loc).unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::TomlParse { .. }),
        "got {err:?}"
    );
}

#[test]
fn no_extend_returns_none() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\n");
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.extend(), None);
}

#[test]
fn string_extend_returns_some() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "extend = \"../parent/Makefile.toml\"\n");
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.extend(), Some("../parent/Makefile.toml"));
}

#[test]
fn array_extend_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "extend = [\"a\", \"b\"]\n");
    let err = parse(loc).unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::ExtendNotString { .. }),
        "got {err:?}"
    );
}

#[test]
fn table_extend_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "extend = { path = \"../p/Makefile.toml\" }\n");
    let err = parse(loc).unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::ExtendNotString { .. }),
        "got {err:?}"
    );
}

#[test]
fn empty_file_has_empty_env() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "");
    let parsed = parse(loc).unwrap();
    assert!(parsed.env().plain_keys().is_empty());
    assert!(parsed.env().file().is_none());
    assert!(parsed.env().caller().is_none());
    assert!(parsed.env().inherit().is_empty());
}

#[test]
fn env_keys_collected_in_order_as_plain() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\nFOO = \"a\"\nBAR = \"b\"\nBAZ = \"c\"\n");
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.env().plain_keys(), &["FOO", "BAR", "BAZ"]);
}

#[test]
fn no_env_section_means_empty_env() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[tasks.default]\nscript = \"echo hi\"\n");
    let parsed = parse(loc).unwrap();
    assert!(parsed.env().plain_keys().is_empty());
}
