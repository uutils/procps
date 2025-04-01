// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::collections::HashSet;
use uu_pgrep::process::{walk_process, ProcessInformation};
#[cfg(target_os = "linux")]
pub fn get_all_processes() -> Vec<ProcessInformation> {
    walk_process().collect()
}

#[cfg(target_os = "linux")]
pub fn filter_processes_by_pid(pids: &[i32]) -> Vec<ProcessInformation> {
    let mut matching_pids = Vec::new();

    for process in get_all_processes() {
        if pids.iter().any(|pid| *pid == process.pid as i32) {
            matching_pids.push(process);
        }
    }

    matching_pids
}

#[cfg(target_os = "linux")]
pub fn filter_processes_by_user(users: &[String]) -> Vec<ProcessInformation> {
    let mut matching_pids = Vec::new();

    for mut process in get_all_processes() {
        if let Some(owner) = get_process_owner(&mut process) {
            if users.iter().any(|u| *u == owner) {
                matching_pids.push(process);
            }
        }
    }

    matching_pids
}

#[cfg(target_os = "linux")]
pub fn filter_processes_by_command(commands: &[String]) -> Vec<ProcessInformation> {
    let mut matching_processes = Vec::new();

    for process in get_all_processes() {
        let cmdline = process.cmdline.split(" ").collect::<Vec<_>>()[0];
        let cmd_name = cmdline.split("/").last().unwrap_or(cmdline);
        if commands.iter().any(|c| c == cmd_name) {
            matching_processes.push(process);
        }
    }

    matching_processes
}

#[cfg(target_os = "linux")]
pub fn filter_processes_by_terminal(terminals: &[String]) -> Vec<ProcessInformation> {
    let mut matching_processes = Vec::new();

    for process in get_all_processes() {
        if let Some(tty) = get_process_terminal(&process) {
            if terminals.iter().any(|t| tty.contains(t)) {
                matching_processes.push(process);
            }
        }
    }

    matching_processes
}

#[cfg(target_os = "linux")]
pub fn get_process_owner(process: &mut ProcessInformation) -> Option<String> {
    // let status_path = format!("/proc/{}/status", pid);
    let uid = process.uid().ok()?;

    // Read /etc/passwd to look up the username for this UID
    std::fs::read_to_string("/etc/passwd")
        .ok()?
        .lines()
        .find_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(entry_uid) = parts[2].parse::<u32>() {
                    if entry_uid == uid {
                        return Some(parts[0].to_string());
                    }
                }
            }
            None
        })
}

#[cfg(target_os = "linux")]
pub fn get_process_terminal(process: &ProcessInformation) -> Option<String> {
    use uu_pgrep::process::Teletype;
    match process.tty() {
        Teletype::Tty(id) => Some(format!("tty{}", id)),
        Teletype::TtyS(id) => Some(format!("ttyS{}", id)),
        Teletype::Pts(id) => Some(format!("pts/{}", id)),
        Teletype::Unknown => None,
    }
}

#[cfg(target_os = "linux")]
pub fn get_active_users(processes: &mut [ProcessInformation]) -> HashSet<String> {
    let mut users = HashSet::new();

    for process in processes {
        if let Some(user) = get_process_owner(process) {
            users.insert(user);
        }
    }

    users
}

#[cfg(target_os = "linux")]
pub fn get_active_terminals(processes: &[ProcessInformation]) -> HashSet<String> {
    let mut terminals = HashSet::new();

    for process in processes {
        if let Some(tty) = get_process_terminal(process) {
            terminals.insert(tty);
        }
    }

    terminals
}

#[cfg(target_os = "linux")]
pub fn get_active_commands(processes: &[ProcessInformation]) -> HashSet<String> {
    let mut commands = HashSet::new();

    for process in processes {
        commands.insert(process.cmdline.to_string());
    }

    commands
}
