// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(unix)]
use crate::common::util::TestScenario;

#[cfg(target_os = "linux")]
#[test]
fn test_missing_expression() {
    new_ucmd!().fails().no_stdout().code_is(1).stderr_contains(
        "the following required arguments were not provided:
  <expression>...",
    );
}
#[cfg(target_os = "linux")]
#[test]
fn test_default_signal() {
    new_ucmd!()
        .arg("-nv") // no action + verbose
        .arg("1")
        .succeeds()
        .stdout_contains("Would send signal TERM to process 1");
}

#[cfg(target_os = "linux")]
#[test]
fn test_invalid_signal() {
    new_ucmd!()
        .arg("-INVALID")
        .fails()
        .code_is(2)
        .no_stdout()
        .stderr_contains("Invalid signal: INVALID");
}
#[cfg(target_os = "linux")]
#[test]
fn test_mutiple_signal() {
    new_ucmd!()
        .arg("-HUP")
        .arg("-TERM")
        .fails()
        .code_is(2)
        .no_stdout()
        .stderr_contains("Too many signals");
}
#[cfg(target_os = "linux")]
#[test]
fn test_verbose_option() {
    new_ucmd!()
        .arg("-n") // no action
        .arg("-v")
        .arg("1")
        .succeeds()
        .stdout_contains("Would send signal TERM to process 1")
        .no_stderr();
}
#[cfg(target_os = "linux")]
#[test]
fn test_list_option() {
    new_ucmd!()
        .arg("-l")
        .succeeds()
        .no_stderr()
        .stdout_contains("HUP INT QUIT ILL TRAP ABRT BUS FPE KILL USR1 SEGV USR2 PIPE ALRM TERM STKFLT CHLD CONT STOP TSTP TTIN TTOU URG XCPU XFSZ VTALRM PROF WINCH POLL PWR SYS");
}

#[cfg(target_os = "linux")]
#[test]
fn test_list_option_long() {
    new_ucmd!()
        .arg("--list")
        .succeeds()
        .no_stderr()
        .stdout_contains("HUP INT QUIT ILL TRAP ABRT BUS FPE KILL USR1 SEGV USR2 PIPE ALRM TERM STKFLT CHLD CONT STOP TSTP TTIN TTOU URG XCPU XFSZ VTALRM PROF WINCH POLL PWR SYS");
}

#[cfg(target_os = "linux")]
#[test]
fn test_table_option() {
    new_ucmd!()
        .arg("-L")
        .succeeds()
        .no_stderr()
        .stdout_contains("1 HUP     2 INT     3 QUIT    4 ILL     5 TRAP    6 ABRT    7 BUS")
        .stdout_contains("8 FPE     9 KILL   10 USR1   11 SEGV   12 USR2   13 PIPE   14 ALRM")
        .stdout_contains("15 TERM   16 STKFLT 17 CHLD   18 CONT   19 STOP   20 TSTP   21 TTIN")
        .stdout_contains("22 TTOU   23 URG    24 XCPU   25 XFSZ   26 VTALRM 27 PROF   28 WINCH")
        .stdout_contains("29 POLL   30 PWR    31 SYS");
}

#[cfg(target_os = "linux")]
#[test]
fn test_table_option_long() {
    new_ucmd!()
        .arg("--table")
        .succeeds()
        .no_stderr()
        .stdout_contains("1 HUP     2 INT     3 QUIT    4 ILL     5 TRAP    6 ABRT    7 BUS")
        .stdout_contains("8 FPE     9 KILL   10 USR1   11 SEGV   12 USR2   13 PIPE   14 ALRM")
        .stdout_contains("15 TERM   16 STKFLT 17 CHLD   18 CONT   19 STOP   20 TSTP   21 TTIN")
        .stdout_contains("22 TTOU   23 URG    24 XCPU   25 XFSZ   26 VTALRM 27 PROF   28 WINCH")
        .stdout_contains("29 POLL   30 PWR    31 SYS");
}

#[cfg(target_os = "linux")]
#[test]
fn test_mutiple_options() {
    new_ucmd!()
        .arg("-nv") // no action + verbose
        .arg("1")
        .succeeds()
        .stdout_contains("Would send signal TERM to process 1");
}

#[cfg(target_os = "linux")]
#[test]
fn test_command_option() {
    use std::process::Command;

    Command::new("sleep")
        .arg("5")
        .spawn()
        .expect("Failed to start sleep process");

    new_ucmd!()
        .arg("-n") // no action
        .arg("-c")
        .arg("sleep")
        .succeeds();
}

#[cfg(target_os = "linux")]
#[test]
fn test_user_option() {
    use std::process::Command;
    let output = Command::new("whoami")
        .output()
        .expect("Failed to execute whoami");
    let current_user = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 output")
        .trim()
        .to_string();

    new_ucmd!()
        .arg("-n") // no action
        .arg("-u")
        .arg(&current_user)
        .succeeds()
        .no_stderr();
}
