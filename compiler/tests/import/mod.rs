use crate::{assert_satisfied, parse_program};

use std::env::{current_dir, set_current_dir};

static TEST_SOURCE_DIRECTORY: &str = "tests/import";

// Import tests rely on knowledge of local directories. They should be run locally only.

pub fn set_local_dir() {
    let mut local = current_dir().unwrap();
    local.push(TEST_SOURCE_DIRECTORY);

    set_current_dir(local).unwrap();
}

#[test]
#[ignore]
fn test_basic() {
    set_local_dir();

    let bytes = include_bytes!("basic.leo");
    let program = parse_program(bytes).unwrap();

    assert_satisfied(program);
}

#[test]
#[ignore]
fn test_multiple() {
    set_local_dir();

    let bytes = include_bytes!("multiple.leo");
    let program = parse_program(bytes).unwrap();

    assert_satisfied(program);
}

#[test]
#[ignore]
fn test_star() {
    set_local_dir();

    let bytes = include_bytes!("star.leo");
    let program = parse_program(bytes).unwrap();

    assert_satisfied(program);
}

#[test]
#[ignore]
fn test_star_fail() {
    set_local_dir();

    let bytes = include_bytes!("star_fail.leo");
    assert!(parse_program(bytes).is_err());
}

#[test]
#[ignore]
fn test_alias() {
    set_local_dir();

    let bytes = include_bytes!("alias.leo");
    let program = parse_program(bytes).unwrap();

    assert_satisfied(program);
}

// naming tests
#[test]
#[ignore]
fn test_names_pass() {
    set_local_dir();

    let bytes = include_bytes!("names.leo");
    let program = parse_program(bytes).unwrap();

    assert_satisfied(program);
}

#[test]
#[ignore]
fn test_names_fail_1() {
    set_local_dir();

    let bytes = include_bytes!("names_dash_a.leo");
    assert!(parse_program(bytes).is_err());
}

#[test]
#[ignore]
fn test_names_fail_2() {
    set_local_dir();

    let bytes = include_bytes!("names_a_dash.leo");
    assert!(parse_program(bytes).is_err());
}

#[test]
#[ignore]
fn test_names_fail_3() {
    set_local_dir();

    let bytes = include_bytes!("names_underscore.leo");
    assert!(parse_program(bytes).is_err());
}

#[test]
#[ignore]
fn test_names_fail_4() {
    set_local_dir();

    let bytes = include_bytes!("names_dollar.leo");
    assert!(parse_program(bytes).is_err());
}

// more complex tests
#[test]
#[ignore]
fn test_many_import() {
    set_local_dir();

    let bytes = include_bytes!("many_import.leo");
    let program = parse_program(bytes).unwrap();

    assert_satisfied(program);
}

#[test]
#[ignore]
fn test_many_import_star() {
    set_local_dir();

    let bytes = include_bytes!("many_import_star.leo");
    let program = parse_program(bytes).unwrap();

    assert_satisfied(program);
}
