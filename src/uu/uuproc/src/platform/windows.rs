// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::common::{CgroupMembership, Namespace, RunState, Teletype};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

/// Process ID and its information (Windows)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessInformation {
    pub pid: usize,
    pub cmdline: String,
}

impl ProcessInformation {
    pub fn name(&mut self) -> Result<String, io::Error> {
        Ok(self.cmdline.clone())
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
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn euid(&mut self) -> Result<u32, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn gid(&mut self) -> Result<u32, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn egid(&mut self) -> Result<u32, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn suid(&mut self) -> Result<u32, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }

    pub fn sgid(&mut self) -> Result<u32, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
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

/// Iterate over all processes on Windows
pub fn walk_process() -> impl Iterator<Item = ProcessInformation> {
    std::iter::empty()
}

/// Iterate over all threads on Windows
pub fn walk_threads() -> impl Iterator<Item = ProcessInformation> {
    std::iter::empty()
}
