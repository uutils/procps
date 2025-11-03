// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::{CgroupMembership, Namespace, RunState, Teletype};
use crate::platform::helpers::{windows_string_to_rust, MISSING_DATA_ERROR};
use std::collections::HashMap;
use std::io;
use std::mem;
use std::path::PathBuf;
use winapi::shared::minwindef::FALSE;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};

/// Process ID and its information (Windows)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessInformation {
    pub pid: usize,
    pub cmdline: String,
    proc_entry: Option<Box<PROCESSENTRY32>>,
}

#[allow(dead_code)]
impl ProcessInformation {
    /// Create a new ProcessInformation from PROCESSENTRY32
    pub fn from_process_entry(entry: PROCESSENTRY32) -> Result<Self, io::Error> {
        let pid = entry.th32ProcessID as usize;
        let cmdline = windows_string_to_rust(&entry.szExeFile);

        Ok(Self {
            pid,
            cmdline,
            proc_entry: Some(Box::new(entry)),
        })
    }

    fn pid(&self) -> usize {
        self.pid
    }

    fn cmdline(&self) -> &str {
        &self.cmdline
    }

    fn name(&mut self) -> Result<String, io::Error> {
        Ok(self.cmdline.clone())
    }

    pub fn ppid(&mut self) -> Result<u64, io::Error> {
        self.proc_entry
            .as_ref()
            .map(|e| e.th32ParentProcessID as u64)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn pgid(&mut self) -> Result<u64, io::Error> {
        // Windows doesn't have process groups like Unix
        Ok(self.pid as u64)
    }

    pub fn sid(&mut self) -> Result<u64, io::Error> {
        // Windows doesn't have session IDs like Unix
        Ok(0)
    }

    pub fn uid(&mut self) -> Result<u32, io::Error> {
        // Windows doesn't have Unix-style UIDs
        Ok(0)
    }

    pub fn euid(&mut self) -> Result<u32, io::Error> {
        Ok(0)
    }

    pub fn gid(&mut self) -> Result<u32, io::Error> {
        // Windows doesn't have Unix-style GIDs
        Ok(0)
    }

    pub fn egid(&mut self) -> Result<u32, io::Error> {
        Ok(0)
    }

    pub fn suid(&mut self) -> Result<u32, io::Error> {
        Ok(0)
    }

    pub fn sgid(&mut self) -> Result<u32, io::Error> {
        Ok(0)
    }

    pub fn tty(&mut self) -> Teletype {
        Teletype::Unknown
    }

    pub fn run_state(&mut self) -> Result<RunState, io::Error> {
        // Windows processes are always running (no state info from PROCESSENTRY32)
        Ok(RunState::Running)
    }

    fn start_time(&mut self) -> Result<u64, io::Error> {
        Ok(0)
    }

    fn env_vars(&self) -> Result<HashMap<String, String>, io::Error> {
        Ok(HashMap::new())
    }

    fn namespaces(&self) -> Result<Namespace, io::Error> {
        Ok(Namespace::default())
    }

    fn cgroups(&mut self) -> Result<Vec<CgroupMembership>, io::Error> {
        Ok(Vec::new())
    }

    fn root(&mut self) -> Result<PathBuf, io::Error> {
        Ok(PathBuf::from("C:\\"))
    }

    fn thread_ids(&mut self) -> Result<Vec<usize>, io::Error> {
        self.proc_entry
            .as_ref()
            .map(|e| vec![e.th32ThreadID as usize])
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    fn signals_pending_mask(&mut self) -> Result<u64, io::Error> {
        Ok(0)
    }

    fn signals_blocked_mask(&mut self) -> Result<u64, io::Error> {
        Ok(0)
    }

    fn signals_ignored_mask(&mut self) -> Result<u64, io::Error> {
        Ok(0)
    }

    fn signals_caught_mask(&mut self) -> Result<u64, io::Error> {
        Ok(0)
    }
}

/// Iterate over all processes on Windows
pub fn walk_process() -> impl Iterator<Item = ProcessInformation> {
    get_all_processes().into_iter()
}

/// Iterate over all threads on Windows
pub fn walk_threads() -> impl Iterator<Item = ProcessInformation> {
    get_all_processes().into_iter()
}

/// Get all processes using CreateToolhelp32Snapshot
fn get_all_processes() -> Vec<ProcessInformation> {
    let mut processes = Vec::new();

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot as isize == -1 {
            return processes;
        }

        let mut entry: PROCESSENTRY32 = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut entry) == FALSE {
            return processes;
        }

        loop {
            if let Ok(proc_info) = ProcessInformation::from_process_entry(entry) {
                processes.push(proc_info);
            }

            if Process32Next(snapshot, &mut entry) == FALSE {
                break;
            }
        }
    }

    processes
}
