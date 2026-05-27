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
