use crate::common::util::TestScenario;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_slabtop() {
    new_ucmd!().arg("--help").succeeds().code_is(0);
}
