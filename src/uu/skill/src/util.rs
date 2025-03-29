use std::collections::HashSet;

#[cfg(target_os = "linux")]
pub fn get_all_process_ids() -> Vec<i32> {
    let mut pids = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(pid) = entry.file_name().to_string_lossy().parse::<i32>() {
                pids.push(pid);
            }
        }
    }

    pids
}

#[cfg(target_os = "linux")]
pub fn filter_processes_by_user(users: &[String]) -> Vec<i32> {
    let mut matching_pids = Vec::new();

    for pid in get_all_process_ids() {
        if let Some(owner) = get_process_owner(pid) {
            if users.iter().any(|u| *u == owner) {
                matching_pids.push(pid);
            }
        }
    }

    matching_pids
}

#[cfg(target_os = "linux")]
pub fn filter_processes_by_command(commands: &[String]) -> Vec<i32> {
    let mut matching_pids = Vec::new();

    for pid in get_all_process_ids() {
        if let Some(cmd_name) = get_process_command_name(pid) {
            if commands.iter().any(|c| cmd_name.contains(c)) {
                matching_pids.push(pid);
            }
        }
    }

    matching_pids
}

#[cfg(target_os = "linux")]
pub fn filter_processes_by_terminal(terminals: &[String]) -> Vec<i32> {
    let mut matching_pids = Vec::new();

    for pid in get_all_process_ids() {
        if let Some(tty) = get_process_terminal(pid) {
            if terminals.iter().any(|t| tty.contains(t)) {
                matching_pids.push(pid);
            }
        }
    }

    matching_pids
}

#[cfg(target_os = "linux")]
pub fn get_process_owner(pid: i32) -> Option<String> {
    let status_path = format!("/proc/{}/status", pid);

    if let Ok(status_content) = std::fs::read_to_string(&status_path) {
        for line in status_content.lines() {
            if line.starts_with("Uid:") {
                let uid = line.split_whitespace().nth(1)?;

                if let Ok(output) = std::process::Command::new("id")
                    .args(["-n", "-u", uid])
                    .output()
                {
                    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return Some(username);
                }
                break;
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
pub fn get_process_command_name(pid: i32) -> Option<String> {
    let cmdline_path = format!("/proc/{}/cmdline", pid);

    if let Ok(cmdline_content) = std::fs::read_to_string(&cmdline_path) {
        let cmd_parts: Vec<&str> = cmdline_content.split('\0').collect();

        if !cmd_parts.is_empty() {
            let cmd_with_path = cmd_parts[0];
            let cmd_name = cmd_with_path.split('/').last().unwrap_or(cmd_with_path);
            return Some(cmd_name.to_string());
        }
    }

    None
}

#[cfg(target_os = "linux")]
pub fn get_process_terminal(pid: i32) -> Option<String> {
    let stat_path = format!("/proc/{}/stat", pid);

    if let Ok(stat_content) = std::fs::read_to_string(&stat_path) {
        let fields: Vec<&str> = stat_content.split_whitespace().collect();

        if fields.len() >= 7 {
            let tty_nr = fields[6];

            if tty_nr != "0" {
                if let Ok(tty_num) = tty_nr.parse::<u32>() {
                    let major = tty_num >> 8;
                    let minor = tty_num & 0xFF;

                    return Some(if major == 136 {
                        format!("pts/{}", minor)
                    } else if major == 4 {
                        format!("tty{}", minor)
                    } else {
                        format!("unknown{}", tty_num)
                    });
                }
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
pub fn get_active_users() -> HashSet<String> {
    let mut users = HashSet::new();

    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(pid) = entry.file_name().to_string_lossy().parse::<i32>() {
                if let Some(user) = get_process_owner(pid) {
                    users.insert(user);
                }
            }
        }
    }

    users
}

#[cfg(target_os = "linux")]
pub fn get_active_terminals() -> HashSet<String> {
    let mut terminals = HashSet::new();

    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(pid) = entry.file_name().to_string_lossy().parse::<i32>() {
                if let Some(tty) = get_process_terminal(pid) {
                    if !tty.contains("unknown") {
                        terminals.insert(tty);
                    }
                }
            }
        }
    }

    terminals
}

#[cfg(target_os = "linux")]
pub fn get_active_commands() -> HashSet<String> {
    let mut commands = HashSet::new();

    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(pid) = entry.file_name().to_string_lossy().parse::<i32>() {
                if let Some(cmd) = get_process_command_name(pid) {
                    commands.insert(cmd);
                }
            }
        }
    }

    commands
}
