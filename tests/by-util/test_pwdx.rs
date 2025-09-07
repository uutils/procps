// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::process;

use regex::Regex;

use uutests::new_ucmd;

#[test]
fn test_no_args() {
    new_ucmd!().fails().code_is(1);
}

#[test]
fn test_valid_pid() {
    let pid = process::id();

    new_ucmd!()
        .arg(pid.to_string())
        .succeeds()
        .stdout_matches(&Regex::new(&format!("^{pid}: .+\n$")).unwrap());
}

#[test]
fn test_multiple_valid_pids() {
    let pid = process::id();

    new_ucmd!()
        .arg(pid.to_string())
        .arg(pid.to_string())
        .succeeds()
        // (?m) enables multi-line mode
        .stdout_matches(&Regex::new(&format!("(?m)^{pid}: .+$")).unwrap());
}

#[test]
fn test_non_existing_pid() {
    let non_existing_pid = "999999";

    new_ucmd!()
        .arg(non_existing_pid)
        .fails()
        .code_is(1)
        .no_stdout()
        .stderr_is(format!("{non_existing_pid}: No such process\n"));
}

#[test]
fn test_non_existing_and_existing_pid() {
    let pid = process::id();
    let non_existing_pid = "999999";

    new_ucmd!()
        .arg(non_existing_pid)
        .arg(pid.to_string())
        .fails()
        .code_is(1)
        .stdout_matches(&Regex::new(&format!("^{pid}: .+\n$")).unwrap())
        .stderr_is(format!("{non_existing_pid}: No such process\n"));

    new_ucmd!()
        .arg(pid.to_string())
        .arg(non_existing_pid)
        .fails()
        .code_is(1)
        .stdout_matches(&Regex::new(&format!("^{pid}: .+\n$")).unwrap())
        .stderr_is(format!("{non_existing_pid}: No such process\n"));
}

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
fn test_invalid_and_valid_pid() {
    let pid = process::id();
    let invalid_pid = "invalid";

    new_ucmd!()
        .arg(invalid_pid)
        .arg(pid.to_string())
        .fails()
        .code_is(1)
        .no_stdout()
        .stderr_contains(format!("invalid process id: {invalid_pid}"));

    new_ucmd!()
        .arg(pid.to_string())
        .arg(invalid_pid)
        .fails()
        .code_is(1)
        .stdout_matches(&Regex::new(&format!("^{pid}: .+\n$")).unwrap())
        .stderr_contains(format!("invalid process id: {invalid_pid}"));
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}
