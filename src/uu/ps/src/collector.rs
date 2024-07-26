use std::{cell::RefCell, rc::Rc};

use clap::ArgMatches;
use libc::pid_t;
use nix::errno::Errno;
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

/// Filter for processes
///
/// - `-A` (alias `-e`)
pub(crate) fn process_collector(
    matches: &ArgMatches,
    proc_snapshot: Vec<Rc<RefCell<ProcessInformation>>>,
) -> Vec<Rc<RefCell<ProcessInformation>>> {
    let mut result = Vec::new();

    // flag `-A`
    if matches.get_flag("A") {
        result.extend(proc_snapshot)
    }

    result
}

/// Filter for session
///
/// - `-d`
/// - `-a`
pub(crate) fn session_collector(
    matches: &ArgMatches,
    proc_snapshot: Vec<Rc<RefCell<ProcessInformation>>>,
) -> Vec<Rc<RefCell<ProcessInformation>>> {
    let mut result = Vec::new();

    // session id
    // https://docs.kernel.org/filesystems/proc.html#id10
    let session_id = |proc_info: &mut ProcessInformation| getsid(unsafe { libc::getpid() });

    // flag `-d`
    if matches.get_flag("d") {
        result.extend(proc_snapshot.clone())
    }

    // flag `-a`
    if matches.get_flag("a") {
        result.extend(proc_snapshot)
    }

    result
}

/// Filter for terminal
///
/// - `-t`
pub(crate) fn terminal_collector(
    matches: &ArgMatches,
    proc_snapshot: Vec<Rc<RefCell<ProcessInformation>>>,
) -> Vec<Rc<RefCell<ProcessInformation>>> {
    let mut result = Vec::new();

    let flag_a_collector = || {};

    result
}

// pub(crate) fn negate_select() {}
