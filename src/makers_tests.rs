use std::path::Path;

use super::*;
use crate::error::Error;

#[test]
fn build_args_prepends_cwd_with_empty_passthrough() {
    assert_eq!(
        build_args(Path::new("/abs/dir"), &[]),
        vec!["--cwd".to_string(), "/abs/dir".to_string()]
    );
}

#[test]
fn build_args_prepends_cwd_then_passthrough() {
    assert_eq!(
        build_args(Path::new("/abs/dir"), &["task".into(), "--flag".into()]),
        vec![
            "--cwd".to_string(),
            "/abs/dir".to_string(),
            "task".into(),
            "--flag".into(),
        ]
    );
}

#[test]
fn spawn_missing_binary_returns_makers_not_found() {
    let err = spawn("this-binary-must-not-exist-makerz-test-xyz123", &[]).unwrap_err();
    assert!(matches!(err, Error::MakersNotFound), "got {err:?}");
}

#[test]
fn spawn_zero_exit_is_ok() {
    run_with_exit_code(0).expect("shell exit 0 should succeed");
}

#[test]
fn spawn_forwards_exit_code() {
    let err = run_with_exit_code(42).unwrap_err();
    assert!(matches!(err, Error::MakersExited(42)), "got {err:?}");
}

/// Spawn the platform's default shell with a command that exits with `code`.
fn run_with_exit_code(code: i32) -> Result<(), Error> {
    let (binary, args) = shell_exit_command(code);
    spawn(binary, &args)
}

#[cfg(unix)]
fn shell_exit_command(code: i32) -> (&'static str, Vec<String>) {
    ("sh", vec!["-c".into(), format!("exit {code}")])
}

#[cfg(windows)]
fn shell_exit_command(code: i32) -> (&'static str, Vec<String>) {
    ("cmd", vec!["/C".into(), format!("exit {code}")])
}
