#![cfg(feature = "with-makers")]

mod common;

use common::{copy_fixture, run_makerz};

// NOTE: `--quiet` in cargo-make 0.37 suppresses the task's own stdout in
// addition to the INFO log lines, so we omit it here and assert that the
// expected task output appears somewhere in stdout (INFO logs go to stdout
// too; stderr is always empty on a clean run).
#[test]
fn s1_nearest_ancestor_discovery() {
    let project = copy_fixture("discovery-nested");
    let cwd = project.path().join("root/parent/sub/sub");
    let output = run_makerz(&cwd, &["my-dir"]);
    assert!(
        output.status.success(),
        "makerz exited non-zero. stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("from-parent"),
        "expected 'from-parent' in stdout, got: {stdout}",
    );
    assert!(
        !stdout.contains("from-root"),
        "expected nearest ancestor (parent/) to win, but 'from-root' appeared: {stdout}",
    );
    assert_eq!(output.stderr, b"");
}
