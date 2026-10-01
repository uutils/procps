#[cfg(target_os = "linux")]
use regex::Regex;

use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

#[cfg(target_os = "linux")]
use std::{fs, ops::RangeInclusive, sync::LazyLock};

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
fn parse_proc_tty_drivers(drivers_content: &str) -> Vec<TtyDriverEntry> {
    // Example lines:
    // /dev/tty             /dev/tty        5       0           system:/dev/tty
    // /dev/vc/0            /dev/vc/0       4       0           system:vtmaster
    // hvc                  /dev/hvc        229     0-7         system
    // serial               /dev/ttyS       4       64-95       serial
    // pty_slave            /dev/pts        136     0-1048575   pty:slave
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

#[cfg(target_os = "linux")]
static TTY_DRIVERS_CACHE: LazyLock<Vec<TtyDriverEntry>> = LazyLock::new(|| {
    fs::read_to_string("/proc/tty/drivers")
        .map(|content| parse_proc_tty_drivers(&content))
        .unwrap_or_default()
});

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
        use rustix::fs::{major, minor};

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

#[cfg(test)]
mod tests {
    use super::*;

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
            let tty_nr = rustix::fs::makedev(major, minor);
            let result = Teletype::from_tty_nr_impl(tty_nr, &parsed_entries);
            assert_eq!(result, expected);
        }
    }
}
