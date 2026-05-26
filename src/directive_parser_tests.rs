use std::fs;

use tempfile::tempdir;

use super::*;
use crate::discovery::MakefileLocation;

fn write_makefile(dir: &std::path::Path, content: &str) -> MakefileLocation {
    fs::write(dir.join("Makefile.toml"), content).unwrap();
    MakefileLocation::new(dir.to_path_buf())
}

#[test]
fn invalid_toml_returns_toml_parse_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "this is = = not toml");

    let err = parse(loc).unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::TomlParse { .. }),
        "got {err:?}"
    );
}

#[test]
fn no_extend_returns_none() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\n");
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.extend(), None);
}

#[test]
fn string_extend_returns_some() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "extend = \"../parent/Makefile.toml\"\n");
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.extend(), Some("../parent/Makefile.toml"));
}

#[test]
fn array_extend_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "extend = [\"a\", \"b\"]\n");
    let err = parse(loc).unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::ExtendNotString { .. }),
        "got {err:?}"
    );
}

#[test]
fn table_extend_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "extend = { path = \"../p/Makefile.toml\" }\n");
    let err = parse(loc).unwrap_err();
    assert!(
        matches!(err, ParseMakefileError::ExtendNotString { .. }),
        "got {err:?}"
    );
}

#[test]
fn empty_file_has_empty_env() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "");
    let parsed = parse(loc).unwrap();
    assert!(parsed.env().plain_keys().is_empty());
    assert!(parsed.env().file().is_none());
    assert!(parsed.env().caller().is_none());
    assert!(parsed.env().inherit().is_empty());
}

#[test]
fn env_keys_collected_in_order_as_plain() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\nFOO = \"a\"\nBAR = \"b\"\nBAZ = \"c\"\n");
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.env().plain_keys(), &["FOO", "BAR", "BAZ"]);
}

#[test]
fn no_env_section_means_empty_env() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[tasks.default]\nscript = \"echo hi\"\n");
    let parsed = parse(loc).unwrap();
    assert!(parsed.env().plain_keys().is_empty());
}

#[test]
fn file_directive_binds_to_next_env_key() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\n# @makerz = \"file\"\nFOO_DIR = \".\"\n");
    let parsed = parse(loc).unwrap();
    let file = parsed.env().file().unwrap();
    assert_eq!(file.name(), "FOO_DIR");
    assert_eq!(file.fallback(), ".");
    assert!(parsed.env().plain_keys().is_empty());
}

#[test]
fn caller_directive_binds() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\n# @makerz = \"caller\"\nCALLER_DIR = \".\"\n",
    );
    let parsed = parse(loc).unwrap();
    let caller = parsed.env().caller().unwrap();
    assert_eq!(caller.name(), "CALLER_DIR");
    assert_eq!(caller.fallback(), ".");
}

#[test]
fn inherit_directive_binds() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\n# @makerz = \"inherit\"\nPARENT_DIR = \"..\"\n",
    );
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.env().inherit().len(), 1);
    assert_eq!(parsed.env().inherit()[0].name(), "PARENT_DIR");
    assert_eq!(parsed.env().inherit()[0].fallback(), "..");
}

#[test]
fn multiple_inherits_with_distinct_names() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\n\
         # @makerz = \"inherit\"\n\
         A_DIR = \"../a\"\n\
         # @makerz = \"inherit\"\n\
         B_DIR = \"../b\"\n",
    );
    let parsed = parse(loc).unwrap();
    let inh = parsed.env().inherit();
    assert_eq!(inh.len(), 2);
    assert_eq!(inh[0].name(), "A_DIR");
    assert_eq!(inh[0].fallback(), "../a");
    assert_eq!(inh[1].name(), "B_DIR");
    assert_eq!(inh[1].fallback(), "../b");
}

#[test]
fn blank_line_between_directive_and_key_still_binds() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\n# @makerz = \"file\"\n\n# unrelated comment\n\nFOO_DIR = \".\"\n",
    );
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.env().file().unwrap().name(), "FOO_DIR");
}

#[test]
fn directive_combined_with_plain_keys() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\n\
         PLAIN_A = \"x\"\n\
         # @makerz = \"file\"\n\
         FOO_DIR = \".\"\n\
         PLAIN_B = \"y\"\n",
    );
    let parsed = parse(loc).unwrap();
    assert_eq!(parsed.env().file().unwrap().name(), "FOO_DIR");
    assert_eq!(parsed.env().plain_keys(), &["PLAIN_A", "PLAIN_B"]);
}

#[test]
fn unknown_directive_value_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\n# @makerz = \"weird\"\nFOO = \".\"\n");
    let err = parse(loc).unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveUnknownValue { value } if value == "weird"));
}

#[test]
fn empty_directive_value_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\n# @makerz = \"\"\nFOO = \".\"\n");
    let err = parse(loc).unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveUnknownValue { value } if value.is_empty()));
}

#[test]
fn directive_before_env_section_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "# @makerz = \"file\"\nFOO = \".\"\n[env]\nBAR = \"x\"\n",
    );
    let err = parse(loc).unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveOutsideEnv { value } if value == "file"));
}

#[test]
fn directive_in_tasks_section_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\nFOO = \"x\"\n[tasks.default]\n# @makerz = \"file\"\ncwd = \".\"\n",
    );
    let err = parse(loc).unwrap_err();
    assert!(matches!(
        err,
        ParseMakefileError::DirectiveOutsideEnv { .. }
    ));
}

#[test]
fn directive_at_eof_without_key_is_unbound() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\nFOO = \"x\"\n# @makerz = \"file\"\n");
    let err = parse(loc).unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveUnbound { value } if value == "file"));
}

#[test]
fn directive_before_next_section_is_unbound() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\nFOO = \"x\"\n# @makerz = \"file\"\n[tasks.default]\nscript = \"x\"\n",
    );
    let err = parse(loc).unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveUnbound { value } if value == "file"));
}

#[test]
fn two_directives_on_same_key_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\n# @makerz = \"file\"\n# @makerz = \"caller\"\nFOO = \".\"\n",
    );
    let err = parse(loc).unwrap_err();
    assert!(matches!(err, ParseMakefileError::DirectiveOnSameVar { name } if name == "FOO"));
}

#[test]
fn two_file_directives_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\n\
         # @makerz = \"file\"\n\
         A_DIR = \".\"\n\
         # @makerz = \"file\"\n\
         B_DIR = \".\"\n",
    );
    let err = parse(loc).unwrap_err();
    assert!(matches!(
        err,
        ParseMakefileError::DuplicateFile { previous, new } if previous == "A_DIR" && new == "B_DIR"
    ));
}

#[test]
fn two_caller_directives_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(
        tmp.path(),
        "[env]\n\
         # @makerz = \"caller\"\n\
         A = \".\"\n\
         # @makerz = \"caller\"\n\
         B = \".\"\n",
    );
    let err = parse(loc).unwrap_err();
    assert!(matches!(
        err,
        ParseMakefileError::DuplicateCaller { previous, new } if previous == "A" && new == "B"
    ));
}

#[test]
fn fallback_not_string_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\n# @makerz = \"file\"\nFOO = 42\n");
    let err = parse(loc).unwrap_err();
    assert!(matches!(err, ParseMakefileError::FallbackNotString { name } if name == "FOO"));
}

#[test]
fn fallback_array_value_for_directive_is_error() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\n# @makerz = \"file\"\nFOO = [1, 2]\n");
    let err = parse(loc).unwrap_err();
    assert!(matches!(err, ParseMakefileError::FallbackNotString { name } if name == "FOO"));
}
