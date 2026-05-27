#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

/// Copy `tests/fixtures/<name>/` recursively into a fresh tempdir.
pub fn copy_fixture(name: &str) -> TempDir {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let dst = tempfile::tempdir().expect("create tempdir");
    copy_recursive(&src, dst.path()).expect("copy fixture");
    dst
}

fn copy_recursive(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let dst = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_recursive(&entry.path(), &dst)?;
        } else {
            fs::copy(entry.path(), &dst)?;
        }
    }
    Ok(())
}

/// Spawn `makerz` (the binary built for tests) with given args at `cwd`.
pub fn run_makerz(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_makerz"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("spawn makerz")
}
