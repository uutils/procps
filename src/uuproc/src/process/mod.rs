use crate::ns::Namespace;
use crate::process::runstate::RunState;
use crate::{cgroups::CgroupMembership, teletype::Teletype};

use walkdir::{DirEntry, WalkDir};

use std::{collections::HashMap, fs, hash::Hash, io, path::PathBuf, sync::OnceLock};

mod runstate;
mod walk;

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
    /// If the files in path cannot be parsed into [ProcessInformation], it almost caused by wrong
    /// filesystem structure.
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
        let pid = rustix::process::getpid().as_raw_pid();

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
        fs::read_link(format!("/proc/{}/root", self.pid))
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use rustix::process::getpid;
    #[cfg(target_os = "linux")]
    use std::collections::HashSet;

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
        let main_tid = rustix::thread::gettid().as_raw_nonzero().get() as u64;
        std::thread::spawn(move || {
            let mut pid_entry = ProcessInformation::current_process_info().unwrap();
            let thread_ids = pid_entry.thread_ids();

            assert!(thread_ids.contains(&(main_tid as usize)));

            let new_thread_tid = rustix::thread::gettid().as_raw_nonzero().get() as u64;
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
            rustix::process::getppid()
                .map(|pid| pid.as_raw_nonzero().get() as u64)
                .unwrap_or(0)
        );
        assert_eq!(
            pid_entry.pgid().unwrap(),
            rustix::process::getpgid(None)
                .ok()
                .map(|pid| pid.as_raw_nonzero().get() as u64)
                .unwrap_or(0)
        );
        assert_eq!(
            pid_entry.sid().unwrap(),
            rustix::process::getsid(None)
                .ok()
                .map(|pid| pid.as_raw_nonzero().get() as u64)
                .unwrap_or(0)
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_uid_gid() {
        let mut pid_entry = ProcessInformation::current_process_info().unwrap();
        assert_eq!(pid_entry.uid().unwrap(), rustix::process::getuid().as_raw());
        assert_eq!(
            pid_entry.euid().unwrap(),
            rustix::process::geteuid().as_raw()
        );
        assert_eq!(pid_entry.gid().unwrap(), rustix::process::getgid().as_raw());
        assert_eq!(
            pid_entry.egid().unwrap(),
            rustix::process::getegid().as_raw()
        );
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
