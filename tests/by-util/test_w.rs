// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore (words) symdir somefakedir

use crate::common::util::TestScenario;
use std::path::Path;

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
    // There is no guarantee that there will be a cmdline entry, but for testing purposes, we will assume so,
    // since there will be one present in most cases.
    for line in output_lines {
        // We need to get rid of extra 0 terminators on strings, such as the cmdline, so we don't have characters at the end.
        let line_vec: Vec<String> = line
            .split_whitespace()
            .map(|s| String::from(s.trim_end_matches('\0')))
            .collect();
        // Check the time formatting, this should be the third entry in list
        // For now, we are just going to check that that length of time is 5 and it has a colon
        assert!(line_vec[2].contains(":") && line_vec[2].chars().count() == 5);
        // Check to make sure that cmdline is a path that exists.
        // For now, cmdline will be in index 3, until the output is complete.
        assert!(Path::new(line_vec.last().unwrap().as_str()).exists())
    }
}
