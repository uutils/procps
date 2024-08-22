// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_conflict_arg() {
    new_ucmd!().arg("-p 0").arg("-U 0").fails().code_is(1);
}

#[test]
fn test_flag_user() {
    let binding = new_ucmd!().arg("-U=root").succeeds();
    let output = binding.code_is(0).stderr_str();

    assert!(output
        .lines()
        .map(|it| it.split_whitespace().collect::<Vec<_>>())
        .filter(|it| it[0].parse::<u32>().is_ok())
        .all(|it| it[1] == "root"));
}
