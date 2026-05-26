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
