// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use regex::Regex;
use std::fs::read_link;
use std::hash::Hash;
use std::sync::LazyLock;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs, io,
    path::PathBuf,
    rc::Rc,
};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Teletype {
    Tty(u64),
    TtyS(u64),
    Pts(u64),
    Unknown,
}

impl Display for Teletype {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Tty(id) => write!(f, "/dev/tty{id}"),
            Self::TtyS(id) => write!(f, "/dev/ttyS{id}"),
            Self::Pts(id) => write!(f, "/dev/pts/{id}"),
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
        // Three case: /dev/pts/* , /dev/ttyS**, /dev/tty**

        let mut iter = value.iter();
        // Case 1

        // Considering this format: **/**/pts/<num>
        if let (Some(_), Some(num)) = (iter.find(|it| *it == "pts"), iter.next()) {
            return num
                .to_str()
                .ok_or(())?
                .parse::<u64>()
                .map_err(|_| ())
                .map(Teletype::Pts);
        };

        // Considering this format: **/**/ttyS** then **/**/tty**
        let path = value.to_str().ok_or(())?;

        let f = |prefix: &str| {
            value
                .iter()
                .next_back()?
                .to_str()?
                .strip_prefix(prefix)?
                .parse::<u64>()
                .ok()
        };

        if path.contains("ttyS") {
            // Case 2
            f("ttyS").ok_or(()).map(Teletype::TtyS)
        } else if path.contains("tty") {
            // Case 3
            f("tty").ok_or(()).map(Teletype::Tty)
        } else {
            Err(())
        }
    }
}

impl TryFrom<u64> for Teletype {
    type Error = ();

    fn try_from(tty_nr: u64) -> Result<Self, Self::Error> {
        // tty_nr is 0 for processes without a controlling terminal
        if tty_nr == 0 {
            return Ok(Self::Unknown);
        }

        // Extract major and minor device numbers
        // In Linux, tty_nr is encoded as: (major << 8) | minor
        // However, for pts devices, the encoding is different: major is 136-143
        let major = (tty_nr >> 8) & 0xFFF;
        let minor = (tty_nr & 0xFF) | ((tty_nr >> 12) & 0xFFF00);

        match major {
            // Virtual console terminals (/dev/tty1, /dev/tty2, etc.)
            4 => Ok(Self::Tty(minor)),
            // Serial terminals (/dev/ttyS0, /dev/ttyS1, etc.)
            5 => Ok(Self::TtyS(minor)),
            // Pseudo-terminals (/dev/pts/0, /dev/pts/1, etc.)
            // pts major numbers are 136-143
            136..=143 => {
                let pts_num = (major - 136) * 256 + minor;
                Ok(Self::Pts(pts_num))
            }
            // Unknown terminal type
            _ => Ok(Self::Unknown),
        }
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

// See https://www.man7.org/linux/man-pages/man7/namespaces.7.html
#[derive(Default)]
pub struct Namespace {
    pub ipc: Option<String>,
    pub mnt: Option<String>,
    pub net: Option<String>,
    pub pid: Option<String>,
    pub user: Option<String>,
    pub uts: Option<String>,
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

    pub fn from_pid(pid: usize) -> Result<Self, io::Error> {
        let mut ns = Namespace::new();
        let path = PathBuf::from(format!("/proc/{pid}/ns"));
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(value) = read_link(entry.path()) {
                    match name {
                        "ipc" => ns.ipc = Some(value.to_str().unwrap_or_default().to_string()),
                        "mnt" => ns.mnt = Some(value.to_str().unwrap_or_default().to_string()),
                        "net" => ns.net = Some(value.to_str().unwrap_or_default().to_string()),
                        "pid" => ns.pid = Some(value.to_str().unwrap_or_default().to_string()),
                        "user" => ns.user = Some(value.to_str().unwrap_or_default().to_string()),
                        "uts" => ns.uts = Some(value.to_str().unwrap_or_default().to_string()),
                        _ => {}
                    }
                }
            }
        }
        Ok(ns)
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

    pub fn current_process_info() -> Result<ProcessInformation, io::Error> {
        use std::str::FromStr;

        #[cfg(target_os = "linux")]
        let pid = uucore::process::getpid();
        #[cfg(not(target_os = "linux"))]
        let pid = 0; // dummy

        ProcessInformation::try_new(PathBuf::from_str(&format!("/proc/{pid}")).unwrap())
    }

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

    /// Collect information from `/proc/<pid>/stat` file
    pub fn stat(&mut self) -> Rc<Vec<String>> {
        if let Some(c) = &self.cached_stat {
            return Rc::clone(c);
        }

        let result: Vec<_> = stat_split(&self.inner_stat);

        let result = Rc::new(result);
        self.cached_stat = Some(Rc::clone(&result));
        Rc::clone(&result)
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

    pub fn tty_nr(&mut self) -> Result<u64, io::Error> {
        // the tty_nr is the seventh field in /proc/<PID>/stat
        // (https://www.kernel.org/doc/html/latest/filesystems/proc.html#id10)
        self.get_numeric_stat_field(6)
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

    /// Get the controlling terminal of the process.
    ///
    /// This function first tries to get the terminal from `/proc/<pid>/stat` (field 7, tty_nr)
    /// which is world-readable and doesn't require special permissions.
    /// Only if that fails, it falls back to scanning `/proc/<pid>/fd` directory.
    ///
    /// Returns [Teletype::Unknown] if the process has no controlling terminal.
    pub fn tty(&mut self) -> Teletype {
        // First try to get tty_nr from stat file (always accessible)
        if let Ok(tty_nr) = self.tty_nr() {
            if let Ok(tty) = Teletype::try_from(tty_nr) {
                return tty;
            }
        }

        // Fall back to scanning /proc/<pid>/fd directory
        // This requires permissions to read the process's fd directory
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

    pub fn thread_ids(&mut self) -> Rc<Vec<usize>> {
        if let Some(c) = &self.cached_thread_ids {
            return Rc::clone(c);
        }

        let tids_dir = format!("/proc/{}/task", self.pid);
        let result = Rc::new(
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
                .collect::<Vec<_>>(),
        );

        self.cached_thread_ids = Some(Rc::clone(&result));
        Rc::clone(&result)
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
    fn test_tty_nr_decoding() {
        // Test no controlling terminal
        assert_eq!(Teletype::try_from(0u64).unwrap(), Teletype::Unknown);

        // Test virtual console terminals (/dev/tty1, /dev/tty2, etc.)
        // tty1: major=4, minor=1 => (4 << 8) | 1 = 1025
        assert_eq!(Teletype::try_from(1025u64).unwrap(), Teletype::Tty(1));
        // tty12: major=4, minor=12 => (4 << 8) | 12 = 1036
        assert_eq!(Teletype::try_from(1036u64).unwrap(), Teletype::Tty(12));

        // Test serial terminals (/dev/ttyS0, /dev/ttyS1, etc.)
        // ttyS0: major=5, minor=0 => (5 << 8) | 0 = 1280
        assert_eq!(Teletype::try_from(1280u64).unwrap(), Teletype::TtyS(0));
        // ttyS1: major=5, minor=1 => (5 << 8) | 1 = 1281
        assert_eq!(Teletype::try_from(1281u64).unwrap(), Teletype::TtyS(1));

        // Test pseudo-terminals (/dev/pts/0, /dev/pts/1, etc.)
        // pts/0: major=136, minor=0 => (136 << 8) | 0 = 34816
        assert_eq!(Teletype::try_from(34816u64).unwrap(), Teletype::Pts(0));
        // pts/1: major=136, minor=1 => (136 << 8) | 1 = 34817
        assert_eq!(Teletype::try_from(34817u64).unwrap(), Teletype::Pts(1));
        // pts/256: major=137, minor=0 => (137 << 8) | 0 = 35072
        assert_eq!(Teletype::try_from(35072u64).unwrap(), Teletype::Pts(256));
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
        let mut pid_entry = ProcessInformation::current_process_info().unwrap();
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
        let mut pid_entry = ProcessInformation::try_new("/proc/1".into()).unwrap();
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
