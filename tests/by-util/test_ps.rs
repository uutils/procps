// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use regex::Regex;
use uutests::new_ucmd;

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
fn test_long_format() {
    check_header(
        "-l",
        &[
            "F", "S", "UID", "PID", "PPID", "C", "PRI", "NI", "ADDR", "SZ", "WCHAN", "TTY", "TIME",
            "CMD",
        ],
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_long_format_with_y() {
    check_header(
        "-ly",
        &[
            "S", "UID", "PID", "PPID", "C", "PRI", "NI", "RSS", "SZ", "WCHAN", "TTY", "TIME", "CMD",
        ],
    );
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
fn test_register_format() {
    check_header(
        "-X",
        &[
            "PID", "STACKP", "ESP", "EIP", "TMOUT", "ALARM", "STAT", "TTY", "TIME", "COMMAND",
        ],
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_x_format() {
    check_header("-x", &["PID", "TTY", "STAT", "TIME", "COMMAND"]);
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

#[test]
#[cfg(target_os = "linux")]
fn test_no_headers_flags() {
    let regex = Regex::new("^ *PID +").unwrap();
    for flag in &["--no-headers", "--no-heading"] {
        new_ucmd!()
            .arg(flag)
            .succeeds()
            .stdout_does_not_match(&regex);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_deselect() {
    // Inverse of all processes should be empty
    new_ucmd!()
        .args(&["--deselect", "-A", "--no-headers"])
        .fails()
        .code_is(1)
        .stdout_is("");

    // PID 1 should be present in inverse of default filter criteria
    new_ucmd!()
        .args(&["--deselect"])
        .succeeds()
        .stdout_matches(&Regex::new("\n *1 ").unwrap());
}

#[test]
#[cfg(target_os = "linux")]
fn test_pid_selection() {
    let our_pid = std::process::id();
    // Test that only pid 1 and pid of the test runner is present
    let test = |pid_args: &[&str]| {
        let match_regex = Regex::new(&format!("^ *1 *\n *{our_pid} *\n$")).unwrap();
        let mut args = vec!["--no-headers", "-o", "pid"];
        args.extend_from_slice(pid_args);
        new_ucmd!()
            .args(&args)
            .succeeds()
            .stdout_matches(&match_regex);
    };

    for flag in ["-p", "--pid"] {
        test(&[flag, &format!("1 {our_pid}")]);
        test(&[flag, &format!("1,{our_pid}")]);
        test(&[flag, "1", flag, &our_pid.to_string()]);
    }

    // Test nonexistent PID (should show no output)
    new_ucmd!()
        .args(&["-p", "0", "--no-headers"])
        .fails()
        .code_is(1)
        .stdout_is("");

    // Test invalid PID
    new_ucmd!()
        .args(&["-p", "invalid"])
        .fails()
        .stderr_contains("invalid number");
}
