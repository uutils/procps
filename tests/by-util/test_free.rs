// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
// spell-checker:ignore (words) symdir somefakedir

use pretty_assertions::assert_eq;
use regex::Regex;

use crate::common::util::TestScenario;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_free() {
    let result = new_ucmd!().succeeds();
    assert!(result.stdout_str().contains("Mem:"))
}

#[test]
fn test_free_wide() {
    let result = new_ucmd!().arg("--wide").succeeds();
    assert!(result.stdout_str().contains("Mem:"));
    assert!(!result.stdout_str().contains("buff/cache"));
}

#[test]
fn test_free_column_format() {
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
    let mut free_lines = free_result.split("\n");
    for re in re_list {
        assert!(re.is_match(free_lines.next().unwrap()));
    }
}

#[test]
fn test_free_wide_column_format() {
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
    let mut free_lines = free_result.split("\n");
    for re in re_list {
        assert!(re.is_match(free_lines.next().unwrap()));
    }
}
