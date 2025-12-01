// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use regex::Regex;
use std::fs::read_link;
use std::hash::Hash;
#[cfg(target_os = "linux")]
use std::ops::RangeInclusive;
use std::sync::{LazyLock, OnceLock};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs, io,
    path::PathBuf,
};
use walkdir::{DirEntry, WalkDir};

/// Represents a TTY driver entry from /proc/tty/drivers
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct TtyDriverEntry {
    device_prefix: String,
    major: u32,
    minor_range: RangeInclusive<u32>,
}

#[cfg(target_os = "linux")]
impl TtyDriverEntry {
    fn new(device_prefix: String, major: u32, minor_range: RangeInclusive<u32>) -> Self {
        Self {
            device_prefix,
            major,
            minor_range,
        }
    }

    fn device_path_if_matches(&self, major: u32, minor: u32) -> Option<String> {
        if self.major != major || !self.minor_range.contains(&minor) {
            return None;
        }

        // /dev/pts devices are in a subdirectory unlike others
        if self.device_prefix == "/dev/pts" {
            return Some(format!("/dev/pts/{}", minor));
        }

        // If there is only one minor (e.g. /dev/console) it should not get a number
        if self.minor_range.start() == self.minor_range.end() {
            Some(self.device_prefix.clone())
        } else {
            let device_number = minor - self.minor_range.start();
            Some(format!("{}{}", self.device_prefix, device_number))
        }
    }
}

#[cfg(target_os = "linux")]
static TTY_DRIVERS_CACHE: LazyLock<Vec<TtyDriverEntry>> = LazyLock::new(|| {
    fs::read_to_string("/proc/tty/drivers")
        .map(|content| parse_proc_tty_drivers(&content))
        .unwrap_or_default()
});

#[cfg(target_os = "linux")]
fn parse_proc_tty_drivers(drivers_content: &str) -> Vec<TtyDriverEntry> {
    // Example lines:
    // /dev/tty             /dev/tty        5       0 system:/dev/tty
    // /dev/vc/0            /dev/vc/0       4       0 system:vtmaster
    // hvc                  /dev/hvc      229 0-7 system
    // serial               /dev/ttyS       4 64-95 serial
    // pty_slave            /dev/pts      136 0-1048575 pty:slave
    let regex = Regex::new(r"^[^ ]+ +([^ ]+) +(\d+) +(\d+)(?:-(\d+))?").unwrap();

    let mut entries = Vec::new();
    for line in drivers_content.lines() {
        let Some(captures) = regex.captures(line) else {
            continue;
        };

        let device_prefix = captures[1].to_string();
        let Ok(major) = captures[2].parse::<u32>() else {
            continue;
        };
        let Ok(min_minor) = captures[3].parse::<u32>() else {
            continue;
        };
        let max_minor = captures
            .get(4)
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .unwrap_or(min_minor);

        entries.push(TtyDriverEntry::new(
            device_prefix,
            major,
            min_minor..=max_minor,
        ));
    }

    entries
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Teletype {
    Known(String),
    Unknown,
}

impl Teletype {
    #[cfg(target_os = "linux")]
    pub fn from_tty_nr(tty_nr: u64) -> Self {
        Self::from_tty_nr_impl(tty_nr, &TTY_DRIVERS_CACHE)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn from_tty_nr(_tty_nr: u64) -> Self {
        Self::Unknown
    }

    #[cfg(target_os = "linux")]
    fn from_tty_nr_impl(tty_nr: u64, drivers: &[TtyDriverEntry]) -> Self {
        use uucore::libc::{major, minor};

        if tty_nr == 0 {
            return Self::Unknown;
        }

        let (major_dev, minor_dev) = (major(tty_nr), minor(tty_nr));
        for entry in drivers.iter() {
            if let Some(device_path) = entry.device_path_if_matches(major_dev, minor_dev) {
                return Self::Known(device_path);
            }
        }

        Self::Unknown
    }
}

impl Display for Teletype {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Known(device_path) => write!(f, "{}", device_path),
            Self::Unknown => write!(f, "?"),
        }
    }
}

impl TryFrom<String> for Teletype {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value == "?" {
            return Ok(Self::Unknown);
        }

        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Teletype {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(PathBuf::from(value))
    }
}

impl TryFrom<PathBuf> for Teletype {
    type Error = ();

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let path_str = value.to_str().ok_or(())?;
        Ok(Self::Known(path_str.to_string()))
    }
}

/// State of process
/// https://www.man7.org/linux/man-pages//man5/proc_pid_stat.5.html
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunState {
    /// `R`, running
    Running,
    /// `S`, sleeping
    Sleeping,
    /// `D`, sleeping in an uninterruptible wait
    UninterruptibleWait,
    /// `Z`, zombie
    Zombie,
    /// `T`, stopped (on a signal)
    Stopped,
    /// `t`, tracing stop
    TraceStopped,
    /// `X`, dead
    Dead,
    /// `I`, idle
    Idle,
}

impl Display for RunState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Running => write!(f, "R"),
            Self::Sleeping => write!(f, "S"),
            Self::UninterruptibleWait => write!(f, "D"),
            Self::Zombie => write!(f, "Z"),
            Self::Stopped => write!(f, "T"),
            Self::TraceStopped => write!(f, "t"),
            Self::Dead => write!(f, "X"),
            Self::Idle => write!(f, "I"),
        }
    }
}

impl TryFrom<char> for RunState {
    type Error = io::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'R' => Ok(Self::Running),
            'S' => Ok(Self::Sleeping),
            'D' => Ok(Self::UninterruptibleWait),
            'Z' => Ok(Self::Zombie),
            'T' => Ok(Self::Stopped),
            't' => Ok(Self::TraceStopped),
            'X' => Ok(Self::Dead),
            'I' => Ok(Self::Idle),
            _ => Err(io::ErrorKind::InvalidInput.into()),
        }
    }
}

impl TryFrom<&str> for RunState {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            return Err(io::ErrorKind::InvalidInput.into());
        }

        Self::try_from(
            value
                .chars()
                .nth(0)
                .ok_or::<io::Error>(io::ErrorKind::InvalidInput.into())?,
        )
    }
}

impl TryFrom<String> for RunState {
    type Error = io::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&String> for RunState {
    type Error = io::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// Represents an entry in `/proc/<pid>/cgroup`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CgroupMembership {
    pub hierarchy_id: u32,
    pub controllers: Vec<String>,
    pub cgroup_path: String,
}

impl TryFrom<&str> for CgroupMembership {
    type Error = io::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 3 {
            return Err(io::ErrorKind::InvalidData.into());
        }

        Ok(CgroupMembership {
            hierarchy_id: parts[0]
                .parse::<u32>()
                .map_err(|_| io::ErrorKind::InvalidData)?,
            controllers: if parts[1].is_empty() {
                vec![]
            } else {
                parts[1].split(',').map(String::from).collect()
            },
            cgroup_path: parts[2].to_string(),
        })
    }
}

/// See https://www.man7.org/linux/man-pages/man7/namespaces.7.html
///
/// # Support status
///
/// **_Linux only._**
#[derive(Default)]
pub struct Namespace {
    pub ipc: Option<u64>,
    pub mnt: Option<u64>,
    pub net: Option<u64>,
    pub pid: Option<u64>,
    pub user: Option<u64>,
    pub uts: Option<u64>,
}

impl Namespace {
    pub fn new() -> Self {
        Namespace {
            ipc: None,
            mnt: None,
            net: None,
            pid: None,
            user: None,
            uts: None,
        }
    }

    #[cfg(target_os = "linux")]
    pub fn from_pid(pid: usize) -> io::Result<Self> {
        use std::os::fd::OwnedFd;

        use rustix::fs::{openat, statx, AtFlags, Mode, OFlags, StatxFlags, CWD};

        let f = |name: &str, fd: &OwnedFd| {
            statx(
                fd,
                name,
                AtFlags::empty(), // NO FOLLOW LINKS
                StatxFlags::INO,  // INNODE ONLY
            )
        };

        let ns_dir = openat(
            CWD,
            PathBuf::from(format!("/proc/{}/ns", pid)),
            OFlags::RDONLY | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let mut ns = Namespace::default();

        for (name, slot) in [
            ("ipc", &mut ns.ipc),
            ("mnt", &mut ns.mnt),
            ("net", &mut ns.net),
            ("pid", &mut ns.pid),
            ("user", &mut ns.user),
            ("uts", &mut ns.uts),
        ] {
            match f(name, &ns_dir) {
                Ok(st) => *slot = Some(st.stx_ino),
                Err(e) => return Err(e.into()),
            }
        }
        Ok(ns)
    }

    /// TODO: implementation for other system
    #[cfg(not(target_os = "linux"))]
    pub fn from_pid(_pid: usize) -> Result<Self, io::Error> {
        Ok(Namespace::new())
    }

    pub fn filter(&mut self, filters: &[&str]) {
        if !filters.contains(&"ipc") {
            self.ipc = None;
        }
        if !filters.contains(&"mnt") {
            self.mnt = None;
        }
        if !filters.contains(&"net") {
            self.net = None;
        }
        if !filters.contains(&"pid") {
            self.pid = None;
        }
        if !filters.contains(&"user") {
            self.user = None;
        }
        if !filters.contains(&"uts") {
            self.uts = None;
        }
    }

    pub fn matches(&self, ns: &Namespace) -> bool {
        ns.ipc.is_some()
            && self
                .ipc
                .as_ref()
                .is_some_and(|v| v == ns.ipc.as_ref().unwrap())
            || ns.mnt.is_some()
                && self
                    .mnt
                    .as_ref()
                    .is_some_and(|v| v == ns.mnt.as_ref().unwrap())
            || ns.net.is_some()
                && self
                    .net
                    .as_ref()
                    .is_some_and(|v| v == ns.net.as_ref().unwrap())
            || ns.pid.is_some()
                && self
                    .pid
                    .as_ref()
                    .is_some_and(|v| v == ns.pid.as_ref().unwrap())
            || ns.user.is_some()
                && self
                    .user
                    .as_ref()
                    .is_some_and(|v| v == ns.user.as_ref().unwrap())
            || ns.uts.is_some()
                && self
                    .uts
                    .as_ref()
                    .is_some_and(|v| v == ns.uts.as_ref().unwrap())
    }
}

/// Process ID and its information
#[derive(Debug, Clone, Default)]
pub struct ProcessInformation {
    pub pid: usize,
    pub cmdline: String,

    inner_status: String,
    inner_stat: String,

    /// Processed `/proc/self/status` file
    status: OnceLock<HashMap<String, String>>,
    /// Processed `/proc/self/stat` file
    stat: OnceLock<Vec<String>>,

    cached_start_time: Option<u64>,

    thread_ids: OnceLock<Vec<usize>>,
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

    pub fn from_pid(pid: usize) -> Result<Self, io::Error> {
        Self::try_new(PathBuf::from(format!("/proc/{}", pid)))
    }

    pub fn current_process_info() -> Result<ProcessInformation, io::Error> {
        #[cfg(target_os = "linux")]
        let pid = uucore::process::getpid();
        #[cfg(not(target_os = "linux"))]
        let pid = 0; // dummy

        Self::from_pid(pid as usize)
    }

    pub fn proc_status(&self) -> &str {
        &self.inner_status
    }

    pub fn proc_stat(&self) -> &str {
        &self.inner_stat
    }

    /// Collect information from `/proc/<pid>/status` file
    pub fn status(&self) -> &HashMap<String, String> {
        self.status.get_or_init(|| {
            self.inner_status
                .lines()
                .filter_map(|it| it.split_once(':'))
                .map(|it| (it.0.to_string(), it.1.trim_start().to_string()))
                .collect::<HashMap<_, _>>()
        })
    }

    /// Collect information from `/proc/<pid>/stat` file
    pub fn stat(&self) -> &Vec<String> {
        self.stat.get_or_init(|| stat_split(&self.inner_stat))
    }

    pub fn name(&mut self) -> Result<String, io::Error> {
        self.status()
            .get("Name")
            .cloned()
            .ok_or(io::ErrorKind::InvalidData.into())
    }

    fn get_numeric_stat_field(&mut self, index: usize) -> Result<u64, io::Error> {
        self.stat()
            .get(index)
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<u64>()
            .map_err(|_| io::ErrorKind::InvalidData.into())
    }

    /// Fetch start time from [ProcessInformation::cached_stat]
    ///
    /// - [The /proc Filesystem: Table 1-4](https://docs.kernel.org/filesystems/proc.html#id10)
    pub fn start_time(&mut self) -> Result<u64, io::Error> {
        if let Some(time) = self.cached_start_time {
            return Ok(time);
        }

        // Kernel doc: https://docs.kernel.org/filesystems/proc.html#process-specific-subdirectories
        // Table 1-4
        let time = self.get_numeric_stat_field(21)?;

        self.cached_start_time = Some(time);

        Ok(time)
    }

    pub fn ppid(&mut self) -> Result<u64, io::Error> {
        // the PPID is the fourth field in /proc/<PID>/stat
        // (https://www.kernel.org/doc/html/latest/filesystems/proc.html#id10)
        self.get_numeric_stat_field(3)
    }

    pub fn pgid(&mut self) -> Result<u64, io::Error> {
        // the process group ID is the fifth field in /proc/<PID>/stat
        // (https://www.kernel.org/doc/html/latest/filesystems/proc.html#id10)
        self.get_numeric_stat_field(4)
    }

    pub fn sid(&mut self) -> Result<u64, io::Error> {
        // the session ID is the sixth field in /proc/<PID>/stat
        // (https://www.kernel.org/doc/html/latest/filesystems/proc.html#id10)
        self.get_numeric_stat_field(5)
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

    /// Helper function to get a hex field from status and parse it as u64
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

    /// Returns the signal caught mask for the process
    pub fn signals_caught_mask(&mut self) -> Result<u64, io::Error> {
        self.get_hex_status_field("SigCgt")
    }

    /// Returns the pending signals mask for the process
    pub fn signals_pending_mask(&mut self) -> Result<u64, io::Error> {
        self.get_hex_status_field("SigPnd")
    }

    /// Returns the blocked signals mask for the process
    pub fn signals_blocked_mask(&mut self) -> Result<u64, io::Error> {
        self.get_hex_status_field("SigBlk")
    }

    /// Returns the ignored signals mask for the process
    pub fn signals_ignored_mask(&mut self) -> Result<u64, io::Error> {
        self.get_hex_status_field("SigIgn")
    }

    // Root directory of the process (which can be changed by chroot)
    pub fn root(&mut self) -> Result<PathBuf, io::Error> {
        read_link(format!("/proc/{}/root", self.pid))
    }

    /// Returns cgroups (both v1 and v2) that the process belongs to.
    pub fn cgroups(&mut self) -> Result<Vec<CgroupMembership>, io::Error> {
        fs::read_to_string(format!("/proc/{}/cgroup", self.pid))?
            .lines()
            .map(CgroupMembership::try_from)
            .collect()
    }

    /// Returns path to the v2 cgroup that the process belongs to.
    pub fn cgroup_v2_path(&mut self) -> Result<String, io::Error> {
        const V2_HIERARCHY_ID: u32 = 0;
        self.cgroups()?
            .iter()
            .find(|cg| cg.hierarchy_id == V2_HIERARCHY_ID)
            .map(|cg| cg.cgroup_path.clone())
            .ok_or(io::ErrorKind::NotFound.into())
    }

    /// Fetch run state from [ProcessInformation::cached_stat]
    ///
    /// - [The /proc Filesystem: Table 1-4](https://docs.kernel.org/filesystems/proc.html#id10)
    ///
    /// # Error
    ///
    /// If parsing failed, this function will return [io::ErrorKind::InvalidInput]
    pub fn run_state(&mut self) -> Result<RunState, io::Error> {
        RunState::try_from(self.stat().get(2).unwrap().as_str())
    }

    /// Get the controlling terminal from the tty_nr field in /proc/<pid>/stat
    ///
    /// Returns Teletype::Unknown if the process has no controlling terminal (tty_nr == 0)
    /// or if the tty_nr cannot be resolved to a device.
    pub fn tty(&mut self) -> Teletype {
        let tty_nr = match self.get_numeric_stat_field(6) {
            Ok(tty_nr) => tty_nr,
            Err(_) => return Teletype::Unknown,
        };

        Teletype::from_tty_nr(tty_nr)
    }

    pub fn thread_ids(&mut self) -> &[usize] {
        self.thread_ids.get_or_init(|| {
            let tids_dir = format!("/proc/{}/task", self.pid);
            WalkDir::new(tids_dir)
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
                .collect::<Vec<_>>()
        })
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
}
impl TryFrom<DirEntry> for ProcessInformation {
    type Error = io::Error;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let value = value.into_path();

        Self::try_new(value)
    }
}

impl Hash for ProcessInformation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Make it faster.
        self.pid.hash(state);
        self.inner_status.hash(state);
        self.inner_stat.hash(state);
    }
}

/// Parsing `/proc/self/stat` file.
///
/// TODO: If possible, test and use regex to replace this algorithm.
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

static THREAD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^/proc/[0-9]+$|^/proc/[0-9]+/task$|^/proc/[0-9]+/task/[0-9]+$").unwrap()
});

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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use std::collections::HashSet;
    #[cfg(target_os = "linux")]
    use uucore::process::getpid;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_tty_resolution() {
        let test_content = r#"/dev/tty             /dev/tty        5       0 system:/dev/tty
/dev/console         /dev/console    5       1 system:console
/dev/ptmx            /dev/ptmx       5       2 system
/dev/vc/0            /dev/vc/0       4       0 system:vtmaster
hvc                  /dev/hvc      229 0-7 system
serial               /dev/ttyS       4 64-95 serial
pty_slave            /dev/pts      136 0-1048575 pty:slave
pty_master           /dev/ptm      128 0-1048575 pty:master
unknown              /dev/tty        4 1-63 console"#;

        let expected_entries = vec![
            TtyDriverEntry::new("/dev/tty".to_string(), 5, 0..=0),
            TtyDriverEntry::new("/dev/console".to_string(), 5, 1..=1),
            TtyDriverEntry::new("/dev/ptmx".to_string(), 5, 2..=2),
            TtyDriverEntry::new("/dev/vc/0".to_string(), 4, 0..=0),
            TtyDriverEntry::new("/dev/hvc".to_string(), 229, 0..=7),
            TtyDriverEntry::new("/dev/ttyS".to_string(), 4, 64..=95),
            TtyDriverEntry::new("/dev/pts".to_string(), 136, 0..=1048575),
            TtyDriverEntry::new("/dev/ptm".to_string(), 128, 0..=1048575),
            TtyDriverEntry::new("/dev/tty".to_string(), 4, 1..=63),
        ];

        let parsed_entries = parse_proc_tty_drivers(test_content);
        assert_eq!(parsed_entries, expected_entries);

        let test_cases = vec![
            // (major, minor, expected_result)
            (0, 0, Teletype::Unknown),
            (5, 0, Teletype::Known("/dev/tty".to_string())),
            (5, 1, Teletype::Known("/dev/console".to_string())),
            (136, 123, Teletype::Known("/dev/pts/123".to_string())),
            (4, 64, Teletype::Known("/dev/ttyS0".to_string())),
            (4, 65, Teletype::Known("/dev/ttyS1".to_string())),
            (229, 3, Teletype::Known("/dev/hvc3".to_string())),
            (999, 999, Teletype::Unknown),
        ];

        for (major, minor, expected) in test_cases {
            let tty_nr = uucore::libc::makedev(major, minor);
            let result = Teletype::from_tty_nr_impl(tty_nr, &parsed_entries);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_run_state_conversion() {
        assert_eq!(RunState::try_from("R").unwrap(), RunState::Running);
        assert_eq!(RunState::try_from("S").unwrap(), RunState::Sleeping);
        assert_eq!(
            RunState::try_from("D").unwrap(),
            RunState::UninterruptibleWait
        );
        assert_eq!(RunState::try_from("T").unwrap(), RunState::Stopped);
        assert_eq!(RunState::try_from("Z").unwrap(), RunState::Zombie);
        assert_eq!(RunState::try_from("t").unwrap(), RunState::TraceStopped);
        assert_eq!(RunState::try_from("X").unwrap(), RunState::Dead);
        assert_eq!(RunState::try_from("I").unwrap(), RunState::Idle);

        assert!(RunState::try_from("G").is_err());
        assert!(RunState::try_from("Rg").is_err());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_walk_pid() {
        let find = walk_process().find(|it| it.pid == getpid() as usize);

        assert!(find.is_some());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_pid_entry() {
        use std::io::IsTerminal;

        let mut pid_entry = ProcessInformation::current_process_info().unwrap();

        if !std::io::stdout().is_terminal() && !std::io::stderr().is_terminal() {
            assert_eq!(pid_entry.tty(), Teletype::Unknown);
            return;
        }
        let mut result = WalkDir::new(format!("/proc/{}/fd", getpid()))
            .into_iter()
            .flatten()
            .map(DirEntry::into_path)
            .flat_map(|it| it.read_link())
            .flat_map(Teletype::try_from)
            .collect::<HashSet<_>>();

        if result.is_empty() {
            result.insert(Teletype::Unknown);
        }

        assert!(result.contains(&pid_entry.tty()));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_thread_ids() {
        let main_tid = unsafe { uucore::libc::gettid() };
        std::thread::spawn(move || {
            let mut pid_entry = ProcessInformation::current_process_info().unwrap();
            let thread_ids = pid_entry.thread_ids();

            assert!(thread_ids.contains(&(main_tid as usize)));

            let new_thread_tid = unsafe { uucore::libc::gettid() };
            assert!(thread_ids.contains(&(new_thread_tid as usize)));
        })
        .join()
        .unwrap();
    }

    #[test]
    fn test_stat_split() {
        let case = "32 (idle_inject/3) S 2 0 0 0 -1 69238848 0 0 0 0 0 0 0 0 -51 0 1 0 34 0 0 18446744073709551615 0 0 0 0 0 0 0 2147483647 0 0 0 0 17 3 50 1 0 0 0 0 0 0 0 0 0 0 0";
        assert!(stat_split(case)[1] == "idle_inject/3");

        let case = "3508 (sh) S 3478 3478 3478 0 -1 4194304 67 0 0 0 0 0 0 0 20 0 1 0 11911 2961408 238 18446744073709551615 94340156948480 94340157028757 140736274114368 0 0 0 0 4096 65538 1 0 0 17 8 0 0 0 0 0 94340157054704 94340157059616 94340163108864 140736274122780 140736274122976 140736274122976 140736274124784 0";
        assert!(stat_split(case)[1] == "sh");

        let case = "47246 (kworker /10:1-events) I 2 0 0 0 -1 69238880 0 0 0 0 17 29 0 0 20 0 1 0 1396260 0 0 18446744073709551615 0 0 0 0 0 0 0 2147483647 0 0 0 0 17 10 0 0 0 0 0 0 0 0 0 0 0 0 0";
        assert!(stat_split(case)[1] == "kworker /10:1-events");

        let case = "83875 (sleep (2) .sh) S 75750 83875 75750 34824 83875 4194304 173 0 0 0 0 0 0 0 20 0 1 0 18366278 23187456 821 18446744073709551615 94424231874560 94424232638561 140734866834816 0 0 0 65536 4 65538 1 0 0 17 6 0 0 0 0 0 94424232876752 94424232924772 94424259932160 140734866837287 140734866837313 140734866837313 140734866841576 0";
        assert!(stat_split(case)[1] == "sleep (2) .sh");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_ids() {
        let mut pid_entry = ProcessInformation::current_process_info().unwrap();
        assert_eq!(
            pid_entry.ppid().unwrap(),
            unsafe { uucore::libc::getppid() } as u64
        );
        assert_eq!(
            pid_entry.pgid().unwrap(),
            unsafe { uucore::libc::getpgid(0) } as u64
        );
        assert_eq!(pid_entry.sid().unwrap(), unsafe { uucore::libc::getsid(0) }
            as u64);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_uid_gid() {
        let mut pid_entry = ProcessInformation::current_process_info().unwrap();
        assert_eq!(pid_entry.uid().unwrap(), uucore::process::getuid());
        assert_eq!(pid_entry.euid().unwrap(), uucore::process::geteuid());
        assert_eq!(pid_entry.gid().unwrap(), uucore::process::getgid());
        assert_eq!(pid_entry.egid().unwrap(), uucore::process::getegid());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_root() {
        let mut pid_entry = ProcessInformation::current_process_info().unwrap();
        assert_eq!(pid_entry.root().unwrap(), PathBuf::from("/"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_cgroups() {
        let mut pid_entry = ProcessInformation::from_pid(1).unwrap();
        if pid_entry.name().unwrap() == "systemd" {
            let cgroups = pid_entry.cgroups().unwrap();
            if let Some(membership) = cgroups.iter().find(|cg| cg.hierarchy_id == 0) {
                let expected = CgroupMembership {
                    hierarchy_id: 0,
                    controllers: vec![],
                    cgroup_path: "/init.scope".to_string(),
                };
                assert_eq!(membership, &expected);
                assert_eq!(pid_entry.cgroup_v2_path().unwrap(), "/init.scope");
            }
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_namespaces() {
        let pid_entry = ProcessInformation::current_process_info().unwrap();
        let namespaces = pid_entry.namespaces().unwrap();

        assert!(namespaces.ipc.is_some());
        assert!(namespaces.mnt.is_some());
        assert!(namespaces.net.is_some());
        assert!(namespaces.pid.is_some());
        assert!(namespaces.user.is_some());
        assert!(namespaces.uts.is_some());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_environ() {
        let pid_entry = ProcessInformation::current_process_info().unwrap();
        let env_vars = pid_entry.env_vars().unwrap();

        assert_eq!(
            *env_vars.get("HOME").unwrap(),
            std::env::var("HOME").unwrap()
        );
    }
}
