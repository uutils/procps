// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uutests::new_ucmd;
use uutests::util::TestScenario;
use uutests::util_name;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_non_matching_pattern() {
    new_ucmd!()
        .arg("THIS_PATTERN_DOES_NOT_MATCH")
        .fails()
        .code_is(1)
        .stderr_contains("pidwait: pattern that searches for process name longer than 15 characters will result in zero matches");

    new_ucmd!()
        .arg("DOES_NOT_MATCH")
        .fails()
        .code_is(1)
        .no_output();
}

#[test]
fn test_no_args() {
    new_ucmd!()
        .fails()
        .code_is(2)
        .no_stdout()
        .stderr_contains("no matching criteria specified");
}

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
