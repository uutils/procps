// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::ArgMatches;
use libc::pid_t;
use nix::errno::Errno;
use std::{cell::RefCell, path::PathBuf, rc::Rc, str::FromStr};
use uu_pgrep::process::{ProcessInformation, Teletype};

// TODO: Temporary add to this file, this function will add to uucore.
#[cfg(not(target_os = "redox"))]
fn getsid(pid: i32) -> Result<pid_t, Errno> {
    unsafe {
        let result = libc::getsid(pid);
        if Errno::last() == Errno::UnknownErrno {
            Ok(result)
        } else {
            Err(Errno::last())
        }
    }
}

// Guessing it matches the current terminal
pub(crate) fn basic_collector(
    proc_snapshot: &[Rc<RefCell<ProcessInformation>>],
) -> Vec<Rc<RefCell<ProcessInformation>>> {
    let mut result = Vec::new();

    let current_tty = {
        // SAFETY: The `libc::getpid` always return i32
        let proc_path =
            PathBuf::from_str(&format!("/proc/{}/", unsafe { libc::getpid() })).unwrap();
        let current_proc_info = ProcessInformation::try_new(proc_path).unwrap();

        current_proc_info.tty()
    };

    for proc_info in proc_snapshot {
        let proc_ttys = proc_info.borrow().tty();

        if proc_ttys == current_tty {
            result.push(proc_info.clone())
        }
    }

    result
}

/// Filter for processes
///
/// - `-A` Select all processes.  Identical to `-e`.
pub(crate) fn process_collector(
    matches: &ArgMatches,
    proc_snapshot: &[Rc<RefCell<ProcessInformation>>],
) -> Vec<Rc<RefCell<ProcessInformation>>> {
    let mut result = Vec::new();

    // flag `-A`
    if matches.get_flag("A") {
        result.extend(proc_snapshot.iter().map(Rc::clone))
    }

    result
}

/// Filter for session
///
/// - `-d` Select all processes except session leaders.
/// - `-a` Select all processes except both session leaders (see getsid(2)) and processes not associated with a terminal.
pub(crate) fn session_collector(
    matches: &ArgMatches,
    proc_snapshot: &[Rc<RefCell<ProcessInformation>>],
) -> Vec<Rc<RefCell<ProcessInformation>>> {
    let mut result = Vec::new();

    let tty = |proc: &Rc<RefCell<ProcessInformation>>| proc.borrow_mut().tty();

    // flag `-d`
    // Guessing it pid=sid, and all
    if matches.get_flag("d") {
        proc_snapshot.iter().for_each(|_| {});
    }

    // flag `-a`
    // Guessing it pid=sid, and associated terminal.
    if matches.get_flag("a") {
        proc_snapshot.iter().for_each(|it| {
            let pid = it.borrow().pid;

            if let Ok(sid) = getsid(pid as i32) {
                // Check is session leader
                if sid != (pid as i32) && tty(it) != Teletype::Unknown {
                    result.push(it.clone())
                }
            }
        });
    }

    result
}
