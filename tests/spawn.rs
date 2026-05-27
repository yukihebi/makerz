#![cfg(feature = "with-makers")]

mod common;

use common::{copy_fixture, run_makerz};

#[test]
fn s1_nearest_ancestor_discovery() {
    let project = copy_fixture("discovery-nested");
    let cwd = project.path().join("root/parent/sub/sub");
    let output = run_makerz(&cwd, &["--quiet", "my-dir"]);
    assert!(
        output.status.success(),
        "makerz exited non-zero. stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "from-parent");
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr, got: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn s2_caller_injection_from_subdir() {
    let project = copy_fixture("caller-basic");
    let isolated = tempfile::tempdir().expect("isolated tempdir");
    let run_cwd = project.path().join("sub");
    let env_arg = format!("ISOLATED={}", isolated.path().display());
    let output = run_makerz(&run_cwd, &["--quiet", "--env", &env_arg, "verify-cwd"]);
    assert!(
        output.status.success(),
        "makerz exited non-zero. stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "MAKERZ-S2-MARKER");
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr, got: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn s3_caller_injection_from_makefile_dir() {
    let project = copy_fixture("caller-basic");
    let isolated = tempfile::tempdir().expect("isolated tempdir");
    let env_arg = format!("ISOLATED={}", isolated.path().display());
    let output = run_makerz(
        project.path(),
        &["--quiet", "--env", &env_arg, "verify-cwd"],
    );
    assert!(
        output.status.success(),
        "makerz exited non-zero. stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "MAKERZ-S3-MARKER");
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr, got: {}",
        String::from_utf8_lossy(&output.stderr),
    );
}
