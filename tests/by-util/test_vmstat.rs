// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use std::time::Duration;
use uutests::new_ucmd;
use uutests::util::TestScenario;
use uutests::util_name;

#[test]
fn test_simple() {
    new_ucmd!().succeeds();
}

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_invalid_number() {
    new_ucmd!().arg("-1").fails().code_is(1);
    new_ucmd!().arg("0").fails().code_is(1);
}

#[test]
fn test_unit() {
    new_ucmd!().args(&["-S", "M"]).succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn test_invalid_unit() {
    new_ucmd!().args(&["-S", "x"]).fails().code_is(1);
}

#[test]
#[cfg(target_os = "linux")]
fn test_header() {
    let result = new_ucmd!().succeeds();
    assert!(result.stdout_str().starts_with("procs"));
}

#[test]
#[cfg(target_os = "linux")]
fn test_wide_mode() {
    let result = new_ucmd!().arg("-w").succeeds();
    assert!(result.stdout_str().starts_with("--procs--"));
}

#[test]
#[cfg(target_os = "linux")]
fn test_no_first() {
    let time = std::time::Instant::now();
    new_ucmd!().arg("-y").succeeds();
    assert!(time.elapsed() >= Duration::from_secs(1));
}

#[test]
#[cfg(target_os = "linux")]
fn test_active() {
    let result = new_ucmd!().arg("-a").succeeds();
    assert!(result
        .stdout_str()
        .lines()
        .nth(1)
        .unwrap()
        .contains("active"));
}
