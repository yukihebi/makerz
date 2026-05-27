mod common;

use common::run_makerz;
use tempfile::tempdir;

#[test]
fn version_first_line_matches_cargo_version() {
    let dir = tempdir().expect("tempdir");
    let output = run_makerz(dir.path(), &["--version"]);
    assert!(
        output.status.success(),
        "makerz --version exited non-zero. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let first_line = stdout.lines().next().expect("at least one line in stdout");
    let expected = format!("makerz {}", env!("CARGO_PKG_VERSION"));
    assert_eq!(first_line, expected);
}
