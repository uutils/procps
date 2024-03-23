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
fn test_no_header() {
    let cmd = new_ucmd!().arg("--no-header").succeeds();

    let result = cmd.stdout_str();

    assert!(!result.contains("USER\tTTY\t\tLOGIN@\t\tIDLE\tJCPU\tPCPU\tWHAT"));
}
