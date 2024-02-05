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
