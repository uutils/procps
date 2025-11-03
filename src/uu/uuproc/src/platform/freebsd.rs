// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::{CgroupMembership, Namespace, RunState, Teletype};
use crate::platform::helpers::{c_string_to_rust, MISSING_DATA_ERROR};
use libc::{c_int, c_void, sysctl};
use std::collections::HashMap;
use std::io;
use std::mem;
use std::path::PathBuf;

// FreeBSD sysctl constants
const CTL_KERN: c_int = 1;
const KERN_PROC: c_int = 14;
const KERN_PROC_ALL: c_int = 0;

// kinfo_proc structure (simplified for basic fields)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KInfoProc {
    pub ki_structsize: c_int,
    pub ki_layout: c_int,
    pub ki_pid: i32,
    pub ki_ppid: i32,
    pub ki_pgid: i32,
    pub ki_sid: i32,
    pub ki_ruid: u32,
    pub ki_uid: u32,
    pub ki_svuid: u32,
    pub ki_rgid: u32,
    pub ki_groups: [u32; 16],
    pub ki_ngroups: i16,
    pub ki_gid: u32,
    pub ki_svgid: u32,
    pub ki_tdev: u32,
    pub ki_siglist: u64,
    pub ki_sigmask: u64,
    pub ki_sigignore: u64,
    pub ki_sigcatch: u64,
    pub ki_login: [u8; 17],
    pub ki_lockflags: u8,
    pub ki_state: u8,
    pub ki_nice: i8,
    pub ki_comlen: u8,
    pub ki_comm: [u8; 19],
    pub ki_name: [u8; 19],
    pub ki_onprio: u8,
    pub ki_lastcpu: u8,
    pub ki_tracer: i32,
    pub ki_flag: i32,
    pub ki_flag2: i32,
    pub ki_fibnum: i32,
    pub ki_cr_flags: u32,
    pub ki_jid: i32,
    pub ki_numthreads: i32,
    pub ki_tid: i32,
    pub ki_pri: [u8; 4],
    pub ki_rusage: [u64; 16],
    pub ki_rusage_ch: [u64; 16],
    pub ki_pcb: *mut c_void,
    pub ki_kstack: *mut c_void,
    pub ki_udata: *mut c_void,
    pub ki_tdaddr: *mut c_void,
    pub ki_spareptrs: [*mut c_void; 2],
    pub ki_sparelongs: [i64; 2],
    pub ki_sflag: i32,
    pub ki_tdflags: i32,
}

/// Process ID and its information (FreeBSD)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessInformation {
    pub pid: usize,
    pub cmdline: String,
    kinfo: Option<Box<KInfoProc>>,
}

#[allow(dead_code)]
impl ProcessInformation {
    /// Create a new ProcessInformation from a kinfo_proc structure
    pub fn from_kinfo(kinfo: KInfoProc) -> Result<Self, io::Error> {
        let pid = kinfo.ki_pid as usize;
        let cmdline = c_string_to_rust(&kinfo.ki_comm);

        Ok(Self {
            pid,
            cmdline,
            kinfo: Some(Box::new(kinfo)),
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
        self.kinfo
            .as_ref()
            .map(|k| k.ki_ppid as u64)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn pgid(&mut self) -> Result<u64, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_pgid as u64)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn sid(&mut self) -> Result<u64, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_sid as u64)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn uid(&mut self) -> Result<u32, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_uid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn euid(&mut self) -> Result<u32, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_uid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn gid(&mut self) -> Result<u32, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_gid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn egid(&mut self) -> Result<u32, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_gid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn suid(&mut self) -> Result<u32, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_svuid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn sgid(&mut self) -> Result<u32, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_svgid)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))
    }

    pub fn tty(&mut self) -> Teletype {
        self.kinfo
            .as_ref()
            .map(|k| Teletype::from(k.ki_tdev as u64))
            .unwrap_or(Teletype::Unknown)
    }

    pub fn run_state(&mut self) -> Result<RunState, io::Error> {
        let state = self
            .kinfo
            .as_ref()
            .map(|k| k.ki_state)
            .ok_or_else(|| io::Error::other(MISSING_DATA_ERROR))?;

        Ok(match state {
            1 => RunState::Running,
            2 => RunState::Running,
            3 => RunState::Sleeping,
            4 => RunState::Stopped,
            5 => RunState::Zombie,
            6 => RunState::Waiting,
            7 => RunState::Locked,
            _ => RunState::Unknown,
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
        self.kinfo
            .as_ref()
            .map(|k| k.ki_siglist)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No kinfo available"))
    }

    fn signals_blocked_mask(&mut self) -> Result<u64, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_sigmask)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No kinfo available"))
    }

    fn signals_ignored_mask(&mut self) -> Result<u64, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_sigignore)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No kinfo available"))
    }

    fn signals_caught_mask(&mut self) -> Result<u64, io::Error> {
        self.kinfo
            .as_ref()
            .map(|k| k.ki_sigcatch)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No kinfo available"))
    }
}

/// Iterate over all processes on FreeBSD
pub fn walk_process() -> impl Iterator<Item = ProcessInformation> {
    get_all_processes().into_iter()
}

/// Iterate over all threads on FreeBSD
pub fn walk_threads() -> impl Iterator<Item = ProcessInformation> {
    get_all_processes().into_iter()
}

/// Get all processes using sysctl
fn get_all_processes() -> Vec<ProcessInformation> {
    let mut processes = Vec::new();

    let mib = [CTL_KERN, KERN_PROC, KERN_PROC_ALL, 0];
    let mut size: usize = 0;

    // First call to get the size
    unsafe {
        if sysctl(
            mib.as_ptr() as *mut c_int,
            4,
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        ) == -1
        {
            return processes;
        }
    }

    if size == 0 {
        return processes;
    }

    // Allocate buffer
    let mut buf = vec![0u8; size];

    // Second call to get the data
    unsafe {
        if sysctl(
            mib.as_ptr() as *mut c_int,
            4,
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        ) == -1
        {
            return processes;
        }
    }

    // Parse the buffer into kinfo_proc structures
    let count = size / mem::size_of::<KInfoProc>();
    let kinfo_ptr = buf.as_ptr() as *const KInfoProc;

    for i in 0..count {
        unsafe {
            let kinfo = *kinfo_ptr.add(i);
            if let Ok(proc_info) = ProcessInformation::from_kinfo(kinfo) {
                processes.push(proc_info);
            }
        }
    }

    processes
}
