use super::*;

fn parse_strs(args: &[&str]) -> Result<Parsed, ParseError> {
    parse(args.iter().map(|s| (*s).to_string()).collect())
}

#[test]
fn empty_args_is_passthrough() {
    assert_eq!(
        parse_strs(&[]).unwrap(),
        Parsed::Passthrough { args: vec![] }
    );
}

#[test]
fn version_flag() {
    assert_eq!(parse_strs(&["--version"]).unwrap(), Parsed::Version);
}

#[test]
fn help_flag() {
    assert_eq!(parse_strs(&["--help"]).unwrap(), Parsed::Help);
}

#[test]
fn help_wins_over_version() {
    assert_eq!(parse_strs(&["--version", "--help"]).unwrap(), Parsed::Help);
    assert_eq!(parse_strs(&["--help", "--version"]).unwrap(), Parsed::Help);
}

#[test]
fn init_alone() {
    assert_eq!(
        parse_strs(&["--init"]).unwrap(),
        Parsed::Init { extend: None }
    );
}

#[test]
fn init_with_extend_spaced() {
    assert_eq!(
        parse_strs(&["--init", "--extend", "foo"]).unwrap(),
        Parsed::Init {
            extend: Some("foo".into())
        }
    );
}

#[test]
fn init_with_extend_equals() {
    assert_eq!(
        parse_strs(&["--init", "--extend=foo"]).unwrap(),
        Parsed::Init {
            extend: Some("foo".into())
        }
    );
}

#[test]
fn extend_order_does_not_matter() {
    assert_eq!(
        parse_strs(&["--extend", "foo", "--init"]).unwrap(),
        Parsed::Init {
            extend: Some("foo".into())
        }
    );
}

#[test]
fn extend_without_init_is_error() {
    assert_eq!(
        parse_strs(&["--extend", "foo"]).unwrap_err(),
        ParseError::ExtendWithoutInit
    );
}

#[test]
fn extend_missing_value_is_error() {
    assert_eq!(
        parse_strs(&["--init", "--extend"]).unwrap_err(),
        ParseError::ExtendMissingValue
    );
}

#[test]
fn extend_empty_value_via_equals_is_error() {
    assert_eq!(
        parse_strs(&["--init", "--extend="]).unwrap_err(),
        ParseError::ExtendEmptyValue
    );
}

#[test]
fn extend_empty_value_via_spaced_is_error() {
    assert_eq!(
        parse_strs(&["--init", "--extend", ""]).unwrap_err(),
        ParseError::ExtendEmptyValue
    );
}

#[test]
fn extend_more_than_once_is_error() {
    assert_eq!(
        parse_strs(&["--init", "--extend", "a", "--extend", "b"]).unwrap_err(),
        ParseError::ExtendDuplicated
    );
}

#[test]
fn init_with_passthrough_args_is_error() {
    assert_eq!(
        parse_strs(&["--init", "task1", "task2"]).unwrap_err(),
        ParseError::InitWithExtraArgs(vec!["task1".into(), "task2".into()])
    );
}

#[test]
fn passthrough_unknown_flags_and_positionals() {
    assert_eq!(
        parse_strs(&["task1", "--cwd", "somewhere", "--verbose"]).unwrap(),
        Parsed::Passthrough {
            args: vec![
                "task1".into(),
                "--cwd".into(),
                "somewhere".into(),
                "--verbose".into()
            ]
        }
    );
}

#[test]
fn duplicate_init_is_allowed() {
    assert_eq!(
        parse_strs(&["--init", "--init"]).unwrap(),
        Parsed::Init { extend: None }
    );
}

#[test]
fn help_intercepts_even_with_other_args() {
    assert_eq!(parse_strs(&["task", "--help"]).unwrap(), Parsed::Help);
    assert_eq!(parse_strs(&["--init", "--help"]).unwrap(), Parsed::Help);
}

#[test]
fn version_text_identifies_makerz_and_points_to_makers() {
    let v = version_text();
    assert!(v.contains("makerz"), "got: {v}");
    assert!(v.contains(env!("CARGO_PKG_VERSION")), "got: {v}");
    assert!(v.contains("makers --version"), "got: {v}");
}

#[test]
fn help_text_lists_all_makerz_flags_and_points_to_makers() {
    let h = help_text();
    for needle in ["--init", "--extend", "--version", "--help", "makers --help"] {
        assert!(h.contains(needle), "help missing {needle:?}: {h}");
    }
}
