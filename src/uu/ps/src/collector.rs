// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::ArgMatches;
use libc::pid_t;
use nix::errno::Errno;
use std::{cell::RefCell, path::PathBuf, rc::Rc, str::FromStr};
use uu_pgrep::process::ProcessInformation;

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

// I'm guessing it matches the current terminal
pub(crate) fn basic_collector(
    proc_snapshot: &[Rc<RefCell<ProcessInformation>>],
) -> Vec<Rc<RefCell<ProcessInformation>>> {
    let mut result = Vec::new();

    let current_tty = {
        // SAFETY: The `libc::getpid` always return i32
        let proc_path =
            PathBuf::from_str(&format!("/proc/{}/", unsafe { libc::getpid() })).unwrap();
        let mut current_proc_info = ProcessInformation::try_new(proc_path).unwrap();

        current_proc_info.ttys().unwrap_or_default()
    };

    for proc_info in proc_snapshot {
        let proc_ttys = proc_info.borrow_mut().ttys().unwrap();

        if proc_ttys.iter().any(|it| current_tty.contains(it)) {
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

    // session id
    let session_id = |pid: i32| getsid(pid);

    // flag `-d`
    if matches.get_flag("d") {
        proc_snapshot.iter().for_each(|it| {});
    }

    // flag `-a`
    if matches.get_flag("a") {
        proc_snapshot.iter().for_each(|it| {});
    }

    result
}
