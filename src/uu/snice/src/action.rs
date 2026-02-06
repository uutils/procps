// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::ask_user;
use crate::priority::Priority;
use rustix::io::Errno;
use rustix::process::{getpriority_process, setpriority_process, Pid};
use std::{
    fmt::{self, Display, Formatter},
    sync::OnceLock,
};
use sysinfo::{System, Users};
use uu_pgrep::process::Teletype;

pub(crate) fn process_snapshot() -> &'static sysinfo::System {
    static SNAPSHOT: OnceLock<System> = OnceLock::new();

    SNAPSHOT.get_or_init(System::new_all)
}

pub(crate) fn users() -> &'static Users {
    static SNAPSHOT: OnceLock<Users> = OnceLock::new();

    SNAPSHOT.get_or_init(Users::new_with_refreshed_list)
}

#[derive(Debug)]
pub enum SelectedTarget {
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
        use uu_pgrep::process::ProcessInformation;

        process_snapshot()
            .processes()
            .iter()
            .filter(|(pid, _)| {
                let pid = pid.as_u32();

                ProcessInformation::from_pid(pid as usize).unwrap().tty() == *tty
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ActionResult {
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
/// But we don't know if pid is an existing process. Returns [None] if the process doesn't exist
fn set_priority(pid: u32, prio: &Priority, take_action: bool) -> Option<ActionResult> {
    let pid = Pid::from_raw(i32::try_from(pid).ok()?);
    let process_priority = getpriority_process(pid);

    // Expected errors:
    // https://man7.org/linux/man-pages/man2/setpriority.2.html
    // https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setpriority.2.html
    // ESRCH: no matching process
    // EACCES: no permission to lower priority
    // EPERM: wrong user for process
    let current_priority = match process_priority {
        Err(Errno::SRCH) => return None,
        Err(_) => {
            return Some(ActionResult::PermissionDenied);
        }
        Ok(priority) => priority,
    };

    if !take_action {
        return Some(ActionResult::Success);
    }

    let new_priority = match prio {
        Priority::Increase(prio) => current_priority + *prio as i32,
        Priority::Decrease(prio) => current_priority - *prio as i32,
        Priority::To(prio) => *prio as i32,
    };

    let result = setpriority_process(pid, new_priority);
    match result {
        Err(Errno::SRCH) => None,
        Err(_) => Some(ActionResult::PermissionDenied),
        Ok(_) => Some(ActionResult::Success),
    }
}

pub(crate) fn perform_action(
    pids: &[u32],
    prio: &Priority,
    take_action: bool,
    ask: bool,
) -> Vec<Option<ActionResult>> {
    let f = |pid: &u32| {
        if !ask || ask_user(*pid) {
            set_priority(*pid, prio, take_action)
        } else {
            // won't be used, but we need to return (not None)
            Some(ActionResult::Success)
        }
    };
    pids.iter().map(f).collect()
}
