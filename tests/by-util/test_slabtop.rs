// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use uutests::util::run_ucmd_as_root;

use uutests::new_ucmd;
use uutests::util::TestScenario;
use uutests::util_name;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_help() {
    new_ucmd!().arg("--help").succeeds().code_is(0);
}

#[cfg(target_os = "linux")]
#[test]
fn test_without_args_as_non_root() {
    new_ucmd!()
        .fails()
        .code_is(1)
        .stderr_contains("Permission denied");
}

// TODO: tests some temporary behavior; in the future a TUI should be used
// if there are no args
#[cfg(target_os = "linux")]
#[test]
fn test_without_args_as_root() {
    let ts = TestScenario::new(util_name!());

    if let Ok(result) = run_ucmd_as_root(&ts, &[]) {
        result
            .success()
            .stdout_contains("Active / Total Objects")
            .stdout_contains("OBJS");
    } else {
        print!("Test skipped; requires root user");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn test_once_as_non_root() {
    for arg in ["-o", "--once"] {
        new_ucmd!()
            .arg(arg)
            .fails()
            .code_is(1)
            .stderr_contains("Permission denied");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn test_once_as_root() {
    let ts = TestScenario::new(util_name!());

    for arg in ["-o", "--once"] {
        if let Ok(result) = run_ucmd_as_root(&ts, &[arg]) {
            result
                .success()
                .stdout_contains("Active / Total Objects")
                .stdout_contains("OBJS");
        } else {
            print!("Test skipped; requires root user");
        }
    }
}
