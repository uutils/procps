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
#[cfg(target_os = "linux")]
fn test_find_init() {
    new_ucmd!().arg("init").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn test_find_kthreadd() {
    new_ucmd!().arg("kthreadd").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn test_no_program() {
    new_ucmd!().fails().code_is(1);
}

#[test]
#[cfg(target_os = "linux")]
fn test_no_pid_found() {
    new_ucmd!().arg("NO_THIS_PROGRAM").fails().code_is(1);
}
