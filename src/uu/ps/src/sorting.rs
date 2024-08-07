// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::ArgMatches;
use std::{cell::RefCell, rc::Rc};
use uu_pgrep::process::ProcessInformation;

// TODO: Implementing sorting flags.
pub(crate) fn sort(input: &mut [Rc<RefCell<ProcessInformation>>], _matches: &ArgMatches) {
    sort_by_pid(input)
}

/// Sort by pid. (Default)
fn sort_by_pid(input: &mut [Rc<RefCell<ProcessInformation>>]) {
    input.sort_by(|a, b| a.borrow().pid.cmp(&b.borrow().pid))
}
