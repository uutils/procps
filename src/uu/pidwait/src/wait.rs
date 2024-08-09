// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uu_pgrep::process::ProcessInformation;
use uu_pgrep::process::RunState;

// Dirty, but it works.
// TODO: Use better implementation instead
#[cfg(target_os = "linux")]
pub(crate) fn waiting(procs: &[ProcessInformation]) {
    use std::{path::PathBuf, str::FromStr};

    let mut list = procs.to_vec();

    loop {
        for proc in procs.iter().cloned() {
            let proc_path = PathBuf::from_str(&format!("/proc/{}", proc.pid)).unwrap();
            if !proc_path.exists() {
                list.retain(|it| it.pid != proc.pid);
            }

            if let Ok(mut proc) = ProcessInformation::try_new(proc_path) {
                if proc.run_state().unwrap() == RunState::Stopped {
                    list.retain(|it| it.pid != proc.pid);
                }
            } else {
                list.retain(|it| it.pid != proc.pid);
            };
        }

        if list.is_empty() {
            return;
        }
    }
}
