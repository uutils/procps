// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::cell::RefCell;

use chrono::DateTime;
use uu_pgrep::process::{ProcessInformation, Teletype};

pub(crate) fn collect_pickers(
    code_order: &[String],
) -> Vec<Box<dyn Fn(RefCell<ProcessInformation>) -> String>> {
    let mut pickers = Vec::new();

    for code in code_order {
        match code.as_str() {
            "pid" | "tgid" => pickers.push(helper(pid)),
            "tname" | "tt" | "tty" => pickers.push(helper(tty)),
            "time" | "cputime" => pickers.push(helper(time)),
            "ucmd" => pickers.push(helper(ucmd)),
            "cmd" => pickers.push(helper(cmd)),
            _ => {}
        }
    }

    pickers
}

#[inline]
fn helper(
    f: impl Fn(RefCell<ProcessInformation>) -> String + 'static,
) -> Box<dyn Fn(RefCell<ProcessInformation>) -> String> {
    Box::new(f)
}

fn pid(proc_info: RefCell<ProcessInformation>) -> String {
    format!("{}", proc_info.borrow().pid)
}

fn tty(proc_info: RefCell<ProcessInformation>) -> String {
    match proc_info.borrow().tty() {
        Teletype::Tty(tty) => format!("tty{}", tty),
        Teletype::TtyS(ttys) => format!("ttyS{}", ttys),
        Teletype::Pts(pts) => format!("pts/{}", pts),
        Teletype::Unknown => "?".to_owned(),
    }
}

fn time(proc_info: RefCell<ProcessInformation>) -> String {
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

fn cmd(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow().cmdline.clone()
}

fn ucmd(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().status().get("Name").unwrap().into()
}
