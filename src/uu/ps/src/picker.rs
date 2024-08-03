// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use chrono::DateTime;
use std::{cell::RefCell, collections::LinkedList, rc::Rc};
use uu_pgrep::process::{ProcessInformation, Teletype};

type RefMutableProcInfo = Rc<RefCell<ProcessInformation>>;

pub(crate) fn collect_pickers(
    code_order: &[String],
) -> LinkedList<Box<dyn Fn(RefMutableProcInfo) -> String>> {
    let mut pickers = LinkedList::new();

    for code in code_order {
        match code.as_str() {
            "pid" | "tgid" => pickers.push_back(helper(pid)),
            "tname" | "tt" | "tty" => pickers.push_back(helper(tty)),
            "time" | "cputime" => pickers.push_back(helper(time)),
            "ucmd" => pickers.push_back(helper(ucmd)),
            "cmd" => pickers.push_back(helper(cmd)),
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
    match proc_info.borrow().tty() {
        Teletype::Tty(tty) => format!("tty{}", tty),
        Teletype::TtyS(ttys) => format!("ttyS{}", ttys),
        Teletype::Pts(pts) => format!("pts/{}", pts),
        Teletype::Unknown => "?".to_owned(),
    }
}

fn time(proc_info: RefMutableProcInfo) -> String {
    // https://docs.kernel.org/filesystems/proc.html#id10
    // Index of 13 14

    let cumulative_cpu_time = {
        let utime = proc_info.borrow_mut().stat()[13].parse::<i64>().unwrap();
        let stime = proc_info.borrow_mut().stat()[14].parse::<i64>().unwrap();
        utime + stime
    };

    DateTime::from_timestamp_millis(cumulative_cpu_time)
        .unwrap()
        .format("%H:%M:%S")
        .to_string()
}

fn cmd(proc_info: RefMutableProcInfo) -> String {
    proc_info.borrow_mut().cmdline.clone()
}

fn ucmd(proc_info: RefMutableProcInfo) -> String {
    proc_info.borrow_mut().status().get("Name").unwrap().into()
}
