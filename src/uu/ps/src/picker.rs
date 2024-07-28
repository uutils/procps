// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{cell::RefCell, collections::LinkedList, rc::Rc};
use uu_pgrep::process::ProcessInformation;

type RefMutableProcInfo = Rc<RefCell<ProcessInformation>>;

pub(crate) fn collect_picker(
    code_order: &[String],
) -> LinkedList<Box<dyn Fn(RefMutableProcInfo) -> String>> {
    let mut pickers = LinkedList::new();

    for code in code_order {
        match code.as_str() {
            "pid" | "tgid" => pickers.push_back(helper(pid)),
            "tname" | "tt" | "tty" => pickers.push_back(helper(tty)),
            "time" | "cputime" => pickers.push_back(helper(time)),
            _ => {}
        }
    }

    pickers
}

#[inline]
fn helper(
    f: impl Fn(RefMutableProcInfo) -> String + 'static,
) -> Box<dyn Fn(RefMutableProcInfo) -> String> {
    Box::new(f)
}

fn pid(proc_info: RefMutableProcInfo) -> String {
    todo!()
}

fn tty(proc_info: RefMutableProcInfo) -> String {
    todo!()
}

fn time(proc_info: RefMutableProcInfo) -> String {
    todo!()
}
