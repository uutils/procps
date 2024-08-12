// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uu_pgrep::process::ProcessInformation;

// Dirty, but it works.
// TODO: Use better implementation instead
#[cfg(target_os = "linux")]
pub(crate) fn waiting(procs: &[ProcessInformation]) {
    let mut list = procs.to_vec();

    loop {
        for proc in &list.clone() {
            // Check is running
            if !is_running(proc.pid) {
                list.retain(|it| it.pid != proc.pid)
            }
        }

        if list.is_empty() {
            return;
        }
    }
}
#[cfg(target_os = "linux")]
fn is_running(pid: usize) -> bool {
    use std::{path::PathBuf, str::FromStr};
    use uu_pgrep::process::ProcessInformation;
    use uu_pgrep::process::RunState;

    let proc = PathBuf::from_str(&format!("/proc/{}", pid)).unwrap();

    if !proc.exists() {
        return false;
    }

    match ProcessInformation::try_new(proc) {
        Ok(mut proc) => proc.run_state().unwrap() != RunState::Stopped,
        Err(_) => false,
    }
}

// Just for passing compile on other system.
#[cfg(not(target_os = "linux"))]
pub(crate) fn waiting(procs: &[ProcessInformation]) {}
