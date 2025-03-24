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
        .arg("-v")
        .arg("-WINCH") // will not produce side effects
        .arg("123")
        .succeeds()
        .stdout_contains("Would send signal WINCH")
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
fn test_table_option() {
    new_ucmd!().arg("-L")
    .succeeds()
    .no_stderr()
    .stdout_contains("1 HUP     2 INT     3 QUIT    4 ILL     5 TRAP    6 ABRT    7 BUS")
    .stdout_contains("8 FPE     9 KILL   10 USR1   11 SEGV   12 USR2   13 PIPE   14 ALRM")
    .stdout_contains("15 TERM   16 STKFLT 17 CHLD   18 CONT   19 STOP   20 TSTP   21 TTIN")
    .stdout_contains("22 TTOU   23 URG    24 XCPU   25 XFSZ   26 VTALRM 27 PROF   28 WINCH")
    .stdout_contains("29 POLL   30 PWR    31 SYS");
}
