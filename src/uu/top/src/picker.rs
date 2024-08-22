// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::sync::{OnceLock, RwLock};
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

fn pr(_pid: u32) -> String {
    "TODO".into()
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

    proc.status().to_string()
}

fn time_plus(_pid: u32) -> String {
    "TODO".into()
}

fn mem(_pid: u32) -> String {
    "TODO".into()
}

fn command(pid: u32) -> String {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return "?".into();
    };

    proc.cmd()
        .iter()
        .flat_map(|it| it.to_str())
        .collect::<Vec<_>>()
        .join(" ")
}
