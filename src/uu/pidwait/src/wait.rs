// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::io::Result;
use std::time::Duration;
use uu_pgrep::process::ProcessInformation;

// Reference: pidwait-any crate.
// Thanks to @oxalica's implementation.

#[cfg(target_os = "linux")]
#[path = "./imp/linux.rs"]
mod imp;

#[cfg(windows)]
#[path = "./imp/windows.rs"]
mod imp;

#[cfg(any(
    target_os = "freebsd",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
#[path = "./imp/bsd.rs"]
mod imp;

pub(crate) fn wait(procs: &[ProcessInformation], timeout: Option<Duration>) -> Result<Option<()>> {
    if procs.len() > 0 {
        imp::wait(procs, timeout)
    } else {
        Ok(None)
    }
}

// Dirty, but it works.
// TODO: Use better implementation instead
// #[cfg(target_os = "linux")]
// pub(crate) fn wait(procs: &[ProcessInformation]) {
//     use std::{thread::sleep, time::Duration};

//     let mut list = procs.to_vec();

//     loop {
//         for proc in &list.clone() {
//             // Check is running
//             if !is_running(proc.pid) {
//                 list.retain(|it| it.pid != proc.pid);
//             }
//         }

//         if list.is_empty() {
//             return;
//         }

//         sleep(Duration::from_millis(50));
//     }
// }
// #[cfg(target_os = "linux")]
// fn is_running(pid: usize) -> bool {
//     use std::{path::PathBuf, str::FromStr};
//     use uu_pgrep::process::RunState;

//     let proc = PathBuf::from_str(&format!("/proc/{}", pid)).unwrap();

//     if !proc.exists() {
//         return false;
//     }

//     match ProcessInformation::try_new(proc) {
//         Ok(mut proc) => proc
//             .run_state()
//             .map(|it| it != RunState::Stopped)
//             .unwrap_or(false),
//         Err(_) => false,
//     }
// }

// // Just for passing compile on other system.
// #[cfg(not(target_os = "linux"))]
// pub(crate) fn wait(_procs: &[ProcessInformation]) {}
