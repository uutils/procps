// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::ArgMatches;
use std::collections::HashSet;
use uu_pgrep::process::{walk_process, ProcessInformation, RunState, Teletype};
use uucore::error::UResult;

#[cfg(target_family = "unix")]
use nix::errno::Errno;

// TODO: Temporary add to this file, this function will add to uucore.
#[cfg(not(target_os = "redox"))]
#[cfg(target_family = "unix")]
fn getsid(pid: i32) -> Option<i32> {
    unsafe {
        let result = uucore::libc::getsid(pid);
        if Errno::last() == Errno::UnknownErrno {
            Some(result)
        } else {
            None
        }
    }
}

// TODO: Temporary add to this file, this function will add to uucore.
#[cfg(target_family = "windows")]
fn getsid(_pid: i32) -> Option<i32> {
    Some(0)
}

fn is_session_leader(process: &ProcessInformation) -> bool {
    let pid = process.pid as i32;
    getsid(pid) == Some(pid)
}

pub struct ProcessSelectionSettings {
    /// - `-A` Select all processes.  Identical to `-e`.
    pub select_all: bool,
    /// - `-a` Select all processes except both session leaders (see getsid(2)) and processes not associated with a terminal.
    pub select_non_session_leaders_with_tty: bool,
    /// - `-d` Select all processes except session leaders.
    pub select_non_session_leaders: bool,

    /// - `-x` Lift "must have a tty" restriction.
    pub dont_require_tty: bool,

    /// - `-C` Select by command name
    pub command_names: Option<HashSet<String>>,
    /// - `-q, --quick-pid` Quick process selection by PID
    pub quick_pids: Option<HashSet<usize>>,
    /// - `-p, --pid` Select specific process IDs
    pub pids: Option<HashSet<usize>>,
    /// - `--ppid` Select specific parent process IDs
    pub ppids: Option<HashSet<usize>>,
    /// - `--sid` Select specific session IDs
    pub sids: Option<HashSet<usize>>,
    /// - `-G, --Group` Select by real group ID or name
    pub real_groups: Option<HashSet<u32>>,
    /// - `-g, --group` Select by effective group ID or name
    pub eff_groups: Option<HashSet<u32>>,
    /// - `-U, --User` Select by real user ID or name
    pub real_users: Option<HashSet<u32>>,
    /// - `-u, --user` Select by effective user ID or name
    pub eff_users: Option<HashSet<u32>>,

    /// - `-r` Restrict the selection to only running processes.
    pub only_running: bool,

    /// - `--deselect` Negates the selection.
    pub negate_selection: bool,
}

impl ProcessSelectionSettings {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            select_all: matches.get_flag("A"),
            select_non_session_leaders_with_tty: matches.get_flag("a"),
            select_non_session_leaders: matches.get_flag("d"),
            dont_require_tty: matches.get_flag("x"),
            command_names: matches
                .get_many::<Vec<String>>("command")
                .map(|xs| xs.flatten().cloned().collect()),
            quick_pids: matches
                .get_many::<Vec<usize>>("quick-pid")
                .map(|xs| xs.flatten().copied().collect()),
            pids: matches
                .get_many::<Vec<usize>>("pid")
                .map(|xs| xs.flatten().copied().collect()),
            ppids: matches
                .get_many::<Vec<usize>>("ppid")
                .map(|xs| xs.flatten().copied().collect()),
            sids: matches
                .get_many::<Vec<usize>>("sid")
                .map(|xs| xs.flatten().copied().collect()),
            real_groups: matches
                .get_many::<Vec<u32>>("real-group")
                .map(|xs| xs.flatten().copied().collect()),
            eff_groups: matches
                .get_many::<Vec<u32>>("effective-group")
                .map(|xs| xs.flatten().copied().collect()),
            real_users: matches
                .get_many::<Vec<u32>>("real-user")
                .map(|xs| xs.flatten().copied().collect()),
            eff_users: matches
                .get_many::<Vec<u32>>("effective-user")
                .map(|xs| xs.flatten().copied().collect()),
            only_running: matches.get_flag("r"),
            negate_selection: matches.get_flag("deselect"),
        }
    }

    pub fn select_processes(self) -> UResult<Vec<ProcessInformation>> {
        if let Some(ref quick_pids) = self.quick_pids {
            let mut selected = Vec::new();
            for &pid in quick_pids {
                if let Ok(process) =
                    ProcessInformation::try_new(std::path::PathBuf::from(format!("/proc/{}", pid)))
                {
                    selected.push(process);
                }
            }
            return Ok(selected);
        }

        let mut current_process = ProcessInformation::current_process_info().unwrap();
        let current_tty = current_process.tty();
        let current_euid = current_process.euid().unwrap();

        let matches_criteria = |process: &mut ProcessInformation| -> UResult<bool> {
            if self.only_running && !process.run_state().is_ok_and(|x| x == RunState::Running) {
                return Ok(false);
            }

            if self.select_all {
                return Ok(true);
            }

            // Flags in this group seem to cause rest of the flags to be ignored
            let mut matched: Option<bool> = None;
            fn update_match<T, U>(
                matched: &mut Option<bool>,
                set_opt: &Option<HashSet<T>>,
                value: U,
            ) where
                T: std::cmp::Eq + std::hash::Hash + std::borrow::Borrow<U>,
                U: std::cmp::Eq + std::hash::Hash,
            {
                if let Some(ref set) = set_opt {
                    *matched.get_or_insert_default() |= set.contains(&value);
                }
            }
            update_match(&mut matched, &self.command_names, process.name().unwrap());
            update_match(&mut matched, &self.pids, process.pid);
            update_match(&mut matched, &self.ppids, process.ppid().unwrap() as usize);
            update_match(&mut matched, &self.sids, process.sid().unwrap() as usize);
            update_match(&mut matched, &self.real_users, process.uid().unwrap());
            update_match(&mut matched, &self.eff_users, process.euid().unwrap());
            update_match(&mut matched, &self.real_groups, process.gid().unwrap());
            update_match(&mut matched, &self.eff_groups, process.egid().unwrap());
            if let Some(m) = matched {
                return Ok(m);
            }

            if self.select_non_session_leaders_with_tty {
                return Ok(!is_session_leader(process) && process.tty() != Teletype::Unknown);
            }

            if self.select_non_session_leaders {
                return Ok(!is_session_leader(process));
            }

            // Default behavior: select processes with same effective user ID and same tty (except -x removes tty restriction)
            Ok(process.euid().unwrap() == current_euid
                && (self.dont_require_tty || process.tty() == current_tty))
        };

        let mut selected = vec![];
        for mut process in walk_process() {
            if matches_criteria(&mut process)? ^ self.negate_selection {
                selected.push(process);
            }
        }

        Ok(selected)
    }
}
