// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore (words) symdir somefakedir

use crate::common::util::TestScenario;
use pretty_assertions::assert_eq;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_free() {
    let result = new_ucmd!().succeeds();
    assert!(result.stdout_str().contains("Mem:"))
}

#[test]
fn test_free_wide() {
    let result = new_ucmd!().arg("--wide").succeeds();
    assert!(result.stdout_str().contains("Mem:"));
    assert!(!result.stdout_str().contains("buff/cache"));
}

#[test]
fn test_free_column_format() {
    let free_header =
        "               total        used        free      shared  buff/cache   available";
    let free_result = new_ucmd!().succeeds();
    assert_eq!(free_result.stdout_str().len(), 207);
    assert_eq!(
        free_result.stdout_str().split("\n").next().unwrap(),
        free_header
    )
}

#[test]
fn test_free_wide_column_format() {
    let free_header = "               total        used        free      shared     buffers       cache   available";
    let free_result = new_ucmd!().arg("--wide").succeeds();
    assert_eq!(free_result.stdout_str().len(), 231);
    assert_eq!(
        free_result.stdout_str().split("\n").next().unwrap(),
        free_header
    )
}
