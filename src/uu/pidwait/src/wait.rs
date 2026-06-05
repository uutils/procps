// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use uu_pgrep::process::ProcessInformation;

#[cfg(target_os = "linux")]
pub(crate) fn wait(procs: &[ProcessInformation]) {
    use std::os::fd::OwnedFd;

    use rustix::event::{poll, PollFd, PollFlags};
    use rustix::process::{pidfd_open, Pid, PidfdFlags};

    let mut pidfds: Vec<OwnedFd> = procs
        .iter()
        .filter_map(|proc| {
            let pid = Pid::from_raw(proc.pid as i32)?;
            pidfd_open(pid, PidfdFlags::empty()).ok()
        })
        .collect();

    while !pidfds.is_empty() {
        let to_remove = {
            let mut fds: Vec<PollFd> = pidfds
                .iter()
                .map(|fd| PollFd::new(fd, PollFlags::IN))
                .collect();

            if poll(&mut fds, None).is_err() {
                break;
            }

            fds.iter()
                .enumerate()
                .filter(|(_, pfd)| pfd.revents().contains(PollFlags::IN))
                .map(|(i, _)| i)
                .rev()
                .collect::<Vec<_>>()
        };

        for i in to_remove {
            pidfds.remove(i);
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn wait(_procs: &[ProcessInformation]) {}

#[cfg(test)]
mod tests {

    #[cfg(target_os = "linux")]
    #[test]
    fn test_wait_single_process() {
        use super::*;
        use std::process::Command;
        use std::time::Instant;

        // NOTE: Manually tested with sleep 0.5, sleep 1, and sleep 2. Using 1s here to keep total
        // test time reasonable; 2s would also pass.
        let mut child = Command::new("sleep").arg("1").spawn().unwrap();
        let pid = child.id() as usize;

        let info = ProcessInformation::from_pid(pid).unwrap();
        let start = Instant::now();
        wait(&[info]);
        let elapsed = start.elapsed();

        assert!(
            elapsed >= std::time::Duration::from_millis(900),
            "wait returned too early: {elapsed:?}"
        );
        assert!(
            elapsed < std::time::Duration::from_secs(3),
            "wait took too long: {elapsed:?}"
        );

        let _ = child.wait();
    }
}
