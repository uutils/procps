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

static SYSINFO: OnceLock<RwLock<System>> = OnceLock::new();

pub fn sysinfo() -> &'static RwLock<System> {
    SYSINFO.get_or_init(|| RwLock::new(System::new_all()))
}

pub(crate) fn pickers(fields: &[String]) -> Vec<Box<dyn Fn(u32) -> String>> {
    fields
        .iter()
        .map(|field| match field.as_str() {
            "PID" => helper(pid),
            "USER" => helper(user),
            "PR" => helper(pr),
            "NI" => helper(ni),
            "VIRT" => helper(virt),
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

#[cfg(target_os = "linux")]
fn format_memory(memory_b: u64) -> String {
    let mem_mb = memory_b as f64 / bytesize::MIB as f64;
    if mem_mb >= 10000.0 {
        format!("{:.1}g", memory_b as f64 / bytesize::GIB as f64)
    } else {
        format!("{mem_mb:.1}m")
    }
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

    format!("{usage:.2}")
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

#[cfg(target_os = "linux")]
fn pr(pid: u32) -> String {
    use uucore::libc::*;
    let policy = unsafe { sched_getscheduler(pid as i32) };
    if policy == -1 {
        return String::new();
    }

    // normal processes
    if policy == SCHED_OTHER || policy == SCHED_BATCH || policy == SCHED_IDLE {
        return (get_nice(pid) + 20).to_string();
    }

    // real-time processes
    let mut param = sched_param { sched_priority: 0 };
    unsafe { sched_getparam(pid as c_int, &mut param) };
    if param.sched_priority == -1 {
        return String::new();
    }
    param.sched_priority.to_string()
}

#[cfg(not(target_os = "linux"))]
fn pr(pid: u32) -> String {
    todo(pid)
}

#[cfg(not(target_os = "windows"))]
fn get_nice(pid: u32) -> i32 {
    use libc::{getpriority, PRIO_PROCESS};
    use nix::errno::Errno;

    // this is nice value, not priority value
    let result = unsafe { getpriority(PRIO_PROCESS, pid) };

    let result = if Errno::last() == Errno::UnknownErrno {
        result
    } else {
        Errno::clear();
        0
    };

    result as i32
}

#[cfg(not(target_os = "windows"))]
fn ni(pid: u32) -> String {
    format!("{}", get_nice(pid))
}

// TODO: Implement this function for Windows
#[cfg(target_os = "windows")]
fn ni(_pid: u32) -> String {
    "0".into()
}

#[cfg(target_os = "linux")]
fn virt(pid: u32) -> String {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return "0.0".into();
    };
    format_memory(proc.virtual_memory())
}

#[cfg(not(target_os = "linux"))]
fn virt(_pid: u32) -> String {
    "TODO".into()
}

#[cfg(target_os = "linux")]
fn res(pid: u32) -> String {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return "0.0".into();
    };
    format_memory(proc.memory())
}

#[cfg(not(target_os = "linux"))]
fn res(_pid: u32) -> String {
    "TODO".into()
}

#[cfg(target_os = "linux")]
fn shr(pid: u32) -> String {
    let file_path = format!("/proc/{pid}/statm");
    let Ok(file) = File::open(file_path) else {
        return "0.0".into();
    };
    let content = read_to_string(file).unwrap();
    let values = content.split_whitespace();
    if let Some(shared) = values.collect::<Vec<_>>().get(2) {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        format_memory(shared.parse::<u64>().unwrap() * page_size as u64)
    } else {
        "0.0".into()
    }
}

#[cfg(not(target_os = "linux"))]
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

    format!("{hour}:{min:0>2}.{sec:0>2}")
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
                match PathBuf::from_str(&format!("/proc/{pid}/status")) {
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
        .and_then(|it| it.iter().next_back())
        .map(|it| it.to_str().unwrap())
        .unwrap_or(&f(proc.cmd()))
        .into()
}
