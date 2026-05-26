use std::fs;

use tempfile::tempdir;

use super::*;

#[test]
fn finds_makefile_in_start_dir() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("Makefile.toml"), "").unwrap();

    let found = find_makefile(tmp.path()).unwrap();
    assert_eq!(found.dir(), tmp.path());
}

#[test]
fn finds_makefile_in_parent_dir() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("Makefile.toml"), "").unwrap();
    let sub = tmp.path().join("a").join("b");
    fs::create_dir_all(&sub).unwrap();

    let found = find_makefile(&sub).unwrap();
    assert_eq!(found.dir(), tmp.path());
}

#[test]
fn returns_not_found_when_no_makefile_in_ancestors() {
    // Relies on the test machine not having a Makefile.toml at any ancestor
    // of the OS temp dir (true on CI and typical dev machines).
    let tmp = tempdir().unwrap();
    let err = find_makefile(tmp.path()).unwrap_err();
    assert!(matches!(err, DiscoveryError::NotFound { ref start } if start == tmp.path()));
}

#[test]
fn picks_nearest_ancestor_when_multiple_exist() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("Makefile.toml"), "").unwrap();

    let inner = tmp.path().join("inner");
    fs::create_dir(&inner).unwrap();
    fs::write(inner.join("Makefile.toml"), "").unwrap();

    let leaf = inner.join("leaf");
    fs::create_dir(&leaf).unwrap();

    let found = find_makefile(&leaf).unwrap();
    assert_eq!(found.dir(), inner);
}

#[test]
fn ignores_directory_named_makefile_toml() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("Makefile.toml"), "").unwrap();

    let sub = tmp.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::create_dir(sub.join("Makefile.toml")).unwrap();

    let found = find_makefile(&sub).unwrap();
    assert_eq!(found.dir(), tmp.path());
}
