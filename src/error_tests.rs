use super::*;
use std::io;

#[test]
fn arg_parse_display_includes_message() {
    let err = Error::ArgParse("missing value for --extend".into());
    assert_eq!(
        err.to_string(),
        "argument error: missing value for --extend"
    );
}

#[test]
fn makers_not_found_display_is_friendly() {
    let msg = Error::MakersNotFound.to_string();
    assert!(msg.contains("not found"));
    assert!(msg.contains("cargo-make"));
}

#[test]
fn makers_spawn_display_wraps_io_error() {
    let io = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let msg = Error::MakersSpawn(io).to_string();
    assert!(msg.contains("spawn"));
    assert!(msg.contains("denied"));
}

#[test]
fn makers_failed_with_code_display() {
    let msg = Error::MakersFailed { code: Some(42) }.to_string();
    assert!(msg.contains("42"));
}

#[test]
fn makers_failed_signal_display() {
    let msg = Error::MakersFailed { code: None }.to_string();
    assert!(msg.contains("signal"));
}

#[test]
fn exit_code_arg_parse_is_2() {
    assert_eq!(Error::ArgParse("x".into()).exit_code(), 2);
}

#[test]
fn exit_code_makers_not_found_is_127() {
    assert_eq!(Error::MakersNotFound.exit_code(), 127);
}

#[test]
fn exit_code_forwards_makers_failed_code() {
    assert_eq!(Error::MakersFailed { code: Some(7) }.exit_code(), 7);
    assert_eq!(Error::MakersFailed { code: None }.exit_code(), 1);
}
