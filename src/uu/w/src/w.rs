// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use chrono::Datelike;
use clap::crate_version;
use clap::{Arg, ArgAction, Command};
#[cfg(target_os = "linux")]
use libc::{sysconf, _SC_CLK_TCK};
use std::process;
#[cfg(target_os = "linux")]
use std::{collections::HashMap, fs, path::Path};
#[cfg(target_os = "linux")]
use uucore::utmpx::Utmpx;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("w.md");
const USAGE: &str = help_usage!("w.md");

struct UserInfo {
    user: String,
    terminal: String,
    login_time: String,
    idle_time: String,
    jcpu: String,
    pcpu: String,
    command: String,
}

#[cfg(target_os = "linux")]
fn fetch_terminal_jcpu() -> Result<HashMap<u64, f64>, std::io::Error> {
    // Iterate over all pid folders in /proc and build a HashMap with their terminals and cpu usage.
    let pid_dirs = fs::read_dir("/proc")?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|s| s.to_os_string().into_string().ok())
        })
        // Check to see if directory is an integer (pid)
        .filter_map(|pid_dir_str| pid_dir_str.parse::<i32>().ok());
    let mut pid_hashmap = HashMap::new();
    for pid in pid_dirs {
        // Fetch terminal number for current pid
        let terminal_number = fetch_terminal_number(pid)?;
        // Get current total CPU time for current pid
        let pcpu_time = fetch_pcpu_time(pid)?;
        // Update HashMap with found terminal number and add pcpu time for current pid
        *pid_hashmap.entry(terminal_number).or_insert(0.0) += pcpu_time;
    }
    Ok(pid_hashmap)
}

#[cfg(target_os = "linux")]
fn fetch_terminal_number(pid: i32) -> Result<u64, std::io::Error> {
    let stat_path = Path::new("/proc").join(pid.to_string()).join("stat");
    // Separate stat and get terminal number, which is at position 6
    let f = fs::read_to_string(stat_path)?;
    let stat: Vec<&str> = f.split_whitespace().collect();
    Ok(stat[6].parse().unwrap_or_default())
}

#[cfg(target_os = "linux")]
fn get_clock_tick() -> i64 {
    unsafe { sysconf(_SC_CLK_TCK) }
}

#[cfg(target_os = "linux")]
fn fetch_pcpu_time(pid: i32) -> Result<f64, std::io::Error> {
    let stat_path = Path::new("/proc").join(pid.to_string()).join("stat");
    // Seperate stat file by whitespace and get utime and stime, which are at
    // positions 13 and 14 (0-based), respectively.
    let f = fs::read_to_string(stat_path)?;
    let stat: Vec<&str> = f.split_whitespace().collect();
    // Parse utime and stime to f64
    let utime: f64 = stat[13].parse().unwrap_or_default();
    let stime: f64 = stat[14].parse().unwrap_or_default();
    // Divide by clock tick to get actual time
    Ok((utime + stime) / get_clock_tick() as f64)
}

#[cfg(target_os = "linux")]
fn format_time(time: String) -> Result<String, chrono::format::ParseError> {
    let mut t: String = time;
    // Trim the seconds off of timezone offset, as chrono can't parse the time with it present
    if let Some(time_offset) = t.rfind(':') {
        t = t.drain(..time_offset).collect();
    }
    // If login time day is not current day, format like Sat16, or Fri06
    let current_dt = chrono::Local::now().fixed_offset();
    let dt = chrono::DateTime::parse_from_str(&t, "%Y-%m-%d %H:%M:%S%.f %:z")?;

    if current_dt.day() != dt.day() {
        Ok(dt.format("%a%d").to_string())
    } else {
        Ok(dt.format("%H:%M").to_string())
    }
}

#[cfg(target_os = "linux")]
fn fetch_cmdline(pid: i32) -> Result<String, std::io::Error> {
    let cmdline_path = Path::new("/proc").join(pid.to_string()).join("cmdline");
    fs::read_to_string(cmdline_path)
}

#[cfg(target_os = "linux")]
fn fetch_user_info() -> Result<Vec<UserInfo>, std::io::Error> {
    let terminal_jcpu_hm = fetch_terminal_jcpu()?;

    let mut user_info_list = Vec::new();
    for entry in Utmpx::iter_all_records() {
        if entry.is_user_process() {
            let mut jcpu: f64 = 0.0;

            if let Ok(terminal_number) = fetch_terminal_number(entry.pid()) {
                jcpu = terminal_jcpu_hm
                    .get(&terminal_number)
                    .cloned()
                    .unwrap_or_default();
            }

            let user_info = UserInfo {
                user: entry.user(),
                terminal: entry.tty_device(),
                login_time: format_time(entry.login_time().to_string()).unwrap_or_default(),
                idle_time: String::new(), // Placeholder, needs actual implementation
                jcpu: format!("{:.2}", jcpu),
                pcpu: fetch_pcpu_time(entry.pid()).unwrap_or_default().to_string(),
                command: fetch_cmdline(entry.pid()).unwrap_or_default(),
            };
            user_info_list.push(user_info);
        }
    }

    Ok(user_info_list)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn fetch_user_info() -> Result<Vec<UserInfo>, std::io::Error> {
    Ok(Vec::new())
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let no_header = matches.get_flag("no-header");

    match fetch_user_info() {
        Ok(user_info) => {
            if !no_header {
                println!("USER\tTTY\tLOGIN@\tIDLE\tJCPU\tPCPU\tWHAT");
            }
            for user in user_info {
                println!(
                    "{}\t{}\t{}\t{}\t{}s\t{}s\t{}",
                    user.user,
                    user.terminal,
                    user.login_time,
                    user.idle_time,
                    user.jcpu,
                    user.pcpu,
                    user.command
                );
            }
        }
        Err(e) => {
            eprintln!("w: failed to fetch user info: {}", e);
            process::exit(1);
        }
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .disable_help_flag(true)
        .arg(
            Arg::new("no-header")
                .short('h')
                .long("no-header")
                .help("do not print header")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-current")
                .short('u')
                .long("no-current")
                .help("ignore current process username")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("short")
                .short('s')
                .long("short")
                .help("short format")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("from")
                .short('f')
                .long("from")
                .help("show remote hostname field")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("old-style")
                .short('o')
                .long("old-style")
                .help("old style output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ip-addr")
                .short('i')
                .long("ip-addr")
                .help("display IP address instead of hostname (if possible)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("pids")
                .short('p')
                .long("pids")
                .help("show the PID(s) of processes in WHAT")
                .action(ArgAction::SetTrue),
        )
}

#[cfg(test)]
mod tests {
    use crate::{
        fetch_cmdline, fetch_pcpu_time, fetch_terminal_number, format_time, get_clock_tick,
    };
    use chrono;
    use std::{fs, path::Path, process};

    #[test]
    #[cfg(target_os = "linux")]
    fn test_format_time() {
        let unix_epoc = chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S%.6f %::z")
            .to_string();
        let unix_formatted = format_time(unix_epoc).unwrap();
        assert!(unix_formatted.contains(':') && unix_formatted.chars().count() == 5);
        // Test a date that is 5 days ago
        let td = chrono::Local::now().fixed_offset()
            - chrono::TimeDelta::new(60 * 60 * 24 * 5, 0).unwrap();
        // Pre-format time, so it's similar to how utmpx returns it
        let pre_formatted = format!("{}", td.format("%Y-%m-%d %H:%M:%S%.6f %::z"));

        assert_eq!(
            format_time(pre_formatted).unwrap(),
            td.format("%a%d").to_string()
        )
    }

    #[test]
    #[cfg(target_os = "linux")]
    // Get PID of current process and use that for cmdline testing
    fn test_fetch_cmdline() {
        // uucore's utmpx returns an i32, so we cast to that to mimic it.
        let pid = process::id() as i32;
        let path = Path::new("/proc").join(pid.to_string()).join("cmdline");
        assert_eq!(
            fs::read_to_string(path).unwrap(),
            fetch_cmdline(pid).unwrap()
        )
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_fetch_terminal_number() {
        let pid = process::id() as i32;
        let path = Path::new("/proc").join(pid.to_string()).join("stat");
        let f = fs::read_to_string(path).unwrap();
        let stat: Vec<&str> = f.split_whitespace().collect();
        let term_num = stat[6];
        assert_eq!(fetch_terminal_number(pid).unwrap().to_string(), term_num)
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_fetch_pcpu_time() {
        let pid = process::id() as i32;
        let path = Path::new("/proc").join(pid.to_string()).join("stat");
        let f = fs::read_to_string(path).unwrap();
        let stat: Vec<&str> = f.split_whitespace().collect();
        let utime: f64 = stat[13].parse().unwrap();
        let stime: f64 = stat[14].parse().unwrap();
        assert_eq!(
            fetch_pcpu_time(pid).unwrap(),
            (utime + stime) / get_clock_tick() as f64
        )
    }
}