// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{cell::RefCell, collections::LinkedList, rc::Rc};
use uu_pgrep::process::{ProcessInformation, Teletype};

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
            "cmd" | "ucmd" => pickers.push_back(helper(cmd)),
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
    format!("{}", proc_info.borrow().pid)
}

fn tty(proc_info: RefMutableProcInfo) -> String {
    match proc_info
        .borrow_mut()
        .ttys()
        .unwrap()
        .iter()
        .collect::<Vec<_>>()
        .first()
        .unwrap()
    {
        Teletype::Tty(tty) => format!("tty{}", tty),
        Teletype::TtyS(ttys) => format!("ttyS{}", ttys),
        Teletype::Pts(pts) => format!("pts/{}", pts),
        Teletype::Unknown => "?".to_owned(),
    }
}

fn time(_proc_info: RefMutableProcInfo) -> String {
    "TODO".into()
}

fn cmd(proc_info: RefMutableProcInfo) -> String {
    proc_info.borrow_mut().status().get("Name").unwrap().into()
}
