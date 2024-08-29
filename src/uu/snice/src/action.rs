// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::priority::Priority;
use std::{
    fmt::{self, Display, Formatter},
    sync::OnceLock,
};
use sysinfo::{System, Users};
use uu_pgrep::process::Teletype;

fn process_snapshot() -> &'static sysinfo::System {
    static SNAPSHOT: OnceLock<System> = OnceLock::new();

    SNAPSHOT.get_or_init(System::new_all)
}

fn users() -> &'static Users {
    static SNAPSHOT: OnceLock<Users> = OnceLock::new();

    SNAPSHOT.get_or_init(Users::new_with_refreshed_list)
}

#[derive(Debug)]
pub(crate) enum SelectedTarget {
    Command(String),
    Pid(u32),
    Tty(Teletype),
    User(String),
}

#[allow(unused)]
impl SelectedTarget {
    pub(crate) fn to_pids(&self) -> Vec<u32> {
        match self {
            Self::Command(cmd) => Self::from_cmd(cmd),
            Self::Pid(pid) => vec![*pid],
            Self::Tty(tty) => Self::from_tty(tty),
            Self::User(user) => Self::from_user(user),
        }
    }

    fn from_cmd(cmd: &str) -> Vec<u32> {
        process_snapshot()
            .processes_by_name(cmd.as_ref())
            .map(|it| it.pid().as_u32())
            .collect()
    }

    #[cfg(target_os = "linux")]
    fn from_tty(tty: &Teletype) -> Vec<u32> {
        use std::{path::PathBuf, str::FromStr};
        use uu_pgrep::process::ProcessInformation;

        process_snapshot()
            .processes()
            .iter()
            .filter(|(pid, _)| {
                let pid = pid.as_u32();
                let path = PathBuf::from_str(&format!("/proc/{}/", pid)).unwrap();

                ProcessInformation::try_new(path).unwrap().tty() == *tty
            })
            .map(|(pid, _)| pid.as_u32())
            .collect()
    }

    // TODO: issues:#179 https://github.com/uutils/procps/issues/179
    #[cfg(not(target_os = "linux"))]
    fn from_tty(_tty: &Teletype) -> Vec<u32> {
        Vec::new()
    }

    fn from_user(user: &str) -> Vec<u32> {
        let Some(uid) = users().iter().find(|it| it.name() == user) else {
            return Vec::new();
        };
        let uid = uid.id();

        process_snapshot()
            .processes()
            .iter()
            .filter(|(_, process)| match process.user_id() {
                Some(p_uid) => p_uid == uid,
                None => false,
            })
            .map(|(pid, _)| pid.as_u32())
            .collect()
    }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum ActionResult {
    PermissionDenied,
    Success,
}

impl Display for ActionResult {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::PermissionDenied => write!(f, "Permission Denied"),
            Self::Success => write!(f, "Success"),
        }
    }
}

/// Set priority of process.
///
/// But we don't know if the process of pid are exist, if [None], the process doesn't exist
#[cfg(target_os = "linux")]
fn set_priority(pid: u32, prio: &Priority) -> Option<ActionResult> {
    use libc::{getpriority, setpriority, PRIO_PROCESS};
    use nix::errno::Errno;

    // Very dirty.
    let current_priority = {
        // Clear errno
        Errno::clear();

        let prio = unsafe { getpriority(PRIO_PROCESS, pid) };
        // prio == -1 might be error.
        if prio == -1 && Errno::last() != Errno::UnknownErrno {
            // Must clear errno.
            Errno::clear();

            // I don't know but, just considering it just caused by permission.
            // https://manpages.debian.org/bookworm/manpages-dev/getpriority.2.en.html#ERRORS
            return match Errno::last() {
                Errno::ESRCH => Some(ActionResult::PermissionDenied),
                _ => None,
            };
        } else {
            prio
        }
    };

    let prio = match prio {
        Priority::Increase(prio) => current_priority + *prio as i32,
        Priority::Decrease(prio) => current_priority - *prio as i32,
        Priority::To(prio) => *prio as i32,
    };

    // result only 0, -1
    Errno::clear();
    let result = unsafe { setpriority(PRIO_PROCESS, pid, prio) };

    // https://manpages.debian.org/bookworm/manpages-dev/setpriority.2.en.html#ERRORS
    if result == -1 {
        match Errno::last() {
            Errno::ESRCH => Some(ActionResult::PermissionDenied),
            _ => None,
        }
    } else {
        Some(ActionResult::Success)
    }
}

// TODO: Implemented this on other platform
#[cfg(not(target_os = "linux"))]
fn set_priority(_pid: u32, _prio: &Priority) -> Option<ActionResult> {
    None
}

pub(crate) fn perform_action(pids: &[u32], prio: &Priority) -> Vec<Option<ActionResult>> {
    let f = |pid: &u32| set_priority(*pid, prio);
    pids.iter().map(f).collect()
}
