use std::io;
use std::path::PathBuf;

use super::*;
use crate::cli::ParseError;
use crate::directive_parser::ParseMakefileError;
use crate::location::FindError;

#[test]
fn exit_code_arg_parse_is_2() {
    assert_eq!(
        Error::ArgParse(ParseError::ExtendMissingValue).exit_code(),
        2
    );
}

#[test]
fn exit_code_find_is_1() {
    let err = Error::Find(FindError::NotFound {
        start: PathBuf::from("/nowhere"),
    });
    assert_eq!(err.exit_code(), 1);
}

#[test]
fn exit_code_parse_makefile_is_1() {
    let err = Error::ParseMakefile(ParseMakefileError::ExtendNotString { kind: "array" });
    assert_eq!(err.exit_code(), 1);
}

#[test]
fn exit_code_cwd_is_1() {
    let err = Error::Cwd(io::Error::other("denied"));
    assert_eq!(err.exit_code(), 1);
}

#[test]
fn exit_code_makers_not_found_is_127() {
    assert_eq!(Error::MakersNotFound.exit_code(), 127);
}

#[test]
fn exit_code_forwards_makers_exit_code() {
    assert_eq!(Error::MakersExited(7).exit_code(), 7);
}

#[test]
fn exit_code_for_signal_is_one() {
    assert_eq!(Error::MakersTerminatedBySignal.exit_code(), 1);
}

#[test]
fn exit_code_for_out_of_byte_range_falls_back_to_one() {
    assert_eq!(Error::MakersExited(300).exit_code(), 1);
    assert_eq!(Error::MakersExited(-1).exit_code(), 1);
}
