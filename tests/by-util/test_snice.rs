// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::util::TestScenario;

#[test]
fn test_no_args() {
    new_ucmd!().fails().code_is(1);
}

#[test]
fn test_no_process_selected() {
    new_ucmd!().arg("-u=invalid_user").fails().code_is(1);
}
