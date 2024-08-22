// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use libc::{getpriority, PRIO_PROCESS};
use nix::errno::Errno;
use std::{
    ffi::OsString,
    sync::{OnceLock, RwLock},
};
use sysinfo::{Pid, System};

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
    let sysinfo = sysinfo().read().unwrap();

    let process = sysinfo.process(Pid::from_u32(pid));

    let usage = match process {
        Some(usage) => usage.cpu_usage(),
        None => 0.0,
    };

    format!("{:.2}", usage)
}

fn pid(pid: u32) -> String {
    pid.to_string()
}

fn user(_pid: u32) -> String {
    "TODO".into()
}

#[cfg(not(target_os = "windows"))]
fn pr(pid: u32) -> String {
    let result = unsafe { getpriority(PRIO_PROCESS, pid) };

    let result = if Errno::last() != Errno::UnknownErrno {
        Errno::clear();
        0
    } else {
        result
    };

    format!("{}", result)
}

#[cfg(target_os = "windows")]
fn pr(_pid: u32) -> String {
    return "0".into();
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

        if trimmed.is_empty() {
            "[kthreadd]".into()
        } else {
            trimmed.into()
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
