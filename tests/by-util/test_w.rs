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

    assert!(!result.contains("USER\tTTY\tLOGIN@\tIDLE\tJCPU\tPCPU\tWHAT"));
}

#[test]
fn test_output_format() {
    // Use no header to simplify testing
    let cmd = new_ucmd!().arg("--no-header").succeeds();
    let output_lines = cmd.stdout_str().lines();

    for line in output_lines {
        let line_vec: Vec<String> = line.split_whitespace().map(String::from).collect();
        // Check the time formatting, this should be the third entry in list
        // For now, we are just going to check that that length of time is 5 and it has a colon, else
        // it is possible that a time can look like Fri13, so it can start with a letter and end
        // with a number
        assert!(
            (line_vec[2].contains(':') && line_vec[2].chars().count() == 5)
                || (line_vec[2].starts_with(char::is_alphabetic)
                    && line_vec[2].ends_with(char::is_numeric)
                    && line_vec[2].chars().count() == 5)
        );
    }
}
