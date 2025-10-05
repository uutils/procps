// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::tui::stat::TuiStat;
use crate::Settings;
use std::any::Any;
use std::cmp::Ordering;
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

pub trait Column {
    fn as_string(&self, show_zeros: bool) -> String;
    fn cmp_dyn(&self, other: &dyn Column) -> Ordering;
    fn as_any(&self) -> &dyn Any;
}

impl Column for String {
    fn as_string(&self, _show_zeros: bool) -> String {
        self.clone()
    }

    fn cmp_dyn(&self, other: &dyn Column) -> Ordering {
        other
            .as_any()
            .downcast_ref::<String>()
            .map(|o| self.cmp(o))
            .unwrap_or(Ordering::Equal)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Column for u32 {
    fn as_string(&self, show_zeros: bool) -> String {
        if !show_zeros && self == &0 {
            return String::new();
        }
        self.to_string()
    }

    fn cmp_dyn(&self, other: &dyn Column) -> Ordering {
        other
            .as_any()
            .downcast_ref::<u32>()
            .map(|o| self.cmp(o))
            .unwrap_or(Ordering::Equal)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Column for Option<i32> {
    fn as_string(&self, show_zeros: bool) -> String {
        if !show_zeros && self == &Some(0) {
            return String::new();
        }
        self.map(|v| v.to_string()).unwrap_or_default()
    }

    fn cmp_dyn(&self, other: &dyn Column) -> Ordering {
        other
            .as_any()
            .downcast_ref::<Option<i32>>()
            .map(|o| match (self, o) {
                (Some(a), Some(b)) => a.cmp(b),
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (None, None) => Ordering::Equal,
            })
            .unwrap_or(Ordering::Equal)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct PercentValue {
    value: f32,
}

impl PercentValue {
    fn new_boxed(value: f32) -> Box<Self> {
        Box::new(Self { value })
    }
}

impl Column for PercentValue {
    fn as_string(&self, show_zeros: bool) -> String {
        if !show_zeros && self.value == 0.0 {
            return String::new();
        }
        format!("{:.1}", self.value)
    }

    fn cmp_dyn(&self, other: &dyn Column) -> Ordering {
        other
            .as_any()
            .downcast_ref::<PercentValue>()
            .map(|o| self.value.partial_cmp(&o.value).unwrap_or(Ordering::Equal))
            .unwrap_or(Ordering::Equal)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct MemValue {
    value: u64,
}

impl MemValue {
    fn new_boxed(value: u64) -> Box<Self> {
        Box::new(Self { value })
    }
}

impl Column for MemValue {
    fn as_string(&self, show_zeros: bool) -> String {
        if !show_zeros && self.value == 0 {
            return String::new();
        }
        let mem_mb = self.value as f64 / bytesize::MIB as f64;
        if mem_mb >= 10000.0 {
            format!("{:.1}g", self.value as f64 / bytesize::GIB as f64)
        } else {
            format!("{mem_mb:.1}m")
        }
    }

    fn cmp_dyn(&self, other: &dyn Column) -> Ordering {
        other
            .as_any()
            .downcast_ref::<MemValue>()
            .map(|o| self.value.cmp(&o.value))
            .unwrap_or(Ordering::Equal)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct TimeMSValue {
    min: u64,
    sec: f64,
}

impl TimeMSValue {
    fn new_boxed(min: u64, sec: f64) -> Box<Self> {
        Box::new(Self { min, sec })
    }
}

impl Column for TimeMSValue {
    fn as_string(&self, show_zeros: bool) -> String {
        if !show_zeros && self.min == 0 && self.sec < 0.01 {
            return String::new();
        }
        format!("{}:{:0>5.2}", self.min, self.sec)
    }

    fn cmp_dyn(&self, other: &dyn Column) -> Ordering {
        other
            .as_any()
            .downcast_ref::<TimeMSValue>()
            .map(|o| match self.min.cmp(&o.min) {
                Ordering::Equal => self.sec.partial_cmp(&o.sec).unwrap_or(Ordering::Equal),
                ord => ord,
            })
            .unwrap_or(Ordering::Equal)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

type Stat<'a> = (&'a Settings, &'a TuiStat);
type Picker = Box<dyn Fn(u32, Stat) -> Box<dyn Column>>;

pub(crate) fn pickers(fields: &[String]) -> Vec<Picker> {
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
fn helper(f: impl Fn(u32, Stat) -> Box<dyn Column> + 'static) -> Picker {
    Box::new(f)
}

fn todo(_pid: u32, _stat: Stat) -> Box<dyn Column> {
    Box::new("TODO".to_string())
}

fn cpu(pid: u32, stat: Stat) -> Box<dyn Column> {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return PercentValue::new_boxed(0.0);
    };

    let cpu_usage = if stat.1.irix_mode {
        proc.cpu_usage()
    } else {
        proc.cpu_usage() / binding.cpus().len() as f32
    };

    PercentValue::new_boxed(cpu_usage)
}

fn pid(pid: u32, _stat: Stat) -> Box<dyn Column> {
    Box::new(pid)
}

fn user(pid: u32, _stat: Stat) -> Box<dyn Column> {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return Box::new("?".to_string());
    };

    let users = Users::new_with_refreshed_list();
    Box::new(
        match proc.user_id() {
            Some(uid) => users.get_user_by_id(uid).map(|it| it.name()).unwrap_or("?"),
            None => "?",
        }
        .to_string(),
    )
}

#[cfg(target_os = "linux")]
fn pr(pid: u32, _stat: Stat) -> Box<dyn Column> {
    use uucore::libc::*;
    let policy = unsafe { sched_getscheduler(pid as i32) };
    if policy == -1 {
        return Box::new(None);
    }

    // normal processes
    if policy == SCHED_OTHER || policy == SCHED_BATCH || policy == SCHED_IDLE {
        return Box::new(Some(get_nice(pid) + 20));
    }

    // real-time processes
    let mut param = sched_param { sched_priority: 0 };
    unsafe { sched_getparam(pid as c_int, &mut param) };
    if param.sched_priority == -1 {
        return Box::new(None);
    }
    Box::new(Some(param.sched_priority))
}

#[cfg(not(target_os = "linux"))]
fn pr(_pid: u32, _stat: Stat) -> Box<dyn Column> {
    Box::new(None)
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
fn ni(pid: u32, _stat: Stat) -> Box<dyn Column> {
    Box::new(Some(get_nice(pid)))
}

// TODO: Implement this function for Windows
#[cfg(target_os = "windows")]
fn ni(_pid: u32, _stat: Stat) -> Box<dyn Column> {
    Box::new(None)
}

#[cfg(target_os = "linux")]
fn virt(pid: u32, _stat: Stat) -> Box<dyn Column> {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return MemValue::new_boxed(0);
    };
    MemValue::new_boxed(proc.virtual_memory())
}

#[cfg(not(target_os = "linux"))]
fn virt(_pid: u32, _stat: Stat) -> Box<dyn Column> {
    MemValue::new_boxed(0)
}

#[cfg(target_os = "linux")]
fn res(pid: u32, _stat: Stat) -> Box<dyn Column> {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return MemValue::new_boxed(0);
    };
    MemValue::new_boxed(proc.memory())
}

#[cfg(not(target_os = "linux"))]
fn res(_pid: u32, _stat: Stat) -> Box<dyn Column> {
    MemValue::new_boxed(0)
}

#[cfg(target_os = "linux")]
fn shr(pid: u32, _stat: Stat) -> Box<dyn Column> {
    let file_path = format!("/proc/{pid}/statm");
    let Ok(file) = File::open(file_path) else {
        return MemValue::new_boxed(0);
    };
    let content = read_to_string(file).unwrap();
    let values = content.split_whitespace();
    if let Some(shared) = values.collect::<Vec<_>>().get(2) {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        MemValue::new_boxed(shared.parse::<u64>().unwrap() * page_size as u64)
    } else {
        MemValue::new_boxed(0)
    }
}

#[cfg(not(target_os = "linux"))]
fn shr(_pid: u32, _stat: Stat) -> Box<dyn Column> {
    MemValue::new_boxed(0)
}

fn s(pid: u32, _stat: Stat) -> Box<dyn Column> {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return Box::new("?".to_string());
    };

    Box::new(
        proc.status()
            .to_string()
            .chars()
            .collect::<Vec<_>>()
            .first()
            .unwrap()
            .to_string(),
    )
}

fn time_plus(pid: u32, _stat: Stat) -> Box<dyn Column> {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return TimeMSValue::new_boxed(0, 0.0);
    };

    let (min, sec) = {
        let total = proc.accumulated_cpu_time();
        let minute = total / (60 * 1000);
        let second = (total % (60 * 1000)) as f64 / 1000.0;

        (minute, second)
    };

    TimeMSValue::new_boxed(min, sec)
}

fn mem(pid: u32, _stat: Stat) -> Box<dyn Column> {
    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return PercentValue::new_boxed(0.0);
    };

    PercentValue::new_boxed(
        proc.memory() as f32 / sysinfo().read().unwrap().total_memory() as f32 * 100.0,
    )
}

fn command(pid: u32, stat: Stat) -> Box<dyn Column> {
    let full_command_line = stat.1.full_command_line;
    let f = |cmd: &[OsString]| -> String {
        let binding = cmd
            .iter()
            .map(|os_str| os_str.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        let trimmed = binding.trim();

        let result: String = trimmed.into();

        if cfg!(target_os = "linux") && result.is_empty() {
            // actually executable name
            let path = PathBuf::from_str(&format!("/proc/{pid}/status")).unwrap();
            if let Ok(file) = File::open(path) {
                let content = read_to_string(file).unwrap();
                let line = content
                    .lines()
                    .collect::<Vec<_>>()
                    .first()
                    .unwrap()
                    .split(':')
                    .collect::<Vec<_>>();

                line[1].trim().to_owned()
            } else {
                String::new()
            }
        } else {
            result
        }
    };

    let binding = sysinfo().read().unwrap();
    let Some(proc) = binding.process(Pid::from_u32(pid)) else {
        return Box::new("?".to_string());
    };

    Box::new(
        proc.exe()
            .and_then(|it| {
                if full_command_line {
                    it.iter().next_back()
                } else {
                    it.file_name()
                }
            })
            .map(|it| it.to_str().unwrap().to_string())
            .unwrap_or(if full_command_line {
                f(proc.cmd())
            } else {
                proc.name().to_str().unwrap().to_string()
            }),
    )
}
