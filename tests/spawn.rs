#![cfg(feature = "with-makers")]

mod common;

use common::{copy_fixture, run_makerz};

/// Filter out cargo-make's `[cargo-make]`-prefixed INFO log lines from stdout,
/// leaving only the task's own output. cargo-make 0.37 routes both INFO logs
/// and task stdout to the same stream and suppresses both under `--quiet`,
/// so we run verbose and filter at assertion time.
fn task_output(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter(|line| !line.starts_with("[cargo-make]"))
        .collect::<Vec<_>>()
        .join("\n")
}

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
    assert_eq!(task_output(&output.stdout), "from-parent");
    assert!(output.stderr.is_empty());
}

#[test]
fn s2_caller_injection_from_subdir() {
    let project = copy_fixture("caller-basic");
    let isolated = tempfile::tempdir().expect("isolated tempdir");
    let run_cwd = project.path().join("sub");
    let env_arg = format!("ISOLATED={}", isolated.path().display());
    let output = run_makerz(&run_cwd, &["--env", &env_arg, "verify-cwd"]);
    assert!(
        output.status.success(),
        "makerz exited non-zero. stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(task_output(&output.stdout).trim(), "MAKERZ-S2-MARKER");
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr, got: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}
