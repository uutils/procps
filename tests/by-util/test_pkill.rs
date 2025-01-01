// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;

#[cfg(target_os = "linux")]
#[test]
fn test_no_args() {
    new_ucmd!()
        .fails()
        .code_is(2)
        .no_stdout()
        .stderr_contains("no matching criteria specified");
}

#[cfg(target_os = "linux")]
#[test]
fn test_non_matching_pattern() {
    new_ucmd!()
        .arg("THIS_PATTERN_DOES_NOT_MATCH")
        .fails()
        .code_is(1)
        .no_output();
}

#[cfg(target_os = "linux")]
#[test]
fn test_too_many_patterns() {
    new_ucmd!()
        .arg("sh")
        .arg("sh")
        .fails()
        .code_is(2)
        .no_stdout()
        .stderr_contains("only one pattern can be provided");
}

#[cfg(target_os = "linux")]
#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[cfg(target_os = "linux")]
#[test]
fn test_help() {
    new_ucmd!().arg("--help").succeeds();
}