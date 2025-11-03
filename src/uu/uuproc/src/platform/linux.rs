// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::{CgroupMembership, Namespace, RunState, Teletype};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::LazyLock;
use walkdir::{DirEntry, WalkDir};

/// Process ID and its information (Linux)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessInformation {
    pub pid: usize,
    pub cmdline: String,

    inner_status: String,
    inner_stat: String,

    /// Processed `/proc/self/status` file
    cached_status: Option<Rc<HashMap<String, String>>>,
    /// Processed `/proc/self/stat` file
    cached_stat: Option<Rc<Vec<String>>>,

    cached_start_time: Option<u64>,

    cached_thread_ids: Option<Rc<Vec<usize>>>,
}

impl ProcessInformation {
    /// Try new with pid path such as `/proc/self`
    ///
    /// # Error
    ///
    /// If the files in path cannot be parsed into [ProcessInformation],
    /// it almost caused by wrong filesystem structure.
    ///
    /// - [The /proc Filesystem](https://docs.kernel.org/filesystems/proc.html#process-specific-subdirectories)
    pub fn try_new(value: PathBuf) -> Result<Self, io::Error> {
        let dir_append = |mut path: PathBuf, str: String| {
            path.push(str);
            path
        };

        let value = if value.is_symlink() {
            fs::read_link(value)?
        } else {
            value
        };

        let pid = {
            value
                .iter()
                .next_back()
                .ok_or(io::ErrorKind::Other)?
                .to_str()
                .ok_or(io::ErrorKind::InvalidData)?
                .parse::<usize>()
                .map_err(|_| io::ErrorKind::InvalidData)?
        };

        let cmdline = fs::read_to_string(dir_append(value.clone(), "cmdline".into()))?
            .replace('\0', " ")
            .trim_end()
            .into();

        Ok(Self {
            pid,
            cmdline,
            inner_status: fs::read_to_string(dir_append(value.clone(), "status".into()))?,
            inner_stat: fs::read_to_string(dir_append(value, "stat".into()))?,
            ..Default::default()
        })
    }

    pub fn current_process_info() -> Result<LinuxProcessInfo, io::Error> {
        use std::str::FromStr;

        let pid = uucore::process::getpid();
        LinuxProcessInfo::try_new(PathBuf::from_str(&format!("/proc/{pid}")).unwrap())
    }

    fn status(&mut self) -> Rc<HashMap<String, String>> {
        if let Some(c) = &self.cached_status {
            return Rc::clone(c);
        }

        let result = self
            .inner_status
            .lines()
            .filter_map(|it| it.split_once(':'))
            .map(|it| (it.0.to_string(), it.1.trim_start().to_string()))
            .collect::<HashMap<_, _>>();

        let result = Rc::new(result);
        self.cached_status = Some(Rc::clone(&result));
        Rc::clone(&result)
    }

    fn stat(&mut self) -> Rc<Vec<String>> {
        if let Some(c) = &self.cached_stat {
            return Rc::clone(c);
        }

        let result: Vec<_> = stat_split(&self.inner_stat);
        let result = Rc::new(result);
        self.cached_stat = Some(Rc::clone(&result));
        Rc::clone(&result)
    }

    fn get_numeric_stat_field(&mut self, index: usize) -> Result<u64, io::Error> {
        self.stat()
            .get(index)
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<u64>()
            .map_err(|_| io::ErrorKind::InvalidData.into())
    }

    fn get_uid_or_gid_field(&mut self, field: &str, index: usize) -> Result<u32, io::Error> {
        self.status()
            .get(field)
            .ok_or(io::ErrorKind::InvalidData)?
            .split_whitespace()
            .nth(index)
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<u32>()
            .map_err(|_| io::ErrorKind::InvalidData.into())
    }

    fn get_hex_status_field(&mut self, field_name: &str) -> Result<u64, io::Error> {
        self.status()
            .get(field_name)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{field_name} field not found"),
                )
            })
            .and_then(|value| {
                u64::from_str_radix(value.trim(), 16).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid {field_name} value"),
                    )
                })
            })
    }

    fn tty_nr(&mut self) -> Result<u64, io::Error> {
        self.get_numeric_stat_field(6)
    }
}

impl ProcessInformation {
    pub fn proc_status(&self) -> &str {
        &self.inner_status
    }

    pub fn proc_stat(&self) -> &str {
        &self.inner_stat
    }

    /// Collect information from `/proc/<pid>/status` file
    pub fn status(&mut self) -> Rc<HashMap<String, String>> {
        if let Some(c) = &self.cached_status {
            return Rc::clone(c);
        }

        let result = self
            .inner_status
            .lines()
            .filter_map(|it| it.split_once(':'))
            .map(|it| (it.0.to_string(), it.1.trim_start().to_string()))
            .collect::<HashMap<_, _>>();

        let result = Rc::new(result);
        self.cached_status = Some(Rc::clone(&result));
        Rc::clone(&result)
    }

    fn stat(&mut self) -> Rc<Vec<String>> {
        if let Some(c) = &self.cached_stat {
            return Rc::clone(c);
        }

        let result: Vec<_> = stat_split(&self.inner_stat);
        let result = Rc::new(result);
        self.cached_stat = Some(Rc::clone(&result));
        Rc::clone(&result)
    }

    fn get_numeric_stat_field(&mut self, index: usize) -> Result<u64, io::Error> {
        self.stat()
            .get(index)
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<u64>()
            .map_err(|_| io::ErrorKind::InvalidData.into())
    }

    fn get_uid_or_gid_field(&mut self, field: &str, index: usize) -> Result<u32, io::Error> {
        self.status()
            .get(field)
            .ok_or(io::ErrorKind::InvalidData)?
            .split_whitespace()
            .nth(index)
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<u32>()
            .map_err(|_| io::ErrorKind::InvalidData.into())
    }

    fn get_hex_status_field(&mut self, field_name: &str) -> Result<u64, io::Error> {
        self.status()
            .get(field_name)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{field_name} field not found"),
                )
            })
            .and_then(|value| {
                u64::from_str_radix(value.trim(), 16).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid {field_name} value"),
                    )
                })
            })
    }

    fn tty_nr(&mut self) -> Result<u64, io::Error> {
        self.get_numeric_stat_field(6)
    }

    pub fn name(&mut self) -> Result<String, io::Error> {
        self.status()
            .get("Name")
            .cloned()
            .ok_or(io::ErrorKind::InvalidData.into())
    }

    pub fn ppid(&mut self) -> Result<u64, io::Error> {
        self.get_numeric_stat_field(3)
    }

    pub fn pgid(&mut self) -> Result<u64, io::Error> {
        self.get_numeric_stat_field(4)
    }

    pub fn sid(&mut self) -> Result<u64, io::Error> {
        self.get_numeric_stat_field(5)
    }

    pub fn uid(&mut self) -> Result<u32, io::Error> {
        self.get_uid_or_gid_field("Uid", 0)
    }

    pub fn euid(&mut self) -> Result<u32, io::Error> {
        self.get_uid_or_gid_field("Uid", 1)
    }

    pub fn gid(&mut self) -> Result<u32, io::Error> {
        self.get_uid_or_gid_field("Gid", 0)
    }

    pub fn egid(&mut self) -> Result<u32, io::Error> {
        self.get_uid_or_gid_field("Gid", 1)
    }

    pub fn suid(&mut self) -> Result<u32, io::Error> {
        self.get_uid_or_gid_field("Uid", 2)
    }

    pub fn sgid(&mut self) -> Result<u32, io::Error> {
        self.get_uid_or_gid_field("Gid", 2)
    }

    pub fn tty(&mut self) -> Teletype {
        if let Ok(tty_nr) = self.tty_nr() {
            if let Ok(tty) = Teletype::try_from(tty_nr) {
                return tty;
            }
        }

        let path = PathBuf::from(format!("/proc/{}/fd", self.pid));
        let Ok(result) = fs::read_dir(path) else {
            return Teletype::Unknown;
        };

        for dir in result.flatten().filter(|it| it.path().is_symlink()) {
            if let Ok(path) = fs::read_link(dir.path()) {
                if let Ok(tty) = Teletype::try_from(path) {
                    return tty;
                }
            }
        }

        Teletype::Unknown
    }

    pub fn run_state(&mut self) -> Result<RunState, io::Error> {
        RunState::try_from(self.stat().get(2).unwrap().as_str())
    }

    pub fn start_time(&mut self) -> Result<u64, io::Error> {
        if let Some(time) = self.cached_start_time {
            return Ok(time);
        }
        let time = self.get_numeric_stat_field(21)?;
        self.cached_start_time = Some(time);
        Ok(time)
    }

    pub fn env_vars(&self) -> Result<HashMap<String, String>, io::Error> {
        let content = fs::read_to_string(format!("/proc/{}/environ", self.pid))?;
        let mut env_vars = HashMap::new();
        for entry in content.split('\0') {
            if let Some((key, value)) = entry.split_once('=') {
                env_vars.insert(key.to_string(), value.to_string());
            }
        }
        Ok(env_vars)
    }

    pub fn namespaces(&self) -> Result<Namespace, io::Error> {
        Namespace::from_pid(self.pid)
    }

    pub fn cgroups(&mut self) -> Result<Vec<CgroupMembership>, io::Error> {
        fs::read_to_string(format!("/proc/{}/cgroup", self.pid))?
            .lines()
            .map(CgroupMembership::try_from)
            .collect()
    }

    pub fn root(&mut self) -> Result<PathBuf, io::Error> {
        fs::read_link(format!("/proc/{}/root", self.pid))
    }

    pub fn thread_ids(&mut self) -> Result<Vec<usize>, io::Error> {
        if let Some(c) = &self.cached_thread_ids {
            return Ok(c.as_ref().clone());
        }

        let tids_dir = format!("/proc/{}/task", self.pid);
        let result = WalkDir::new(tids_dir)
            .min_depth(1)
            .max_depth(1)
            .follow_links(false)
            .into_iter()
            .flatten()
            .flat_map(|it| {
                it.path()
                    .file_name()
                    .map(|it| it.to_str().unwrap().parse::<usize>().unwrap())
            })
            .collect::<Vec<_>>();

        let result_rc = Rc::new(result.clone());
        self.cached_thread_ids = Some(result_rc);
        Ok(result)
    }

    pub fn signals_pending_mask(&mut self) -> Result<u64, io::Error> {
        self.get_hex_status_field("SigPnd")
    }

    pub fn signals_blocked_mask(&mut self) -> Result<u64, io::Error> {
        self.get_hex_status_field("SigBlk")
    }

    pub fn signals_ignored_mask(&mut self) -> Result<u64, io::Error> {
        self.get_hex_status_field("SigIgn")
    }

    pub fn signals_caught_mask(&mut self) -> Result<u64, io::Error> {
        self.get_hex_status_field("SigCgt")
    }
}

impl TryFrom<DirEntry> for ProcessInformation {
    type Error = io::Error;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        Self::try_new(value.into_path())
    }
}

fn stat_split(stat: &str) -> Vec<String> {
    let stat = String::from(stat);
    if let (Some(left), Some(right)) = (stat.find('('), stat.rfind(')')) {
        let mut split_stat = vec![];
        split_stat.push(stat[..left - 1].to_string());
        split_stat.push(stat[left + 1..right].to_string());
        split_stat.extend(stat[right + 2..].split_whitespace().map(String::from));
        split_stat
    } else {
        stat.split_whitespace().map(String::from).collect()
    }
}

static THREAD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^/proc/[0-9]+$|^/proc/[0-9]+/task$|^/proc/[0-9]+/task/[0-9]+$").unwrap()
});

/// Iterating pid in current system
pub fn walk_process() -> impl Iterator<Item = ProcessInformation> {
    WalkDir::new("/proc/")
        .max_depth(1)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter(|it| it.path().is_dir())
        .flat_map(ProcessInformation::try_from)
}

pub fn walk_threads() -> impl Iterator<Item = ProcessInformation> {
    WalkDir::new("/proc/")
        .min_depth(1)
        .max_depth(3)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| THREAD_REGEX.is_match(e.path().as_os_str().to_string_lossy().as_ref()))
        .flatten()
        .filter(|it| it.path().as_os_str().to_string_lossy().contains("/task/"))
        .flat_map(ProcessInformation::try_from)
}
