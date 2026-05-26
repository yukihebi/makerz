use std::fs;
use std::path::Path;

use tempfile::tempdir;

use super::*;
use crate::location::MakefileLocation;

// ---- parse_content tests (no file I/O) ----

#[test]
fn invalid_toml_returns_toml_parse_error() {
    let err = parse_content("this is = = not toml").unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::TomlParse(_)),
        "got {err:?}"
    );
}

#[test]
fn no_extend_returns_none() {
    let (_, extend) = parse_content("[env]\n").unwrap();
    assert_eq!(extend, None);
}

#[test]
fn string_extend_returns_some() {
    let (_, extend) = parse_content("extend = \"../parent/Makefile.toml\"\n").unwrap();
    assert_eq!(extend.as_deref(), Some("../parent/Makefile.toml"));
}

#[test]
fn array_extend_is_error() {
    let err = parse_content("extend = [\"a\", \"b\"]\n").unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::ExtendNotString { .. }),
        "got {err:?}"
    );
}

#[test]
fn table_extend_is_error() {
    let err = parse_content("extend = { path = \"../p/Makefile.toml\" }\n").unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::ExtendNotString { .. }),
        "got {err:?}"
    );
}

#[test]
fn empty_file_has_empty_env() {
    let (env, extend) = parse_content("").unwrap();
    assert_eq!(extend, None);
    assert!(env.plain_keys().is_empty());
    assert!(env.file().is_none());
    assert!(env.caller().is_none());
    assert!(env.inherit().is_empty());
}

#[test]
fn env_keys_collected_in_order_as_plain() {
    let (env, _) = parse_content("[env]\nFOO = \"a\"\nBAR = \"b\"\nBAZ = \"c\"\n").unwrap();
    assert_eq!(env.plain_keys(), &["FOO", "BAR", "BAZ"]);
}

#[test]
fn no_env_section_means_empty_env() {
    let (env, _) = parse_content("[tasks.default]\nscript = \"echo hi\"\n").unwrap();
    assert!(env.plain_keys().is_empty());
}

#[test]
fn file_directive_binds_to_next_env_key() {
    let (env, _) = parse_content("[env]\n# @makerz = \"file\"\nFOO_DIR = \".\"\n").unwrap();
    let file = env.file().unwrap();
    assert_eq!(file.name(), "FOO_DIR");
    assert_eq!(file.fallback(), ".");
    assert!(env.plain_keys().is_empty());
}

#[test]
fn caller_directive_binds() {
    let (env, _) = parse_content("[env]\n# @makerz = \"caller\"\nCALLER_DIR = \".\"\n").unwrap();
    let caller = env.caller().unwrap();
    assert_eq!(caller.name(), "CALLER_DIR");
    assert_eq!(caller.fallback(), ".");
}

#[test]
fn inherit_directive_binds() {
    let (env, _) = parse_content("[env]\n# @makerz = \"inherit\"\nPARENT_DIR = \"..\"\n").unwrap();
    assert_eq!(env.inherit().len(), 1);
    assert_eq!(env.inherit()[0].name(), "PARENT_DIR");
    assert_eq!(env.inherit()[0].fallback(), "..");
}

#[test]
fn multiple_inherits_with_distinct_names() {
    let (env, _) = parse_content(
        "[env]\n\
         # @makerz = \"inherit\"\n\
         A_DIR = \"../a\"\n\
         # @makerz = \"inherit\"\n\
         B_DIR = \"../b\"\n",
    )
    .unwrap();
    let inh = env.inherit();
    assert_eq!(inh.len(), 2);
    assert_eq!(inh[0].name(), "A_DIR");
    assert_eq!(inh[0].fallback(), "../a");
    assert_eq!(inh[1].name(), "B_DIR");
    assert_eq!(inh[1].fallback(), "../b");
}

#[test]
fn blank_line_between_directive_and_key_still_binds() {
    let (env, _) =
        parse_content("[env]\n# @makerz = \"file\"\n\n# unrelated comment\n\nFOO_DIR = \".\"\n")
            .unwrap();
    assert_eq!(env.file().unwrap().name(), "FOO_DIR");
}

#[test]
fn directive_combined_with_plain_keys() {
    let (env, _) = parse_content(
        "[env]\n\
         PLAIN_A = \"x\"\n\
         # @makerz = \"file\"\n\
         FOO_DIR = \".\"\n\
         PLAIN_B = \"y\"\n",
    )
    .unwrap();
    assert_eq!(env.file().unwrap().name(), "FOO_DIR");
    assert_eq!(env.plain_keys(), &["PLAIN_A", "PLAIN_B"]);
}

#[test]
fn unknown_directive_value_is_error() {
    let err = parse_content("[env]\n# @makerz = \"weird\"\nFOO = \".\"\n").unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveUnknownValue { value } if value == "weird"));
}

#[test]
fn empty_directive_value_is_error() {
    let err = parse_content("[env]\n# @makerz = \"\"\nFOO = \".\"\n").unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveUnknownValue { value } if value.is_empty()));
}

#[test]
fn directive_before_env_section_is_error() {
    let err = parse_content("# @makerz = \"file\"\nFOO = \".\"\n[env]\nBAR = \"x\"\n").unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveOutsideEnv { value } if value == "file"));
}

#[test]
fn directive_in_tasks_section_is_error() {
    let err =
        parse_content("[env]\nFOO = \"x\"\n[tasks.default]\n# @makerz = \"file\"\ncwd = \".\"\n")
            .unwrap_err();
    assert!(matches!(
        err,
        ParseMakefileError::DirectiveOutsideEnv { .. }
    ));
}

#[test]
fn directive_at_eof_without_key_is_unbound() {
    let err = parse_content("[env]\nFOO = \"x\"\n# @makerz = \"file\"\n").unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveUnbound { value } if value == "file"));
}

#[test]
fn directive_before_next_section_is_unbound() {
    let err = parse_content(
        "[env]\nFOO = \"x\"\n# @makerz = \"file\"\n[tasks.default]\nscript = \"x\"\n",
    )
    .unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveUnbound { value } if value == "file"));
}

#[test]
fn two_directives_on_same_key_is_error() {
    let err = parse_content("[env]\n# @makerz = \"file\"\n# @makerz = \"caller\"\nFOO = \".\"\n")
        .unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveOnSameVar { name } if name == "FOO"));
}

#[test]
fn two_file_directives_is_error() {
    let err = parse_content(
        "[env]\n\
         # @makerz = \"file\"\n\
         A_DIR = \".\"\n\
         # @makerz = \"file\"\n\
         B_DIR = \".\"\n",
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ParseMakefileError::DuplicateFile { previous, new } if previous == "A_DIR" && new == "B_DIR"
    ));
}

#[test]
fn two_caller_directives_is_error() {
    let err = parse_content(
        "[env]\n\
         # @makerz = \"caller\"\n\
         A = \".\"\n\
         # @makerz = \"caller\"\n\
         B = \".\"\n",
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ParseMakefileError::DuplicateCaller { previous, new } if previous == "A" && new == "B"
    ));
}

#[test]
fn fallback_not_string_is_error() {
    let err = parse_content("[env]\n# @makerz = \"file\"\nFOO = 42\n").unwrap_err();
    assert!(matches!(err, ParseMakefileError::FallbackNotString { name } if name == "FOO"));
}

#[test]
fn fallback_array_value_for_directive_is_error() {
    let err = parse_content("[env]\n# @makerz = \"file\"\nFOO = [1, 2]\n").unwrap_err();
    assert!(matches!(err, ParseMakefileError::FallbackNotString { name } if name == "FOO"));
}

#[test]
fn directive_intent_without_equals_is_malformed() {
    let err = parse_content("[env]\n# @makerz\nFOO = \".\"\n").unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::DirectiveMalformed { .. }),
        "got {err:?}"
    );
}

#[test]
fn directive_intent_missing_quotes_is_malformed() {
    let err = parse_content("[env]\n# @makerz = file\nFOO = \".\"\n").unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::DirectiveMalformed { .. }),
        "got {err:?}"
    );
}

#[test]
fn directive_intent_unterminated_quote_is_malformed() {
    let err = parse_content("[env]\n# @makerz = \"file\nFOO = \".\"\n").unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::DirectiveMalformed { .. }),
        "got {err:?}"
    );
}

#[test]
fn directive_intent_with_trailing_garbage_is_malformed() {
    let err = parse_content("[env]\n# @makerz = \"file\" garbage\nFOO_DIR = \".\"\n").unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::DirectiveMalformed { .. }),
        "got {err:?}"
    );
}

#[test]
fn at_makerz_substring_in_non_directive_comment_is_silent() {
    let (env, _) = parse_content(
        "[env]\n# this comment mentions @makerz directives in passing\nFOO = \".\"\n",
    )
    .unwrap();
    assert_eq!(env.plain_keys(), &["FOO"]);
    assert!(env.file().is_none());
}

#[test]
fn makerz_extended_identifier_is_silent() {
    let (env, _) = parse_content("[env]\n# @makerzx = \"file\"\nFOO = \".\"\n").unwrap();
    assert_eq!(env.plain_keys(), &["FOO"]);
    assert!(env.file().is_none());
}

#[test]
fn directive_trailing_comment_is_allowed() {
    let (env, _) =
        parse_content("[env]\n# @makerz = \"file\"  # bind file to FOO_DIR\nFOO_DIR = \".\"\n")
            .unwrap();
    assert_eq!(env.file().unwrap().name(), "FOO_DIR");
}

// ---- parse(location) file-I/O tests ----

fn write_to(dir: &Path, content: &str) -> MakefileLocation {
    fs::write(dir.join("Makefile.toml"), content).unwrap();
    MakefileLocation::new(dir.to_path_buf())
}

#[test]
fn parse_reads_file_and_attaches_location() {
    let tmp = tempdir().unwrap();
    let loc = write_to(tmp.path(), "[env]\nFOO = \"x\"\n");
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.location().dir(), tmp.path());
    assert_eq!(parsed.env().plain_keys(), &["FOO"]);
}

#[test]
fn parse_reports_read_error_when_makefile_missing() {
    let tmp = tempdir().unwrap();
    let loc = MakefileLocation::new(tmp.path().to_path_buf());
    let err = parse(loc).unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::Read { .. }),
        "got {err:?}"
    );
}
