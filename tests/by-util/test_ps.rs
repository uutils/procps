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
fn test_code_mapping() {
    new_ucmd!()
        .args(&["-o", "cmd=CCMD"])
        .succeeds()
        .stdout_contains("CCMD");

    new_ucmd!().args(&["-o", "cmd= "]).succeeds();

    new_ucmd!().args(&["-o", "ccmd=CCMD"]).fails().code_is(1);

    new_ucmd!()
        .args(&["-o", "cmd=CMD1", "-o", "cmd=CMD2"])
        .succeeds()
        .stdout_contains("CMD1")
        .stdout_contains("CMD2");

    new_ucmd!()
        .args(&["-o", "cmd=CMD1,cmd=CMD2"])
        .succeeds()
        .stdout_contains("CMD1")
        .stdout_contains("CMD2");

    new_ucmd!()
        .args(&["-o", "ucmd=CMD1", "-o", "ucmd=CMD2"])
        .succeeds()
        .stdout_contains("CMD1")
        .stdout_contains("CMD2");

    new_ucmd!()
        .args(&["-o", "ucmd=CMD1,ucmd=CMD2"])
        .succeeds()
        .stdout_contains("CMD1")
        .stdout_contains("CMD2");
}
