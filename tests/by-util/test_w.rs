// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore (words) symdir somefakedir

use std::{fs, path::Path, process};

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

#[test]
fn test_format_time() {
    let unix_epoc = time::OffsetDateTime::UNIX_EPOCH;
    assert_eq!(w::format_time(unix_epoc).unwrap(), "00:00");
}

#[test]
// Get PID of current process and use that for cmdline testing
fn test_fetch_cmdline() {
    // uucore's utmpx returns an i32, so we cast to that to mimic it.
    let pid = process::id() as i32;
    let path = Path::new("/proc").join(pid.to_string()).join("cmdline");
    assert_eq!(fs::read_to_string(path).unwrap(), w::fetch_cmdline(pid).unwrap())
}
