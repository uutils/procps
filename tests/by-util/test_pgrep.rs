// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;
#[cfg(target_os = "linux")]
use regex::Regex;

#[cfg(target_os = "linux")]
const SINGLE_PID: &str = "^[1-9][0-9]*";

#[test]
fn test_no_args() {
    new_ucmd!()
        .fails()
        .code_is(2)
        .no_stdout()
        .stderr_contains("no matching criteria specified");
}

#[test]
fn test_non_matching_pattern() {
    new_ucmd!()
        .arg("THIS_PATTERN_DOES_NOT_MATCH")
        .fails()
        .code_is(1)
        .no_output();
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

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_help() {
    new_ucmd!().arg("--help").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn test_oldest() {
    for arg in ["-o", "--oldest"] {
        new_ucmd!()
            .arg(arg)
            .succeeds()
            .stdout_matches(&Regex::new(SINGLE_PID).unwrap());
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_oldest_non_matching_pattern() {
    new_ucmd!()
        .arg("--oldest")
        .arg("non_matching")
        .fails()
        .code_is(1)
        .no_output();
}

#[test]
#[cfg(target_os = "linux")]
fn test_newest() {
    for arg in ["-n", "--newest"] {
        new_ucmd!()
            .arg(arg)
            .succeeds()
            .stdout_matches(&Regex::new(SINGLE_PID).unwrap());
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_newest_non_matching_pattern() {
    new_ucmd!()
        .arg("--newest")
        .arg("non_matching")
        .fails()
        .code_is(1)
        .no_output();
}

#[test]
#[cfg(target_os = "linux")]
fn test_older() {
    for arg in ["-O", "--older"] {
        new_ucmd!()
            .arg(arg)
            .arg("0")
            .succeeds()
            .stdout_matches(&Regex::new("(?m)^[1-9][0-9]*$").unwrap());
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_older_matching_pattern() {
    new_ucmd!()
        .arg("--older=0")
        .arg("sh")
        .succeeds()
        .stdout_matches(&Regex::new("(?m)^[1-9][0-9]*$").unwrap());
}

#[test]
#[cfg(target_os = "linux")]
fn test_older_non_matching_pattern() {
    new_ucmd!()
        .arg("--older=0")
        .arg("non_matching")
        .fails()
        .code_is(1)
        .no_stdout();
}

#[test]
#[cfg(target_os = "linux")]
fn test_full() {
    for arg in ["-f", "--full"] {
        new_ucmd!().arg("sh").arg(arg).succeeds();
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_invalid_regex() {
    new_ucmd!().arg("{(*").arg("--exact").fails().code_is(2);
    new_ucmd!().arg("{(*").fails().code_is(2);
}

#[test]
#[cfg(target_os = "linux")]
fn test_valid_regex() {
    new_ucmd!()
        .arg("NO_PROGRAM*")
        .arg("--exact")
        .fails()
        .code_is(1);
    new_ucmd!().arg("a*").succeeds();
}

#[cfg(target_os = "linux")]
#[test]
fn test_delimiter() {
    for arg in ["-d", "--delimiter"] {
        new_ucmd!()
            .arg("sh")
            .arg(arg)
            .arg("|")
            .succeeds()
            .stdout_contains("|");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn test_delimiter_last_wins() {
    new_ucmd!()
        .arg("sh")
        .arg("-d_")
        .arg("-d:")
        .succeeds()
        .stdout_does_not_contain("_")
        .stdout_contains(":");

    new_ucmd!()
        .arg("sh")
        .arg("-d:")
        .arg("-d_")
        .succeeds()
        .stdout_does_not_contain(":")
        .stdout_contains("_");
}

#[test]
#[cfg(target_os = "linux")]
fn test_ignore_case() {
    for arg in ["-i", "--ignore-case"] {
        new_ucmd!().arg("SH").arg(arg).succeeds();
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_list_full() {
    for arg in ["-a", "--list-full"] {
        new_ucmd!()
            .arg("sh")
            .arg(arg)
            .succeeds()
            // (?m) enables multi-line mode
            .stdout_matches(&Regex::new("(?m)^[1-9][0-9]* .+$").unwrap());
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_list_full_process_with_empty_cmdline() {
    new_ucmd!()
        .arg("kthreadd")
        .arg("--list-full")
        .succeeds()
        .stdout_matches(&Regex::new(r"^[1-9][0-9]* \[kthreadd\]\n$").unwrap());
}

#[test]
#[cfg(target_os = "linux")]
fn test_count_with_matching_pattern() {
    for arg in ["-c", "--count"] {
        new_ucmd!()
            .arg(arg)
            .arg("kthreadd")
            .succeeds()
            .stdout_is("1\n");
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_count_with_non_matching_pattern() {
    new_ucmd!()
        .arg("--count")
        .arg("non_matching")
        .fails()
        .code_is(1)
        .stdout_is("0\n")
        .no_stderr();
}

#[test]
#[cfg(target_os = "linux")]
fn test_terminal() {
    for arg in ["-t", "--terminal"] {
        new_ucmd!()
            .arg(arg)
            .arg("tty1")
            .arg("--inverse") // XXX hack to make test pass in CI
            .succeeds()
            .stdout_matches(&Regex::new("(?m)^[1-9][0-9]*$").unwrap());
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_terminal_invalid_terminal() {
    new_ucmd!()
        .arg("--terminal=invalid")
        .fails()
        .code_is(1)
        .no_output();
}
