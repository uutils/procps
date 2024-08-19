// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;

#[test]
fn test_no_args() {
    new_ucmd!().fails().code_is(1);
}

#[test]
#[cfg(target_os = "linux")]
fn test_existing_pid() {
    use std::process;

    use regex::Regex;

    let pid = process::id();
    // TODO ensure that the output format is correct, which is not the case currently
    let result = new_ucmd!()
        .arg(pid.to_string())
        .succeeds()
        .stdout_move_str();

    let (first_line, rest) = result.split_once('\n').unwrap();
    let re = Regex::new(&format!("^{pid}:   .+$")).unwrap();
    assert!(re.is_match(first_line));

    let rest = rest.trim_end();
    let (memory_map, last_line) = rest.rsplit_once('\n').unwrap();
    let re = Regex::new("(?m)^[0-9a-f]{16} ").unwrap();
    assert!(re.is_match(memory_map));
    // TODO ensure that "total" is followed by a total amount
    assert!(last_line.starts_with(" total"));
}

#[test]
fn test_non_existing_pid() {
    new_ucmd!().arg("999999").fails().code_is(42).no_output();
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}
