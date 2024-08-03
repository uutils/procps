// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore (words) symdir somefakedir

use crate::common::util::TestScenario;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
#[cfg(target_os = "linux")]
fn test_format() {
    new_ucmd!()
        .args(&["-o", "cmd=CCMD"])
        .succeeds()
        .stdout_contains("CCMD");

    new_ucmd!().args(&["-o", "cmd= "]).succeeds();

    new_ucmd!().args(&["-o", "ccmd=CCMD"]).fails().code_is(1);
}
