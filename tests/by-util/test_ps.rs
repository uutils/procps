// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uutests::new_ucmd;
use uutests::util::TestScenario;
use uutests::util_name;

#[test]
#[cfg(target_os = "linux")]
fn test_select_all_processes() {
    for arg in ["-A", "-e"] {
        // TODO ensure the output format is correct
        new_ucmd!().arg(arg).succeeds();
    }
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

/// Helper function to check that ps output has the correct headers in the correct order
#[cfg(target_os = "linux")]
fn check_header(flag: &str, expected_headers: &[&str]) {
    let result = new_ucmd!().arg(flag).succeeds();
    let lines: Vec<&str> = result.stdout_str().lines().collect();
    let headers: Vec<&str> = lines[0].split_whitespace().collect();

    assert_eq!(headers, expected_headers);
}

#[test]
#[cfg(target_os = "linux")]
fn test_full_format_listing() {
    check_header(
        "-f",
        &["UID", "PID", "PPID", "C", "STIME", "TTY", "TIME", "CMD"],
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_extra_full_format() {
    check_header(
        "-F",
        &[
            "UID", "PID", "PPID", "C", "SZ", "RSS", "PSR", "STIME", "TTY", "TIME", "CMD",
        ],
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_job_format() {
    check_header("-j", &["PID", "PGID", "SID", "TTY", "TIME", "CMD"]);
}

#[test]
#[cfg(target_os = "linux")]
fn test_psr_format() {
    check_header("-P", &["PID", "PSR", "TTY", "TIME", "CMD"]);
}

#[test]
#[cfg(target_os = "linux")]
fn test_signal_format() {
    check_header(
        "-s",
        &[
            "UID", "PID", "PENDING", "BLOCKED", "IGNORED", "CAUGHT", "STAT", "TTY", "TIME",
            "COMMAND",
        ],
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_user_format() {
    check_header(
        "-u",
        &[
            "USER", "PID", "%CPU", "%MEM", "VSZ", "RSS", "TTY", "STAT", "START", "TIME", "COMMAND",
        ],
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_virtual_memory_format() {
    check_header(
        "-v",
        &[
            "PID", "TTY", "STAT", "TIME", "MAJFL", "TRS", "DRS", "RSS", "%MEM", "COMMAND",
        ],
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_code_mapping() {
    new_ucmd!()
        .args(&["-o", "cmd=CCMD"])
        .succeeds()
        .stdout_contains("CCMD");

    new_ucmd!().args(&["-o", "cmd= "]).succeeds();

    new_ucmd!().args(&["-o", "ccmd=CCMD"]).fails().code_is(1);

    new_ucmd!()
        .args(&["-o", "cmd=CMD1", "-o", "cmd=CMD2"])
        .succeeds()
        .stdout_contains("CMD1")
        .stdout_contains("CMD2");

    new_ucmd!()
        .args(&["-o", "cmd=CMD1,cmd=CMD2"])
        .succeeds()
        .stdout_contains("CMD1")
        .stdout_contains("CMD2");

    new_ucmd!()
        .args(&["-o", "ucmd=CMD1", "-o", "ucmd=CMD2"])
        .succeeds()
        .stdout_contains("CMD1")
        .stdout_contains("CMD2");

    new_ucmd!()
        .args(&["-o", "ucmd=CMD1,ucmd=CMD2"])
        .succeeds()
        .stdout_contains("CMD1")
        .stdout_contains("CMD2");
}
