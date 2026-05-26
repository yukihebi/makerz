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
