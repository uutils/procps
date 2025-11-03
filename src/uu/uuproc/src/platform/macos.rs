// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::{CgroupMembership, Namespace, RunState, Teletype};
use crate::platform::helpers::{c_string_to_rust, MISSING_DATA_ERROR};
use libc::{c_int, c_void, proc_listpids, proc_pidinfo};
use std::collections::HashMap;
use std::io;
use std::mem;
use std::path::PathBuf;

// macOS libproc constants
const PROC_ALL_PIDS: u32 = 1;
const PROC_PIDTBSDINFO: c_int = 3;

// proc_bsdinfo structure from libproc
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcBsdInfo {
    pub pbi_flags: u32,
    pub pbi_status: u32,
    pub pbi_xstatus: u32,
    pub pbi_pid: u32,
    pub pbi_ppid: u32,
    pub pbi_uid: u32,
    pub pbi_gid: u32,
    pub pbi_ruid: u32,
    pub pbi_rgid: u32,
    pub pbi_svuid: u32,
    pub pbi_svgid: u32,
    pub rfu_1: u32,
    pub pbi_comm: [u8; 16],
    pub pbi_name: [u8; 32],
    pub pbi_nfiles: u32,
    pub pbi_pgid: u32,
    pub pbi_pjobc: u32,
    pub e_tdev: u32,
    pub e_tpgid: u32,
    pub pbi_psflags: u32,
    pub pbi_sid: u32,
    pub pbi_tsessionid: u32,
    pub pbi_cpuid: u32,
    pub pbi_csflags: u32,
}

/// Process ID and its information (macOS)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessInformation {
    pub pid: usize,
    pub cmdline: String,
    bsd_info: Option<Box<ProcBsdInfo>>,
}

#[allow(dead_code)]
impl ProcessInformation {
    /// Create a new ProcessInformation from proc_bsdinfo
    pub fn from_bsd_info(pid: u32, bsd_info: ProcBsdInfo) -> Result<Self, io::Error> {
        let cmdline = c_string_to_rust(&bsd_info.pbi_comm);

        Ok(Self {
            pid: pid as usize,
            cmdline,
            bsd_info: Some(Box::new(bsd_info)),
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
        self.bsd_info
            .as_ref()
            .map(|b| b.pbi_ppid as u64)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn pgid(&mut self) -> Result<u64, io::Error> {
        self.bsd_info
            .as_ref()
            .map(|b| b.pbi_pgid as u64)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn sid(&mut self) -> Result<u64, io::Error> {
        self.bsd_info
            .as_ref()
            .map(|b| b.pbi_sid as u64)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn uid(&mut self) -> Result<u32, io::Error> {
        self.bsd_info
            .as_ref()
            .map(|b| b.pbi_uid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn euid(&mut self) -> Result<u32, io::Error> {
        self.bsd_info
            .as_ref()
            .map(|b| b.pbi_uid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn gid(&mut self) -> Result<u32, io::Error> {
        self.bsd_info
            .as_ref()
            .map(|b| b.pbi_gid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn egid(&mut self) -> Result<u32, io::Error> {
        self.bsd_info
            .as_ref()
            .map(|b| b.pbi_gid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn suid(&mut self) -> Result<u32, io::Error> {
        self.bsd_info
            .as_ref()
            .map(|b| b.pbi_svuid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn sgid(&mut self) -> Result<u32, io::Error> {
        self.bsd_info
            .as_ref()
            .map(|b| b.pbi_svgid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn tty(&mut self) -> Teletype {
        self.bsd_info
            .as_ref()
            .map(|b| {
                let dev = b.e_tdev as u64;
                if dev == 0 {
                    Teletype::Unknown
                } else {
                    Teletype::Tty(dev)
                }
            })
            .unwrap_or(Teletype::Unknown)
    }

    pub fn run_state(&mut self) -> Result<RunState, io::Error> {
        let status = self
            .bsd_info
            .as_ref()
            .map(|b| b.pbi_status)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))?;

        // macOS process states
        Ok(match status {
            1 => RunState::Idle,                // SIDL
            2 => RunState::Running,             // SRUN
            3 => RunState::Sleeping,            // SSLEEP
            4 => RunState::Stopped,             // SSTOP
            5 => RunState::Zombie,              // SZOMB
            6 => RunState::UninterruptibleWait, // SWAIT
            _ => RunState::Idle,
        })
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
        Ok(PathBuf::from("/"))
    }

    fn thread_ids(&mut self) -> Result<Vec<usize>, io::Error> {
        Ok(Vec::new())
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

/// Iterate over all processes on macOS
pub fn walk_process() -> impl Iterator<Item = ProcessInformation> {
    get_all_processes().into_iter()
}

/// Iterate over all threads on macOS
pub fn walk_threads() -> impl Iterator<Item = ProcessInformation> {
    get_all_processes().into_iter()
}

/// Get all processes using libproc
fn get_all_processes() -> Vec<ProcessInformation> {
    let mut processes = Vec::new();

    // Allocate buffer for PIDs (max 10000 processes)
    let max_pids = 10000;
    let mut pids = vec![0u32; max_pids];

    // Get list of all PIDs
    let num_pids = unsafe {
        proc_listpids(
            PROC_ALL_PIDS,
            0,
            pids.as_mut_ptr() as *mut c_void,
            (max_pids * mem::size_of::<u32>()) as i32,
        )
    };

    eprintln!("[DEBUG] proc_listpids returned: {}", num_pids);

    if num_pids <= 0 {
        eprintln!("[DEBUG] proc_listpids returned <= 0, returning empty process list");
        return processes;
    }

    let count = (num_pids as usize) / mem::size_of::<u32>();

    // Get info for each PID
    for pid in pids.iter().take(count) {
        if *pid == 0 {
            continue;
        }

        let mut bsd_info: ProcBsdInfo = unsafe { mem::zeroed() };
        let info_size = unsafe {
            proc_pidinfo(
                *pid as c_int,
                PROC_PIDTBSDINFO,
                0,
                &mut bsd_info as *mut _ as *mut c_void,
                mem::size_of::<ProcBsdInfo>() as i32,
            )
        };

        if info_size > 0 {
            if let Ok(proc_info) = ProcessInformation::from_bsd_info(*pid, bsd_info) {
                processes.push(proc_info);
            }
        }
    }

    processes
}
