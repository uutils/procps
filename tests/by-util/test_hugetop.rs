// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use uutests::new_ucmd;

// Basic functionality tests
#[test]
#[cfg(target_os = "linux")]
fn runs_successfully() {
    new_ucmd!().succeeds();
}

// Lines option tests
#[test]
#[cfg(target_os = "linux")]
fn supports_lines_option() {
    new_ucmd!().arg("-l").arg("1").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn supports_lines_option_long() {
    new_ucmd!().arg("--lines").arg("5").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn supports_lines_option_zero() {
    new_ucmd!().arg("-l").arg("0").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn supports_lines_option_large() {
    new_ucmd!().arg("-l").arg("1000").succeeds();
}

// NUMA option tests
#[test]
#[cfg(target_os = "linux")]
fn supports_numa_option() {
    new_ucmd!().arg("-n").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn supports_numa_option_long() {
    new_ucmd!().arg("--numa").succeeds();
}

// Human-readable format tests
#[test]
#[cfg(target_os = "linux")]
fn supports_human_option() {
    new_ucmd!().arg("-H").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn supports_human_option_long() {
    new_ucmd!().arg("--human").succeeds();
}

// Once option tests
#[test]
#[cfg(target_os = "linux")]
fn supports_once_option() {
    new_ucmd!().arg("-o").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn supports_once_option_long() {
    new_ucmd!().arg("--once").succeeds();
}

// Delay option tests
#[test]
#[cfg(target_os = "linux")]
fn supports_delay_option_zero() {
    new_ucmd!().arg("-d").arg("0").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn supports_delay_option_long() {
    new_ucmd!().arg("--delay").arg("0").succeeds();
}

// Combined options tests
#[test]
#[cfg(target_os = "linux")]
fn combined_lines_and_human() {
    new_ucmd!().arg("-l").arg("5").arg("-H").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn combined_numa_and_human() {
    new_ucmd!().arg("-n").arg("-H").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn combined_lines_and_numa() {
    new_ucmd!().arg("-l").arg("3").arg("-n").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn combined_once_and_human() {
    new_ucmd!().arg("-o").arg("-H").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn combined_lines_human_and_once() {
    new_ucmd!()
        .arg("-l")
        .arg("2")
        .arg("-H")
        .arg("-o")
        .succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn combined_all_options() {
    new_ucmd!()
        .arg("-l")
        .arg("5")
        .arg("-H")
        .arg("-n")
        .arg("-o")
        .succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn combined_delay_and_once() {
    new_ucmd!().arg("-d").arg("0").arg("-o").succeeds();
}

// Long form option combinations
#[test]
#[cfg(target_os = "linux")]
fn long_form_options() {
    new_ucmd!()
        .arg("--lines")
        .arg("3")
        .arg("--human")
        .arg("--once")
        .succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn long_form_with_numa_and_delay() {
    new_ucmd!()
        .arg("--lines")
        .arg("4")
        .arg("--numa")
        .arg("--delay")
        .arg("0")
        .succeeds();
}

// Help tests
#[test]
#[cfg(target_os = "linux")]
fn help_short_flag() {
    new_ucmd!().arg("-h").succeeds();
}

#[test]
#[cfg(target_os = "linux")]
fn help_long_flag() {
    new_ucmd!().arg("--help").succeeds();
}
