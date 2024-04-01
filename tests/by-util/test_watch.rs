// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore (words) symdir somefakedir

use crate::common::util::TestScenario;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_invalid_interval() {
    let args = vec!["-n", "definitely-not-valid", "true"];
    new_ucmd!()
        .args(&args)
        .fails()
        .stderr_contains("Invalid argument");
}

#[test]
fn test_no_interval() {
    let mut p = new_ucmd!().arg("true").run_no_wait();
    p.make_assertion_with_delay(500).is_alive();
    p.kill()
        .make_assertion()
        .with_all_output()
        .no_stderr()
        .no_stdout();
}

#[test]
fn test_valid_interval() {
    let args = vec!["-n", "1.5", "true"];
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
    let args = vec!["-n", "1,5", "true"];
    let mut p = new_ucmd!().args(&args).run_no_wait();
    p.make_assertion_with_delay(1000).is_alive();
    p.kill()
        .make_assertion()
        .with_all_output()
        .no_stderr()
        .no_stdout();
}
