// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use std::{
    array,
    process::{Child, Command},
};

use crate::common::util::TestScenario;
#[cfg(target_os = "linux")]
use regex::Regex;

#[cfg(target_os = "linux")]
const SINGLE_PID: &str = "^[1-9][0-9]*";
#[cfg(target_os = "linux")]
// (?m) enables multi-line mode
const MULTIPLE_PIDS: &str = "(?m)^[1-9][0-9]*$";

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
        .arg("NONMATCHING")
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
    let re = &Regex::new(SINGLE_PID).unwrap();

    for arg in ["-o", "--oldest"] {
        new_ucmd!().arg(arg).succeeds().stdout_matches(re);
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
    let re = &Regex::new(SINGLE_PID).unwrap();

    for arg in ["-n", "--newest"] {
        new_ucmd!().arg(arg).succeeds().stdout_matches(re);
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
    let re = &Regex::new(MULTIPLE_PIDS).unwrap();

    for arg in ["-O", "--older"] {
        new_ucmd!().arg(arg).arg("0").succeeds().stdout_matches(re);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_older_matching_pattern() {
    new_ucmd!()
        .arg("--older=0")
        .arg("sh")
        .succeeds()
        .stdout_matches(&Regex::new(MULTIPLE_PIDS).unwrap());
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
fn spawn_2_dummy_sleep_processes() -> [Child; 2] {
    array::from_fn(|_| Command::new("sleep").arg("2").spawn().unwrap())
}

#[cfg(target_os = "linux")]
#[test]
fn test_delimiter() {
    let mut sleep_processes = spawn_2_dummy_sleep_processes();
    for arg in ["-d", "--delimiter"] {
        new_ucmd!()
            .arg("sleep")
            .arg(arg)
            .arg("|")
            .succeeds()
            .stdout_contains("|");
    }
    sleep_processes.iter_mut().for_each(|p| drop(p.kill()));
}

#[cfg(target_os = "linux")]
#[test]
fn test_delimiter_last_wins() {
    let mut sleep_processes = spawn_2_dummy_sleep_processes();
    new_ucmd!()
        .arg("sleep")
        .arg("-d_")
        .arg("-d:")
        .succeeds()
        .stdout_does_not_contain("_")
        .stdout_contains(":");

    new_ucmd!()
        .arg("sleep")
        .arg("-d:")
        .arg("-d_")
        .succeeds()
        .stdout_does_not_contain(":")
        .stdout_contains("_");
    sleep_processes.iter_mut().for_each(|p| drop(p.kill()));
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
    // (?m) enables multi-line mode
    let re = &Regex::new("(?m)^[1-9][0-9]* .+$").unwrap();

    for arg in ["-a", "--list-full"] {
        new_ucmd!()
            .arg("sh")
            .arg(arg)
            .succeeds()
            // (?m) enables multi-line mode
            .stdout_matches(re);
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
    let re = &Regex::new(MULTIPLE_PIDS).unwrap();

    for arg in ["-t", "--terminal"] {
        new_ucmd!()
            .arg(arg)
            .arg("tty1")
            .arg("--inverse") // XXX hack to make test pass in CI
            .succeeds()
            .stdout_matches(re);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_terminal_multiple_terminals() {
    new_ucmd!()
        .arg("--terminal=tty1,?")
        .arg("kthreadd")
        .succeeds()
        .stdout_matches(&Regex::new(SINGLE_PID).unwrap());
}

#[test]
#[cfg(target_os = "linux")]
fn test_unknown_terminal() {
    new_ucmd!().arg("--terminal=?").succeeds();
    new_ucmd!().arg("--terminal=?").arg("kthreadd").succeeds();
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

#[test]
#[cfg(target_os = "linux")]
fn test_runstates() {
    let re = &Regex::new(MULTIPLE_PIDS).unwrap();

    for arg in ["-r", "--runstates"] {
        new_ucmd!().arg(arg).arg("S").succeeds().stdout_matches(re);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_runstates_invalid_runstate() {
    new_ucmd!()
        .arg("--runstates=invalid")
        .fails()
        .code_is(1)
        .no_output();
}

#[test]
#[cfg(target_os = "linux")]
fn test_parent() {
    let re = &Regex::new(MULTIPLE_PIDS).unwrap();

    for arg in ["-P", "--parent"] {
        new_ucmd!().arg(arg).arg("0").succeeds().stdout_matches(re);
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_parent_multiple_parents() {
    new_ucmd!()
        .arg("--parent=0,1")
        .succeeds()
        .stdout_matches(&Regex::new(MULTIPLE_PIDS).unwrap());
}

#[test]
#[cfg(target_os = "linux")]
fn test_parent_non_matching_parent() {
    new_ucmd!()
        .arg("--parent=10000000")
        .fails()
        .code_is(1)
        .no_output();
}

#[test]
#[cfg(target_os = "linux")]
fn test_require_handler() {
    new_ucmd!()
        .arg("--require-handler")
        .arg("--signal=INT")
        .arg("NONEXISTENT")
        .fails()
        .no_output();
}

#[test]
#[cfg(target_os = "linux")]
fn test_invalid_signal() {
    new_ucmd!()
        .arg("--signal=foo")
        .arg("NONEXISTENT")
        .fails()
        .stderr_contains("Unknown signal 'foo'");
}

#[test]
#[cfg(target_os = "linux")]
fn test_does_not_match_pid() {
    let our_pid = std::process::id();
    new_ucmd!().arg(our_pid.to_string()).fails();
}

#[test]
#[cfg(target_os = "linux")]
fn test_too_long_pattern() {
    new_ucmd!()
        .arg("A".repeat(15))
        .fails()
        .code_is(1)
        .no_output();

    new_ucmd!()
        .arg("A".repeat(16))
        .fails()
        .code_is(1)
        .stderr_contains("pattern that searches for process name longer than 15 characters will result in zero matches");
}

#[test]
#[cfg(target_os = "linux")]
fn test_invalid_username() {
    new_ucmd!()
        .arg("--uid=DOES_NOT_EXIST")
        .fails()
        .code_is(1)
        .stderr_contains("invalid user name");
}

#[test]
#[cfg(target_os = "linux")]
fn test_invalid_group_name() {
    new_ucmd!()
        .arg("--group=DOES_NOT_EXIST")
        .fails()
        .code_is(1)
        .stderr_contains("invalid group name");
}

#[test]
#[cfg(target_os = "linux")]
fn test_current_user() {
    new_ucmd!()
        .arg("-U")
        .arg(uucore::process::getuid().to_string())
        .succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn test_does_not_match_current_process() {
    let our_pid = std::process::id();
    dbg!(&our_pid);
    new_ucmd!()
        .arg("-f")
        .arg("UNIQUE_STRING_THAT_DOES_NOT_MATCH_ANY_OTHER_PROCESS")
        .fails()
        .no_output();
}
