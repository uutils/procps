// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uutests::new_ucmd;

// runddl32.exe has no console window, no side effects,
// and no arguments are required.
// The full path must be provided since tests are ran with clear_env
#[cfg(windows)]
const TRUE_CMD: &str = "%SYSTEMROOT%\\System32\\rundll32.exe";

#[cfg(not(windows))]
const TRUE_CMD: &str = "true";

#[cfg(windows)]
const ECHO_HELLO_CMD: &str = "echo | set /p dummyName=hello";

#[cfg(not(windows))]
const ECHO_HELLO_CMD: &str = "printf hello";

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_invalid_interval() {
    let args = vec!["-n", "definitely-not-valid", TRUE_CMD];
    new_ucmd!()
        .args(&args)
        .fails()
        .stderr_contains("Invalid argument");
}

#[test]
fn test_no_interval() {
    let mut p = new_ucmd!().arg(TRUE_CMD).run_no_wait();
    p.make_assertion_with_delay(500).is_alive();
    p.kill()
        .make_assertion()
        .with_all_output()
        .no_stderr()
        .no_stdout();
}

#[test]
fn test_valid_interval() {
    let args = vec!["-n", "1.5", TRUE_CMD];
    let mut p = new_ucmd!().args(&args).run_no_wait();
    p.make_assertion_with_delay(500).is_alive();
    p.kill()
        .make_assertion()
        .with_all_output()
        .no_stderr()
        .no_stdout();
}

#[test]
fn test_valid_interval_comma() {
    let args = vec!["-n", "1,5", TRUE_CMD];
    let mut p = new_ucmd!().args(&args).run_no_wait();
    p.make_assertion_with_delay(1000).is_alive();
    p.kill()
        .make_assertion()
        .with_all_output()
        .no_stderr()
        .no_stdout();
}

#[test]
fn test_interval_environment_variable() {
    let mut p = new_ucmd!()
        .arg(ECHO_HELLO_CMD)
        .env("WATCH_INTERVAL", "0.3")
        .run_no_wait();
    // With 0.5 seconds runtime, the watched command is called twice if
    // `WATCH_INTERVAL` (0.3 seconds) is taken into account, but only once if the default
    // interval (2 seconds) is used.
    p.make_assertion_with_delay(500).is_alive();
    p.kill()
        .make_assertion()
        .with_all_output()
        .no_stderr()
        .stdout_is_bytes(b"hellohello");
}
