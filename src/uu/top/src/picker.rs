// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::{
    ffi::OsString,
    fs::File,
    io::read_to_string,
    path::PathBuf,
    str::FromStr,
    sync::{OnceLock, RwLock},
};
use sysinfo::{Pid, System, Users};
use systemstat::Platform;

static SYSINFO: OnceLock<RwLock<System>> = OnceLock::new();
static SYSTEMSTAT: OnceLock<RwLock<systemstat::System>> = OnceLock::new();

pub fn sysinfo() -> &'static RwLock<System> {
    SYSINFO.get_or_init(|| RwLock::new(System::new_all()))
}

pub fn systemstat() -> &'static RwLock<systemstat::System> {
    SYSTEMSTAT.get_or_init(|| RwLock::new(systemstat::System::new()))
}

pub(crate) fn pickers(fields: &[String]) -> Vec<Box<dyn Fn(u32) -> String>> {
    fields
        .iter()
        .map(|field| match field.as_str() {
            "PID" => helper(pid),
            "USER" => helper(user),
            "PR" => helper(pr),
            "RES" => helper(res),
            "SHR" => helper(shr),
            "S" => helper(s),
            "%CPU" => helper(cpu),
            "TIME+" => helper(time_plus),
            "%MEM" => helper(mem),
            "COMMAND" => helper(command),
            _ => helper(todo),
        })
        .collect()
}

#[inline]
fn helper(f: impl Fn(u32) -> String + 'static) -> Box<dyn Fn(u32) -> String> {
    Box::new(f)
}

fn todo(_pid: u32) -> String {
    "TODO".into()
}

fn cpu(pid: u32) -> String {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return "0.0".into();
    };

    let usage = proc.cpu_usage();

    format!("{:.2}", usage)
}

fn pid(pid: u32) -> String {
    pid.to_string()
}

fn user(pid: u32) -> String {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return "0.0".into();
    };

    let users = Users::new_with_refreshed_list();
    match proc.user_id() {
        Some(uid) => users.get_user_by_id(uid).map(|it| it.name()).unwrap_or("?"),
        None => "?",
    }
    .to_string()
}

#[cfg(not(target_os = "windows"))]
fn pr(pid: u32) -> String {
    use libc::{getpriority, PRIO_PROCESS};
    use nix::errno::Errno;

    let result = unsafe { getpriority(PRIO_PROCESS, pid) };

    let result = if Errno::last() == Errno::UnknownErrno {
        result
    } else {
        Errno::clear();
        0
    };

    format!("{}", result)
}

// TODO: Implement this function for Windows
#[cfg(target_os = "windows")]
fn pr(_pid: u32) -> String {
    "0".into()
}

fn res(_pid: u32) -> String {
    "TODO".into()
}

fn shr(_pid: u32) -> String {
    "TODO".into()
}

fn s(pid: u32) -> String {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return "?".into();
    };

    proc.status()
        .to_string()
        .chars()
        .collect::<Vec<_>>()
        .first()
        .unwrap()
        .to_string()
}

fn time_plus(pid: u32) -> String {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return "0:00.00".into();
    };

    let (hour, min, sec) = {
        let total = proc.run_time();
        let hour = total / 3600;
        let minute = (total % 3600) / 60;
        let second = total % 60;

        (hour, minute, second)
    };

    format!("{}:{:0>2}.{:0>2}", hour, min, sec)
}

fn mem(pid: u32) -> String {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return "0.0".into();
    };

    format!(
        "{:.1}",
        proc.memory() as f32 / sysinfo().read().unwrap().total_memory() as f32
    )
}

fn command(pid: u32) -> String {
    let f = |cmd: &[OsString]| -> String {
        let binding = cmd
            .iter()
            .map(|os_str| os_str.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        let trimmed = binding.trim();

        let result: String = trimmed.into();

        if cfg!(target_os = "linux") && result.is_empty() {
            {
                match PathBuf::from_str(&format!("/proc/{}/status", pid)) {
                    Ok(path) => {
                        let file = File::open(path).unwrap();
                        let content = read_to_string(file).unwrap();
                        let line = content
                            .lines()
                            .collect::<Vec<_>>()
                            .first()
                            .unwrap()
                            .split(':')
                            .collect::<Vec<_>>();

                        line[1].trim().to_owned()
                    }
                    Err(_) => String::new(),
                }
            }
        } else {
            result
        }
    };

    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return "?".into();
    };

    proc.exe()
        .and_then(|it| it.iter().last())
        .map(|it| it.to_str().unwrap())
        .unwrap_or(&f(proc.cmd()))
        .into()
}
