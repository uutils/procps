// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::{CgroupMembership, Namespace, RunState, Teletype};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

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

    /// Collect information from `/proc/<pid>/status` file
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

    /// Collect information from `/proc/<pid>/stat` file
    #[allow(dead_code)]
    fn stat(&mut self) -> Rc<Vec<String>> {
        if let Some(c) = &self.cached_stat {
            return Rc::clone(c);
        }

        let result: Vec<_> = Self::stat_split(&self.inner_stat);

        let result = Rc::new(result);
        self.cached_stat = Some(Rc::clone(&result));
        Rc::clone(&result)
    }

    /// Helper function to split /proc/<pid>/stat content
    /// Handles process names with spaces/parentheses correctly
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    pub fn name(&mut self) -> Result<String, io::Error> {
        self.status()
            .get("Name")
            .cloned()
            .ok_or(io::ErrorKind::InvalidData.into())
    }

    pub fn ppid(&mut self) -> Result<u64, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn pgid(&mut self) -> Result<u64, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn sid(&mut self) -> Result<u64, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
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
        Teletype::Unknown
    }

    pub fn run_state(&mut self) -> Result<RunState, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn start_time(&mut self) -> Result<u64, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn env_vars(&self) -> Result<HashMap<String, String>, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn namespaces(&self) -> Result<Namespace, io::Error> {
        Ok(Namespace::new())
    }

    pub fn cgroups(&mut self) -> Result<Vec<CgroupMembership>, io::Error> {
        Ok(vec![])
    }

    pub fn root(&mut self) -> Result<PathBuf, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn thread_ids(&mut self) -> Result<Vec<usize>, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn signals_pending_mask(&mut self) -> Result<u64, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn signals_blocked_mask(&mut self) -> Result<u64, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn signals_ignored_mask(&mut self) -> Result<u64, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn signals_caught_mask(&mut self) -> Result<u64, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }
}

/// Iterate over all processes on Linux
pub fn walk_process() -> impl Iterator<Item = ProcessInformation> {
    std::iter::empty()
}

/// Iterate over all threads on Linux
pub fn walk_threads() -> impl Iterator<Item = ProcessInformation> {
    std::iter::empty()
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_try_new_from_self() {
        // Test creating ProcessInformation from current process
        // Use std::process::id() to get actual PID instead of /proc/self which resolves differently
        let pid = std::process::id();
        let proc_path = PathBuf::from(format!("/proc/{}", pid));

        let result = ProcessInformation::try_new(proc_path);
        assert!(
            result.is_ok(),
            "Failed to create ProcessInformation: {:?}",
            result.err()
        );

        let proc_info = result.unwrap();
        assert_eq!(proc_info.pid, pid as usize);
        // cmdline might be empty for some processes, but status and stat should exist
        assert!(!proc_info.inner_status.is_empty());
        assert!(!proc_info.inner_stat.is_empty());
    }

    #[test]
    fn test_try_new_invalid_path() {
        // Test with non-existent PID
        let result = ProcessInformation::try_new(PathBuf::from("/proc/999999999"));
        assert!(result.is_err());
    }

    #[test]
    fn test_try_new_invalid_proc_structure() {
        // Test with path that doesn't have proper structure
        let result = ProcessInformation::try_new(PathBuf::from("/"));
        assert!(result.is_err());
    }

    #[test]
    fn test_name() {
        // Test getting process name
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.name();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_status_caching() {
        // Test that status() caches results
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();

        // First call should populate cache
        let status1 = proc_info.status();
        assert!(!status1.is_empty());

        // Second call should return cached result (same Rc pointer)
        let status2 = proc_info.status();
        assert!(Rc::ptr_eq(&status1, &status2));
    }

    #[test]
    fn test_stat_caching() {
        // Test that stat() caches results
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();

        // First call should populate cache
        let stat1 = proc_info.stat();
        assert!(!stat1.is_empty());

        // Second call should return cached result (same Rc pointer)
        let stat2 = proc_info.stat();
        assert!(Rc::ptr_eq(&stat1, &stat2));
    }

    #[test]
    fn test_stat_split_simple() {
        // Test stat_split with simple process name
        let stat = "1234 (bash) S 1 1234 1234 34816 1234 4194304";
        let result = ProcessInformation::stat_split(stat);

        assert_eq!(result[0], "1234");
        assert_eq!(result[1], "bash");
        assert_eq!(result[2], "S");
        assert_eq!(result[3], "1");
    }

    #[test]
    fn test_stat_split_with_spaces() {
        // Test stat_split with process name containing spaces
        let stat = "1234 (my process) S 1 1234 1234 34816";
        let result = ProcessInformation::stat_split(stat);

        assert_eq!(result[0], "1234");
        assert_eq!(result[1], "my process");
        assert_eq!(result[2], "S");
    }

    #[test]
    fn test_stat_split_with_parentheses() {
        // Test stat_split with process name containing parentheses
        let stat = "1234 (test(1)) S 1 1234 1234 34816";
        let result = ProcessInformation::stat_split(stat);

        assert_eq!(result[0], "1234");
        assert_eq!(result[1], "test(1)");
        assert_eq!(result[2], "S");
    }

    #[test]
    fn test_stat_split_nested_parentheses() {
        // Test stat_split with nested parentheses in process name
        // rfind(')') finds the LAST closing paren, so nested parens work correctly
        let stat = "1234 (name(with(nested))) S 1 1234";
        let result = ProcessInformation::stat_split(stat);

        assert_eq!(result[0], "1234");
        assert_eq!(result[1], "name(with(nested))");
        assert_eq!(result[2], "S");
    }

    #[test]
    fn test_stat_split_no_parentheses() {
        // Test stat_split when parentheses are missing (fallback to whitespace split)
        let stat = "1234 bash S 1 1234 1234";
        let result = ProcessInformation::stat_split(stat);

        assert_eq!(result[0], "1234");
        assert_eq!(result[1], "bash");
        assert_eq!(result[2], "S");
    }

    #[test]
    fn test_ppid_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.ppid();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_pgid_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.pgid();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_sid_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.sid();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_uid() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.uid();
        assert!(result.is_ok());
        let uid = result.unwrap();
        // UID can be any value including 0 for root
        let _ = uid;
    }

    #[test]
    fn test_euid() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.euid();
        assert!(result.is_ok());
        let euid = result.unwrap();
        // EUID is u32, always valid
        let _ = euid;
    }

    #[test]
    fn test_suid() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.suid();
        assert!(result.is_ok());
        let suid = result.unwrap();
        // SUID is u32, always valid
        let _ = suid;
    }

    #[test]
    fn test_gid() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.gid();
        assert!(result.is_ok());
        let gid = result.unwrap();
        // GID is u32, always valid
        let _ = gid;
    }

    #[test]
    fn test_egid() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.egid();
        assert!(result.is_ok());
        let egid = result.unwrap();
        // EGID is u32, always valid
        let _ = egid;
    }

    #[test]
    fn test_sgid() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.sgid();
        assert!(result.is_ok());
        let sgid = result.unwrap();
        // SGID is u32, always valid
        let _ = sgid;
    }

    #[test]
    fn test_tty() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.tty();
        // tty() always returns Unknown for now
        assert_eq!(result, Teletype::Unknown);
    }

    #[test]
    fn test_run_state_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.run_state();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_start_time_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.start_time();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_env_vars_not_implemented() {
        let pid = std::process::id();
        let proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.env_vars();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_namespaces() {
        let pid = std::process::id();
        let proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.namespaces();
        assert!(result.is_ok());

        let ns = result.unwrap();
        // namespaces() returns a new empty Namespace
        assert_eq!(ns, Namespace::new());
    }

    #[test]
    fn test_cgroups() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.cgroups();
        assert!(result.is_ok());

        let cgroups = result.unwrap();
        // cgroups() returns an empty vector for now
        assert!(cgroups.is_empty());
    }

    #[test]
    fn test_root_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.root();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_thread_ids_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.thread_ids();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_signals_pending_mask_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.signals_pending_mask();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_signals_blocked_mask_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.signals_blocked_mask();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_signals_ignored_mask_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.signals_ignored_mask();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_signals_caught_mask_not_implemented() {
        let pid = std::process::id();
        let mut proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let result = proc_info.signals_caught_mask();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn test_walk_process() {
        // Test that walk_process returns an empty iterator
        let mut iter = walk_process();
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_walk_threads() {
        // Test that walk_threads returns an empty iterator
        let mut iter = walk_threads();
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_process_information_clone() {
        // Test that ProcessInformation can be cloned
        let pid = std::process::id();
        let proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let cloned = proc_info.clone();

        assert_eq!(proc_info.pid, cloned.pid);
        assert_eq!(proc_info.cmdline, cloned.cmdline);
    }

    #[test]
    fn test_process_information_debug() {
        // Test that ProcessInformation implements Debug
        let pid = std::process::id();
        let proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        let debug_str = format!("{:?}", proc_info);
        assert!(debug_str.contains("ProcessInformation"));
    }

    #[test]
    fn test_cmdline_parsing() {
        // Test that cmdline is properly parsed (null bytes replaced with spaces)
        let pid = std::process::id();
        let proc_info =
            ProcessInformation::try_new(PathBuf::from(format!("/proc/{}", pid))).unwrap();
        // cmdline should not contain null bytes
        assert!(!proc_info.cmdline.contains('\0'));
    }
}
