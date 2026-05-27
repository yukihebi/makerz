use std::fs;
use std::path::Path;

use tempfile::tempdir;

use super::*;
use crate::location::MakefileLocation;

fn write_makefile(dir: &Path, content: &str) -> MakefileLocation {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("Makefile.toml"), content).unwrap();
    MakefileLocation::new(dir.to_path_buf())
}

#[test]
fn single_makefile_no_extend_returns_self() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\nFOO = \"x\"\n");
    let chain = build_chain(loc).unwrap();
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].env().plain_keys(), &["FOO"]);
}
