// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_pgrep() {
    new_ucmd!().arg("--help").succeeds().code_is(0);
}

#[test]
#[cfg(target_os = "linux")]
fn test_oldest() {
    new_ucmd!().arg("-o").succeeds().code_is(0);
}

#[test]
#[cfg(target_os = "linux")]
fn test_newest() {
    new_ucmd!().arg("-n").succeeds().code_is(0);
}

#[test]
fn test_not_exist_program() {
    new_ucmd!()
        .arg("THIS_PROGRAM_DOES_NOT_EXIST")
        .fails()
        .code_is(1);
}

#[test]
#[cfg(target_os = "linux")]
fn test_full() {
    new_ucmd!().arg("sh").arg("--full").succeeds().code_is(0);
}

#[test]
#[cfg(target_os = "linux")]
fn test_invalid_regex() {
    new_ucmd!().arg("{(*").arg("--exact").fails().code_is(2);
}

#[test]
#[cfg(target_os = "linux")]
fn test_valid_regex() {
    new_ucmd!()
        .arg("NO_PROGRAM*")
        .arg("--exact")
        .fails()
        .code_is(1);
}

#[cfg(target_os = "linux")]
#[test]
fn test_delimiter() {
    let binding = new_ucmd!().arg("sh").arg("-d |").succeeds();
    let output = binding.code_is(0).stdout_str();

    assert!(output.contains('|'))
}

#[test]
fn test_too_many_patterns() {
    new_ucmd!().arg("sh").arg("sh").fails().code_is(2);
}

#[test]
fn test_too_few_patterns() {
    new_ucmd!().fails().code_is(2);
}

#[test]
#[cfg(target_os = "linux")]
fn test_ignore_case() {
    new_ucmd!().arg("SH").arg("-i").succeeds().code_is(0);
}
