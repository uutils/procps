// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;

#[test]
fn test_invalid_pid() {
    for invalid_pid in ["0", "invalid"] {
        new_ucmd!()
            .arg(invalid_pid)
            .fails()
            .code_is(1)
            .no_stdout()
            .stderr_contains(format!("invalid process id: {invalid_pid}"));
    }
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}
