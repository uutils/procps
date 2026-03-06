// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::fmt::{self, Display, Formatter};
use std::io;

/// Errors that can occur when working with process information
#[derive(Debug, Clone)]
pub enum ProcessError {
    /// Process with given PID does not exist
    NotFound(usize),
    /// Permission denied when accessing process information
    PermissionDenied(usize),
    /// Invalid or malformed data in /proc filesystem
    InvalidData(String),
    /// Requested feature is not supported on this platform
    Unsupported(String),
    /// I/O error occurred
    Io(String),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NotFound(pid) => write!(f, "Process {} not found", pid),
            Self::PermissionDenied(pid) => write!(f, "Permission denied for process {}", pid),
            Self::InvalidData(msg) => write!(f, "Invalid process data: {}", msg),
            Self::Unsupported(feature) => write!(f, "Feature not supported: {}", feature),
            Self::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for ProcessError {}

impl From<std::io::Error> for ProcessError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

/// Macro to define RunState variants with their character representations
/// Usage: define_runstates!(Running => 'R', Sleeping => 'S', ...)
macro_rules! define_runstates {
    ($($variant:ident => $char:expr),+ $(,)?) => {
        /// Process run state from `/proc/<pid>/stat` field 3.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum RunState {
            $(#[doc = concat!("Process state: ", stringify!($char))]
            $variant),+
        }

        impl Display for RunState {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match self {
                    $(Self::$variant => write!(f, "{}", $char)),+
                }
            }
        }

        impl TryFrom<char> for RunState {
            type Error = io::Error;

            fn try_from(value: char) -> Result<Self, Self::Error> {
                match value {
                    $($char => Ok(Self::$variant)),+,
                    _ => Err(io::ErrorKind::InvalidInput.into()),
                }
            }
        }
    };
}

// Define all RunState variants with their character codes
define_runstates!(
    Running => 'R',
    Sleeping => 'S',
    UninterruptibleWait => 'D',
    Zombie => 'Z',
    Stopped => 'T',
    TraceStopped => 't',
    Dead => 'X',
    Idle => 'I'
);

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

/// Macro to define Teletype variants with their device paths and names
macro_rules! define_teletypes {
    ($($variant:ident => $path:expr, $name:expr),+ $(,)?) => {
        /// Terminal device associated with a process.
        ///
        /// Represents the controlling terminal (TTY) for a process. Parsed from `/proc/<pid>/stat`
        /// field 7 (tty_nr) or from symbolic link resolution.
        ///
        /// # Examples
        ///
        /// ```
        /// use uu_uuproc::Teletype;
        /// use std::convert::TryFrom;
        ///
        /// // Parse from tty_nr in /proc/*/stat (major/minor device number)
        /// let tty = Teletype::try_from(34816_u64).unwrap();
        /// assert_eq!(tty, Teletype::Pts(0));
        ///
        /// // Parse from path string
        /// let tty = Teletype::try_from("/dev/pts/0").unwrap();
        /// assert_eq!(tty.to_string(), "/dev/pts/0");
        ///
        /// // Unknown TTY for processes without a controlling terminal
        /// assert_eq!(Teletype::Unknown.to_string(), "?");
        /// ```
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum Teletype {
            $($variant(u64)),+,
            Unknown,
        }

        impl Display for Teletype {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match self {
                    $(Self::$variant(id) => write!(f, "{}{}", $path, id)),+,
                    Self::Unknown => write!(f, "?"),
                }
            }
        }

        impl TryFrom<std::path::PathBuf> for Teletype {
            type Error = ();

            fn try_from(value: std::path::PathBuf) -> Result<Self, Self::Error> {
                // Special case for pts (has directory component)
                let mut iter = value.iter();
                if let (Some(_), Some(num)) = (iter.find(|it| *it == "pts"), iter.next()) {
                    return num
                        .to_str()
                        .ok_or(())?
                        .parse::<u64>()
                        .map_err(|_| ())
                        .map(Teletype::Pts);
                }

                let path = value.to_str().ok_or(())?;

                // Try each device type
                $(
                    if path.contains($name) {
                        let f = |prefix: &str| {
                            value
                                .iter()
                                .next_back()?
                                .to_str()?
                                .strip_prefix(prefix)?
                                .parse::<u64>()
                                .ok()
                        };
                        return f($name).ok_or(()).map(Teletype::$variant);
                    }
                )+

                Err(())
            }
        }
    };
}

define_teletypes!(
    TtyS => "/dev/ttyS", "ttyS",
    Tty => "/dev/tty", "tty",
    Pts => "/dev/pts/", "pts"
);

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

/// Macro to define cgroup parsing with configurable delimiter and field count
macro_rules! define_cgroup_parser {
    ($delimiter:expr, $field_count:expr) => {
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
                let parts: Vec<&str> = value.split($delimiter).collect();
                if parts.len() != $field_count {
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
    };
}

// Define cgroup parser with ':' delimiter and 3 fields
define_cgroup_parser!(':', 3);

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

    pub fn filter(&mut self, filters: &[&str]) {
        macro_rules! filter_field {
            ($($field:ident),+) => {
                $(
                    if !filters.contains(&stringify!($field)) {
                        self.$field = None;
                    }
                )+
            };
        }
        filter_field!(ipc, mnt, net, pid, user, uts);
    }

    pub fn matches(&self, ns: &Namespace) -> bool {
        macro_rules! check_match {
            ($($field:ident),+) => {
                $(
                    (ns.$field.is_some()
                        && self
                            .$field
                            .as_ref()
                            .is_some_and(|v| v == ns.$field.as_ref().unwrap()))
                )||+
            };
        }
        check_match!(ipc, mnt, net, pid, user, uts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teletype_display_tty() {
        let tty = Teletype::Tty(0);
        assert_eq!(tty.to_string(), "/dev/tty0");

        let tty = Teletype::Tty(1);
        assert_eq!(tty.to_string(), "/dev/tty1");

        let tty = Teletype::Tty(63);
        assert_eq!(tty.to_string(), "/dev/tty63");
    }

    #[test]
    fn test_teletype_display_ttys() {
        let ttys = Teletype::TtyS(0);
        assert_eq!(ttys.to_string(), "/dev/ttyS0");

        let ttys = Teletype::TtyS(1);
        assert_eq!(ttys.to_string(), "/dev/ttyS1");
    }

    #[test]
    fn test_teletype_display_pts() {
        let pts = Teletype::Pts(0);
        assert_eq!(pts.to_string(), "/dev/pts/0");

        let pts = Teletype::Pts(1);
        assert_eq!(pts.to_string(), "/dev/pts/1");

        let pts = Teletype::Pts(999);
        assert_eq!(pts.to_string(), "/dev/pts/999");
    }

    #[test]
    fn test_teletype_display_unknown() {
        let unknown = Teletype::Unknown;
        assert_eq!(unknown.to_string(), "?");
    }

    #[test]
    fn test_teletype_from_string_unknown() {
        let result = Teletype::try_from("?".to_string());
        assert_eq!(result, Ok(Teletype::Unknown));
    }

    #[test]
    fn test_teletype_from_str_tty() {
        let result = Teletype::try_from("/dev/tty0");
        assert_eq!(result, Ok(Teletype::Tty(0)));

        let result = Teletype::try_from("/dev/tty1");
        assert_eq!(result, Ok(Teletype::Tty(1)));
    }

    #[test]
    fn test_teletype_from_str_ttys() {
        let result = Teletype::try_from("/dev/ttyS0");
        assert_eq!(result, Ok(Teletype::TtyS(0)));

        let result = Teletype::try_from("/dev/ttyS1");
        assert_eq!(result, Ok(Teletype::TtyS(1)));
    }

    #[test]
    fn test_teletype_from_str_pts() {
        let result = Teletype::try_from("/dev/pts/0");
        assert_eq!(result, Ok(Teletype::Pts(0)));

        let result = Teletype::try_from("/dev/pts/1");
        assert_eq!(result, Ok(Teletype::Pts(1)));

        let result = Teletype::try_from("/dev/pts/999");
        assert_eq!(result, Ok(Teletype::Pts(999)));
    }

    #[test]
    fn test_teletype_from_u64_zero() {
        let result = Teletype::try_from(0u64);
        assert_eq!(result, Ok(Teletype::Unknown));
    }

    #[test]
    fn test_teletype_from_u64_tty() {
        // major=4, minor=0: (4 << 8) | 0 = 1024
        let result = Teletype::try_from(1024u64);
        assert_eq!(result, Ok(Teletype::Tty(0)));

        // major=4, minor=1: (4 << 8) | 1 = 1025
        let result = Teletype::try_from(1025u64);
        assert_eq!(result, Ok(Teletype::Tty(1)));
    }

    #[test]
    fn test_teletype_from_u64_ttys() {
        // major=5, minor=0: (5 << 8) | 0 = 1280
        let result = Teletype::try_from(1280u64);
        assert_eq!(result, Ok(Teletype::TtyS(0)));

        // major=5, minor=1: (5 << 8) | 1 = 1281
        let result = Teletype::try_from(1281u64);
        assert_eq!(result, Ok(Teletype::TtyS(1)));
    }

    #[test]
    fn test_teletype_from_u64_pts() {
        // major=136, minor=0: (136 << 8) | 0 = 34816
        let result = Teletype::try_from(34816u64);
        assert_eq!(result, Ok(Teletype::Pts(0)));

        // major=136, minor=1: (136 << 8) | 1 = 34817
        let result = Teletype::try_from(34817u64);
        assert_eq!(result, Ok(Teletype::Pts(1)));

        // major=137, minor=0: (137 << 8) | 0 = 35072
        let result = Teletype::try_from(35072u64);
        assert_eq!(result, Ok(Teletype::Pts(256)));
    }

    #[test]
    fn test_teletype_equality() {
        assert_eq!(Teletype::Tty(0), Teletype::Tty(0));
        assert_ne!(Teletype::Tty(0), Teletype::Tty(1));
        assert_ne!(Teletype::Tty(0), Teletype::TtyS(0));
        assert_ne!(Teletype::Tty(0), Teletype::Pts(0));
        assert_ne!(Teletype::Tty(0), Teletype::Unknown);
    }

    #[test]
    fn test_runstate_display() {
        assert_eq!(RunState::Running.to_string(), "R");
        assert_eq!(RunState::Sleeping.to_string(), "S");
        assert_eq!(RunState::UninterruptibleWait.to_string(), "D");
        assert_eq!(RunState::Zombie.to_string(), "Z");
        assert_eq!(RunState::Stopped.to_string(), "T");
        assert_eq!(RunState::TraceStopped.to_string(), "t");
        assert_eq!(RunState::Dead.to_string(), "X");
        assert_eq!(RunState::Idle.to_string(), "I");
    }

    #[test]
    fn test_runstate_from_char() {
        let result = RunState::try_from('R');
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Running);

        let result = RunState::try_from('S');
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Sleeping);

        let result = RunState::try_from('D');
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::UninterruptibleWait);

        let result = RunState::try_from('Z');
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Zombie);

        let result = RunState::try_from('T');
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Stopped);

        let result = RunState::try_from('t');
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::TraceStopped);

        let result = RunState::try_from('X');
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Dead);

        let result = RunState::try_from('I');
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Idle);
    }

    #[test]
    fn test_runstate_from_char_invalid() {
        assert!(RunState::try_from('Q').is_err());
        assert!(RunState::try_from('A').is_err());
        assert!(RunState::try_from('0').is_err());
    }

    #[test]
    fn test_runstate_from_str() {
        let result = RunState::try_from("R");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Running);

        let result = RunState::try_from("S");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Sleeping);

        let result = RunState::try_from("D");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::UninterruptibleWait);
    }

    #[test]
    fn test_runstate_from_str_invalid() {
        assert!(RunState::try_from("RS").is_err());
        assert!(RunState::try_from("").is_err());
        assert!(RunState::try_from("invalid").is_err());
    }

    #[test]
    fn test_runstate_from_string() {
        let result = RunState::try_from("R".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Running);

        let result = RunState::try_from("S".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RunState::Sleeping);
    }

    #[test]
    fn test_cgroup_membership_from_str() {
        let result = CgroupMembership::try_from("1:cpu,cpuacct:/user.slice");
        assert!(result.is_ok());

        let cgroup = result.unwrap();
        assert_eq!(cgroup.hierarchy_id, 1);
        assert_eq!(cgroup.controllers, vec!["cpu", "cpuacct"]);
        assert_eq!(cgroup.cgroup_path, "/user.slice");
    }

    #[test]
    fn test_cgroup_membership_empty_controllers() {
        let result = CgroupMembership::try_from("1::/user.slice");
        assert!(result.is_ok());

        let cgroup = result.unwrap();
        assert_eq!(cgroup.hierarchy_id, 1);
        assert_eq!(cgroup.controllers, Vec::<String>::new());
        assert_eq!(cgroup.cgroup_path, "/user.slice");
    }

    #[test]
    fn test_cgroup_membership_invalid_format() {
        assert!(CgroupMembership::try_from("invalid").is_err());
        assert!(CgroupMembership::try_from("1:cpu").is_err());
        assert!(CgroupMembership::try_from("").is_err());
    }

    #[test]
    fn test_cgroup_membership_invalid_hierarchy_id() {
        assert!(CgroupMembership::try_from("abc:cpu:/path").is_err());
    }

    #[test]
    fn test_namespace_new() {
        let ns = Namespace::new();
        assert_eq!(ns.ipc, None);
        assert_eq!(ns.mnt, None);
        assert_eq!(ns.net, None);
        assert_eq!(ns.pid, None);
        assert_eq!(ns.user, None);
        assert_eq!(ns.uts, None);
    }

    #[test]
    fn test_namespace_filter() {
        let mut ns = Namespace {
            ipc: Some("ipc_id".to_string()),
            mnt: Some("mnt_id".to_string()),
            net: Some("net_id".to_string()),
            pid: Some("pid_id".to_string()),
            user: Some("user_id".to_string()),
            uts: Some("uts_id".to_string()),
        };

        ns.filter(&["ipc", "pid"]);

        assert_eq!(ns.ipc, Some("ipc_id".to_string()));
        assert_eq!(ns.mnt, None);
        assert_eq!(ns.net, None);
        assert_eq!(ns.pid, Some("pid_id".to_string()));
        assert_eq!(ns.user, None);
        assert_eq!(ns.uts, None);
    }

    #[test]
    fn test_namespace_filter_empty() {
        let mut ns = Namespace {
            ipc: Some("ipc_id".to_string()),
            mnt: Some("mnt_id".to_string()),
            net: Some("net_id".to_string()),
            pid: Some("pid_id".to_string()),
            user: Some("user_id".to_string()),
            uts: Some("uts_id".to_string()),
        };

        ns.filter(&[]);

        assert_eq!(ns.ipc, None);
        assert_eq!(ns.mnt, None);
        assert_eq!(ns.net, None);
        assert_eq!(ns.pid, None);
        assert_eq!(ns.user, None);
        assert_eq!(ns.uts, None);
    }

    #[test]
    fn test_namespace_matches() {
        let ns1 = Namespace {
            ipc: Some("ipc_id".to_string()),
            mnt: None,
            net: None,
            pid: None,
            user: None,
            uts: None,
        };

        let ns2 = Namespace {
            ipc: Some("ipc_id".to_string()),
            mnt: None,
            net: None,
            pid: None,
            user: None,
            uts: None,
        };

        assert!(ns1.matches(&ns2));
    }

    #[test]
    fn test_namespace_matches_different() {
        let ns1 = Namespace {
            ipc: Some("ipc_id_1".to_string()),
            mnt: None,
            net: None,
            pid: None,
            user: None,
            uts: None,
        };

        let ns2 = Namespace {
            ipc: Some("ipc_id_2".to_string()),
            mnt: None,
            net: None,
            pid: None,
            user: None,
            uts: None,
        };

        assert!(!ns1.matches(&ns2));
    }

    #[test]
    fn test_namespace_equality() {
        let ns1 = Namespace::new();
        let ns2 = Namespace::new();
        assert_eq!(ns1, ns2);
    }
}
