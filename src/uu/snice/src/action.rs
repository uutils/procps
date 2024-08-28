// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::priority::Priority;
use std::{
    ffi::OsStr,
    fmt::{self, Display, Formatter},
    os::unix::ffi::OsStrExt,
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
            .processes_by_name(OsStr::from_bytes(cmd.as_bytes()))
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

pub(crate) fn perform_action(pids: &[u32], prio: &Priority) -> Vec<Option<ActionResult>> {
    todo!()
}
