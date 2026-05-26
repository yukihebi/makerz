use super::*;
use crate::cli::ParseError;

#[test]
fn exit_code_arg_parse_is_2() {
    assert_eq!(
        Error::ArgParse(ParseError::ExtendMissingValue).exit_code(),
        2
    );
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
