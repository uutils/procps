// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Reference: pidwait-any crate.
// Thanks to @oxalica's implementation.

use std::io::{Error, ErrorKind};
use std::os::fd::OwnedFd;

use rustix::event::{poll, PollFd, PollFlags};
use rustix::io::Errno;
use rustix::process::{pidfd_open, Pid, PidfdFlags};
use std::io::Result;
use std::time::Duration;
use uu_pgrep::process::ProcessInformation;

pub fn wait(procs: &[ProcessInformation], timeout: Option<Duration>) -> Result<Option<()>> {
    let mut pidfds: Vec<OwnedFd> = Vec::with_capacity(procs.len());
    for proc in procs {
        let pid = Pid::from_raw(proc.pid as i32).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidInput,
                format!("Invalid PID: {}", proc.pid),
            )
        })?;
        let pidfd = pidfd_open(pid, PidfdFlags::empty())?;
        pidfds.push(pidfd);
    }
    let timespec = match timeout {
        Some(timeout) => Some(timeout.try_into().map_err(|_| Errno::INVAL)?),
        None => None,
    };
    let mut fds: Vec<PollFd> = Vec::with_capacity(pidfds.len());
    for pidfd in &pidfds {
        fds.push(PollFd::new(pidfd, PollFlags::IN));
    }
    let ret = poll(&mut fds, timespec.as_ref())?;
    if ret == 0 {
        return Ok(None);
    }
    debug_assert!(fds[0].revents().contains(PollFlags::IN));
    Ok(Some(()))
}
