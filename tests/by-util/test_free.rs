// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use pretty_assertions::assert_eq;
use regex::Regex;

use crate::common::util::TestScenario;

// TODO: make tests combineable (e.g. test --total --human)

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_no_args() {
    let result = new_ucmd!().succeeds();
    assert!(result.stdout_str().contains("Mem:"));
}

#[test]
fn test_wide() {
    let result = new_ucmd!().arg("--wide").succeeds();
    assert!(result.stdout_str().contains("Mem:"));
    assert!(!result.stdout_str().contains("buff/cache"));
}

#[test]
fn test_total() {
    let result = new_ucmd!().arg("-t").succeeds();
    assert_eq!(result.stdout_str().lines().count(), 4);
    assert!(result
        .stdout_str()
        .lines()
        .last()
        .unwrap()
        .starts_with("Total:"))
}

#[test]
fn test_count() {
    let result = new_ucmd!().args(&["-c", "2", "-s", "0"]).succeeds();
    assert_eq!(result.stdout_str().lines().count(), 7);
}

#[test]
fn test_lohi() {
    let result = new_ucmd!().arg("--lohi").succeeds();
    assert_eq!(result.stdout_str().lines().count(), 5);
    let lines = result.stdout_str().lines().collect::<Vec<&str>>();
    assert!(lines[2].starts_with("Low:"));
    assert!(lines[3].starts_with("High:"));
}

#[test]
fn test_committed() {
    let result = new_ucmd!().arg("-v").succeeds();
    assert_eq!(result.stdout_str().lines().count(), 4);
    assert!(result
        .stdout_str()
        .lines()
        .last()
        .unwrap()
        .starts_with("Comm:"))
}

#[test]
fn test_always_one_line() {
    // -L should ignore all other parameters and always print one line
    let result = new_ucmd!().arg("-hltvwL").succeeds();
    let stdout = result.stdout_str().lines().collect::<Vec<&str>>();
    assert_eq!(stdout.len(), 1);
    assert!(stdout[0].starts_with("SwapUse"));
}

#[test]
fn test_column_format() {
    let re_head_str = r"^ {15}total {8}used {8}free {6}shared {2}buff/cache {3}available$";
    let re_mem_str = r"^Mem:( +\d+){6}$";
    let re_swap_str = r"^Swap: ( +\d+){3}$";

    let re_list = vec![
        Regex::new(re_head_str).unwrap(),
        Regex::new(re_mem_str).unwrap(),
        Regex::new(re_swap_str).unwrap(),
    ];

    let binding = new_ucmd!().succeeds();
    let free_result = binding.stdout_str();
    assert_eq!(free_result.len(), 207);

    // Check the format for each line output
    let mut free_lines = free_result.split('\n');
    for re in re_list {
        assert!(re.is_match(free_lines.next().unwrap()));
    }
}

#[test]
fn test_wide_column_format() {
    let re_head_str = r"^ {15}total {8}used {8}free {6}shared {5}buffers {7}cache {3}available$";
    let re_mem_str = r"^Mem:( +\d+){7}$";
    let re_swap_str = r"^Swap: ( +\d+){3}$";

    let re_list = vec![
        Regex::new(re_head_str).unwrap(),
        Regex::new(re_mem_str).unwrap(),
        Regex::new(re_swap_str).unwrap(),
    ];

    let binding = new_ucmd!().arg("--wide").succeeds();
    let free_result = binding.stdout_str();

    // The total number of character is always fixed
    assert_eq!(free_result.len(), 231);

    // Check the format for each line output
    let mut free_lines = free_result.split('\n');
    for re in re_list {
        assert!(re.is_match(free_lines.next().unwrap()));
    }
}
