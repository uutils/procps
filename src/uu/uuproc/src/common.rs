// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::fmt::{self, Display, Formatter};
use std::io;

/// Terminal type for a process
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
        Self::try_from(std::path::PathBuf::from(value))
    }
}

impl TryFrom<std::path::PathBuf> for Teletype {
    type Error = ();

    fn try_from(value: std::path::PathBuf) -> Result<Self, Self::Error> {
        let mut iter = value.iter();

        if let (Some(_), Some(num)) = (iter.find(|it| *it == "pts"), iter.next()) {
            return num
                .to_str()
                .ok_or(())?
                .parse::<u64>()
                .map_err(|_| ())
                .map(Teletype::Pts);
        };

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
            f("ttyS").ok_or(()).map(Teletype::TtyS)
        } else if path.contains("tty") {
            f("tty").ok_or(()).map(Teletype::Tty)
        } else {
            Err(())
        }
    }
}

impl TryFrom<u64> for Teletype {
    type Error = ();

    fn try_from(tty_nr: u64) -> Result<Self, Self::Error> {
        if tty_nr == 0 {
            return Ok(Self::Unknown);
        }

        let major = (tty_nr >> 8) & 0xFFF;
        let minor = (tty_nr & 0xFF) | ((tty_nr >> 12) & 0xFFF00);

        match major {
            4 => Ok(Self::Tty(minor)),
            5 => Ok(Self::TtyS(minor)),
            136..=143 => {
                let pts_num = (major - 136) * 256 + minor;
                Ok(Self::Pts(pts_num))
            }
            _ => Ok(Self::Unknown),
        }
    }
}

/// Process run state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunState {
    /// Running
    Running,
    /// Sleeping
    Sleeping,
    /// Sleeping in an uninterruptible wait
    UninterruptibleWait,
    /// Zombie
    Zombie,
    /// Stopped (on a signal)
    Stopped,
    /// Tracing stop
    TraceStopped,
    /// Dead
    Dead,
    /// Idle
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
                .next()
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

/// Cgroup membership information
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

/// Process namespace information
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
        let path = std::path::PathBuf::from(format!("/proc/{pid}/ns"));
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(value) = std::fs::read_link(entry.path()) {
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
