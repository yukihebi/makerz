use super::*;
use crate::error::Error;

#[test]
fn build_args_is_identity_for_passthrough() {
    assert_eq!(build_args(&[]), Vec::<String>::new());
    assert_eq!(
        build_args(&["task".into(), "--flag".into()]),
        vec!["task".to_string(), "--flag".into()]
    );
}

#[test]
fn spawn_missing_binary_returns_makers_not_found() {
    let err = spawn("this-binary-must-not-exist-makerz-test-xyz123", &[]).unwrap_err();
    assert!(matches!(err, Error::MakersNotFound), "got {err:?}");
}

#[test]
fn spawn_zero_exit_is_ok() {
    spawn("true", &[]).expect("/usr/bin/true should exit 0");
}

#[test]
fn spawn_nonzero_exit_returns_makers_exited() {
    let err = spawn("false", &[]).unwrap_err();
    match err {
        Error::MakersExited(c) => assert_ne!(c, 0),
        other => panic!("expected MakersExited, got {other:?}"),
    }
}

#[test]
fn spawn_forwards_arbitrary_exit_code() {
    let err = spawn("sh", &["-c".into(), "exit 42".into()]).unwrap_err();
    assert!(matches!(err, Error::MakersExited(42)), "got {err:?}");
}
