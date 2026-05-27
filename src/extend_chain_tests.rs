use std::fs;
use std::path::Path;

use tempfile::tempdir;

use super::*;
use crate::location::MakefileLocation;

fn write_makefile(dir: &Path, content: &str) -> MakefileLocation {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("Makefile.toml"), content).unwrap();
    MakefileLocation::new(dir.to_path_buf())
}

#[test]
fn single_makefile_no_extend_returns_self() {
    let tmp = tempdir().unwrap();
    let loc = write_makefile(tmp.path(), "[env]\nFOO = \"x\"\n");
    let chain = build_chain(loc).unwrap();
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].env().plain_keys(), &["FOO"]);
}

#[test]
fn two_level_chain_orders_ancestor_then_leaf() {
    let tmp = tempdir().unwrap();
    let parent = tmp.path().join("parent");
    let child = tmp.path().join("child");
    write_makefile(&parent, "[env]\nP = \"p\"\n");
    let child_loc = write_makefile(
        &child,
        "extend = \"../parent/Makefile.toml\"\n[env]\nC = \"c\"\n",
    );

    let chain = build_chain(child_loc).unwrap();

    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0].env().plain_keys(), &["P"]);
    assert_eq!(chain[1].env().plain_keys(), &["C"]);
}

#[test]
fn three_level_chain_orders_root_first() {
    let tmp = tempdir().unwrap();
    let gp = tmp.path().join("gp");
    let p = tmp.path().join("p");
    let c = tmp.path().join("c");
    write_makefile(&gp, "[env]\nGP = \"gp\"\n");
    write_makefile(
        &p,
        "extend = \"../gp/Makefile.toml\"\n[env]\nP = \"p\"\n",
    );
    let c_loc = write_makefile(
        &c,
        "extend = \"../p/Makefile.toml\"\n[env]\nC = \"c\"\n",
    );

    let chain = build_chain(c_loc).unwrap();

    assert_eq!(chain.len(), 3);
    assert_eq!(chain[0].env().plain_keys(), &["GP"]);
    assert_eq!(chain[1].env().plain_keys(), &["P"]);
    assert_eq!(chain[2].env().plain_keys(), &["C"]);
}

#[test]
fn extend_to_missing_file_is_not_found() {
    let tmp = tempdir().unwrap();
    let child = tmp.path().join("child");
    let c_loc = write_makefile(&child, "extend = \"../nope/Makefile.toml\"\n");

    let err = build_chain(c_loc).unwrap_err();

    let ChainError::ExtendNotFound { target, from, .. } = err else {
        panic!("expected ExtendNotFound, got {err:?}");
    };
    assert!(target.ends_with("nope/Makefile.toml"), "target = {target:?}");
    assert!(from.ends_with("child/Makefile.toml"), "from = {from:?}");
}

#[test]
fn cycle_between_two_makefiles_is_detected() {
    let tmp = tempdir().unwrap();
    let a = tmp.path().join("a");
    let b = tmp.path().join("b");
    write_makefile(&a, "extend = \"../b/Makefile.toml\"\n");
    write_makefile(&b, "extend = \"../a/Makefile.toml\"\n");

    let err = build_chain(MakefileLocation::new(a.clone())).unwrap_err();

    let ChainError::Cycle { cycle_at } = err else {
        panic!("expected Cycle, got {err:?}");
    };
    assert!(cycle_at.ends_with("a/Makefile.toml"), "cycle_at = {cycle_at:?}");
}

#[test]
fn self_extend_is_cycle() {
    let tmp = tempdir().unwrap();
    let a = tmp.path().join("a");
    let a_loc = write_makefile(&a, "extend = \"./Makefile.toml\"\n");

    let err = build_chain(a_loc).unwrap_err();

    assert!(matches!(err, ChainError::Cycle { .. }), "got {err:?}");
}

#[test]
fn multi_parent_in_chain_propagates_as_parse_error() {
    let tmp = tempdir().unwrap();
    let c = tmp.path().join("c");
    let c_loc = write_makefile(&c, "extend = [\"../p/Makefile.toml\"]\n");

    let err = build_chain(c_loc).unwrap_err();

    assert!(
        matches!(
            err,
            ChainError::Parse {
                source: ParseMakefileError::ExtendMultiParent,
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn relative_attribute_in_chain_propagates() {
    let tmp = tempdir().unwrap();
    let c = tmp.path().join("c");
    let c_loc = write_makefile(
        &c,
        "extend = { path = \"../p/Makefile.toml\", relative = \"git\" }\n",
    );

    let err = build_chain(c_loc).unwrap_err();

    assert!(
        matches!(
            err,
            ChainError::Parse {
                source: ParseMakefileError::ExtendRelative { .. },
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn optional_attribute_in_chain_propagates() {
    let tmp = tempdir().unwrap();
    let c = tmp.path().join("c");
    let c_loc = write_makefile(
        &c,
        "extend = { path = \"../p/Makefile.toml\", optional = true }\n",
    );

    let err = build_chain(c_loc).unwrap_err();

    assert!(
        matches!(
            err,
            ChainError::Parse {
                source: ParseMakefileError::ExtendOptional,
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn ancestor_parse_error_propagates() {
    let tmp = tempdir().unwrap();
    let p = tmp.path().join("p");
    let c = tmp.path().join("c");
    write_makefile(&p, "this is = = not toml\n");
    let c_loc = write_makefile(&c, "extend = \"../p/Makefile.toml\"\n");

    let err = build_chain(c_loc).unwrap_err();

    assert!(
        matches!(
            err,
            ChainError::Parse {
                source: ParseMakefileError::TomlParse(_),
                ..
            }
        ),
        "got {err:?}"
    );
}
