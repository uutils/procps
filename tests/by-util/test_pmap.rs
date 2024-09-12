// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;
#[cfg(target_os = "linux")]
use regex::Regex;
#[cfg(target_os = "linux")]
use std::process;

const NON_EXISTING_PID: &str = "999999";

#[test]
fn test_no_args() {
    new_ucmd!().fails().code_is(1);
}

#[test]
#[cfg(target_os = "linux")]
fn test_existing_pid() {
    let pid = process::id();

    let result = new_ucmd!()
        .arg(pid.to_string())
        .succeeds()
        .stdout_move_str();

    assert_format(pid, &result);
}

#[test]
#[cfg(target_os = "linux")]
fn test_multiple_existing_pids() {
    let pid = process::id();

    let result = new_ucmd!()
        .arg(pid.to_string())
        .arg(pid.to_string())
        .succeeds()
        .stdout_move_str();

    let result: Vec<_> = result.lines().collect();

    let re = Regex::new(r"^[1-9]\d*:").unwrap();
    let pos_second_pid = result.iter().rposition(|line| re.is_match(line)).unwrap();
    let (left, right) = result.split_at(pos_second_pid);

    assert_format(pid, &left.join("\n"));
    assert_format(pid, &right.join("\n"));
}

#[test]
#[cfg(target_os = "linux")]
fn test_non_existing_and_existing_pid() {
    let pid = process::id();

    let result = new_ucmd!()
        .arg(NON_EXISTING_PID)
        .arg(pid.to_string())
        .fails();
    let result = result.code_is(42).no_stderr().stdout_str();

    assert_format(pid, result);
}

#[test]
fn test_non_existing_pid() {
    new_ucmd!()
        .arg(NON_EXISTING_PID)
        .fails()
        .code_is(42)
        .no_output();
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

// Ensure `s` has the following format:
//
// 1234:   /some/path
// 00007ff01c6aa000      8K rw--- ld-linux-x86-64.so.2
// 00007fffa80a6000    132K rw---   [ stack ]
// ffffffffff600000      4K --x--   [ anon ]
// ...
//  total          1040320K
#[cfg(target_os = "linux")]
fn assert_format(pid: u32, s: &str) {
    let (first_line, rest) = s.split_once('\n').unwrap();
    let re = Regex::new(&format!("^{pid}:   .+[^ ]$")).unwrap();
    assert!(re.is_match(first_line));

    let rest = rest.trim_end();
    let (memory_map, last_line) = rest.rsplit_once('\n').unwrap();
    let re = Regex::new("(?m)^[0-9a-f]{16} +[1-9][0-9]*K (-|r)(-|w)(-|x)(-|s)- (  $$[ (anon|stack) $$]|[a-zA-Z0-9._-]+)$").unwrap();
    assert!(re.is_match(memory_map));

    let re = Regex::new("^ total +[1-9][0-9]*K$").unwrap();
    assert!(re.is_match(last_line));
}
