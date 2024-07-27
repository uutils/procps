use std::{cell::RefCell, collections::LinkedList, rc::Rc};
use uu_pgrep::process::ProcessInformation;

type RefMutableProcInfo = Rc<RefCell<ProcessInformation>>;

pub(crate) fn collect_picker(
    code_order: &[String],
) -> LinkedList<Box<dyn Fn(RefMutableProcInfo) -> String>> {
    let mut pickers = LinkedList::new();

    for code in code_order {
        if code == "pid" || code == "tgid" {
            pickers.push_back(helper(pid))
        }

        if code == "tname" || code == "tt" || code == "tty" {
            pickers.push_back(helper(tty))
        }

        if code == "time" || code == "cputime" {
            pickers.push_back(helper(time))
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
