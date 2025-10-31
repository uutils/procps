// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::cell::RefCell;

use uu_pgrep::process::{ProcessInformation, Teletype};
#[cfg(unix)]
use uucore::entries::{gid2grp, uid2usr};

#[cfg(not(unix))]
fn uid2usr(id: u32) -> Result<String, std::io::Error> {
    Ok(id.to_string())
}

#[cfg(not(unix))]
fn gid2grp(id: u32) -> Result<String, std::io::Error> {
    Ok(id.to_string())
}

pub(crate) fn collect_pickers(
    code_order: &[String],
) -> Vec<Box<dyn Fn(RefCell<ProcessInformation>) -> String>> {
    let mut pickers = Vec::new();

    for code in code_order {
        match code.as_str() {
            "pid" | "tgid" => pickers.push(helper(pid)),
            "ppid" => pickers.push(helper(ppid)),
            "uid" | "euid" => pickers.push(helper(euid)),
            "ruid" => pickers.push(helper(ruid)),
            "suid" => pickers.push(helper(suid)),
            "uid_hack" | "user" | "euser" => pickers.push(helper(euser)),
            "ruser" => pickers.push(helper(ruser)),
            "suser" => pickers.push(helper(suser)),
            "pgid" => pickers.push(helper(pgid)),
            "sid" | "sess" => pickers.push(helper(sid)),
            "gid" | "egid" => pickers.push(helper(egid)),
            "rgid" => pickers.push(helper(rgid)),
            "sgid" => pickers.push(helper(sgid)),
            "group" | "egroup" => pickers.push(helper(egroup)),
            "rgroup" => pickers.push(helper(rgroup)),
            "sgroup" => pickers.push(helper(sgroup)),
            "pending" => pickers.push(helper(pending)),
            "blocked" => pickers.push(helper(blocked)),
            "ignored" => pickers.push(helper(ignored)),
            "caught" => pickers.push(helper(caught)),
            "tname" | "tt" | "tty" => pickers.push(helper(tty)),
            "time" | "cputime" => pickers.push(helper(time)),
            "ucmd" | "comm" => pickers.push(helper(ucmd)),
            "cmd" | "command" | "args" => pickers.push(helper(cmd)),
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
    proc_info.borrow().pid.to_string()
}

fn ppid(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().ppid().unwrap().to_string()
}

fn ruid(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().uid().unwrap().to_string()
}

fn euid(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().euid().unwrap().to_string()
}

fn suid(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().suid().unwrap_or(0).to_string()
}

fn ruser(proc_info: RefCell<ProcessInformation>) -> String {
    let uid = proc_info.borrow_mut().uid().unwrap();
    uid2usr(uid).ok().unwrap_or_else(|| uid.to_string())
}

fn euser(proc_info: RefCell<ProcessInformation>) -> String {
    let euid = proc_info.borrow_mut().euid().unwrap();
    uid2usr(euid).ok().unwrap_or_else(|| euid.to_string())
}

fn suser(proc_info: RefCell<ProcessInformation>) -> String {
    let suid = proc_info.borrow_mut().suid().unwrap_or(0);
    uid2usr(suid).unwrap_or_else(|_| suid.to_string())
}

fn rgid(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().gid().unwrap().to_string()
}

fn egid(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().egid().unwrap().to_string()
}

fn sgid(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().sgid().unwrap_or(0).to_string()
}

fn rgroup(proc_info: RefCell<ProcessInformation>) -> String {
    let gid = proc_info.borrow_mut().gid().unwrap();
    gid2grp(gid).ok().unwrap_or_else(|| gid.to_string())
}

fn egroup(proc_info: RefCell<ProcessInformation>) -> String {
    let egid = proc_info.borrow_mut().egid().unwrap();
    gid2grp(egid).ok().unwrap_or_else(|| egid.to_string())
}

fn sgroup(proc_info: RefCell<ProcessInformation>) -> String {
    let sgid = proc_info.borrow_mut().sgid().unwrap_or(0);
    gid2grp(sgid).unwrap_or_else(|_| sgid.to_string())
}

fn pgid(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().pgid().unwrap().to_string()
}

fn sid(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().sid().unwrap().to_string()
}

fn tty(proc_info: RefCell<ProcessInformation>) -> String {
    match proc_info.borrow_mut().tty() {
        Teletype::Known(device_path) => device_path
            .strip_prefix("/dev/")
            .unwrap_or(&device_path)
            .to_owned(),
        Teletype::Unknown => "?".to_owned(),
    }
}

fn time(proc_info: RefCell<ProcessInformation>) -> String {
    // https://docs.kernel.org/filesystems/proc.html#id10
    // Index of 13 14

    let cumulative_cpu_time = {
        let utime = proc_info.borrow_mut().stat()[13].parse::<i64>().unwrap();
        let stime = proc_info.borrow_mut().stat()[14].parse::<i64>().unwrap();
        (utime + stime) / 100
    };

    format_time(cumulative_cpu_time)
}

fn format_time(seconds: i64) -> String {
    let day = seconds / (3600 * 24);
    let hour = (seconds % (3600 * 24)) / 3600;
    let minute = (seconds % 3600) / 60;
    let second = seconds % 60;

    if day != 0 {
        format!("{day:02}-{hour:02}:{minute:02}:{second:02}")
    } else {
        format!("{hour:02}:{minute:02}:{second:02}")
    }
}

fn cmd(proc_info: RefCell<ProcessInformation>) -> String {
    // Use command line if available, otherwise show process name in brackets (for kernel threads)
    let cmdline = proc_info.borrow().cmdline.clone();
    if !cmdline.is_empty() {
        cmdline
    } else {
        format!("[{}]", proc_info.borrow_mut().name().unwrap())
    }
}

fn ucmd(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info.borrow_mut().name().unwrap()
}

fn pending(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info
        .borrow_mut()
        .signals_pending_mask()
        .map(|mask| format!("{mask:016x}"))
        .unwrap_or_else(|_| "?".to_string())
}

fn blocked(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info
        .borrow_mut()
        .signals_blocked_mask()
        .map(|mask| format!("{mask:016x}"))
        .unwrap_or_else(|_| "?".to_string())
}

fn ignored(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info
        .borrow_mut()
        .signals_ignored_mask()
        .map(|mask| format!("{mask:016x}"))
        .unwrap_or_else(|_| "?".to_string())
}

fn caught(proc_info: RefCell<ProcessInformation>) -> String {
    proc_info
        .borrow_mut()
        .signals_caught_mask()
        .map(|mask| format!("{mask:016x}"))
        .unwrap_or_else(|_| "?".to_string())
}

#[test]
fn test_time() {
    let formatted = {
        let time = {
            let utime = 29i64;
            let stime = 18439i64;
            (utime + stime) / 100
        };
        format_time(time)
    };
    assert_eq!(formatted, "00:03:04");

    let formatted = {
        let time = {
            let utime = 12345678i64;
            let stime = 90i64;
            (utime + stime) / 100
        };
        format_time(time)
    };
    assert_eq!(formatted, "01-10:17:37");
}
