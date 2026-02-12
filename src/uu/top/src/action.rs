// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
pub(crate) fn renice(pid: u32, nice_value: i32) -> uucore::error::UResult<()> {
    use rustix::process::{setpriority_process, Pid};
    use uucore::error::USimpleError;

    let pid = Pid::from_raw(pid as i32);
    setpriority_process(pid, nice_value).map_err(|_| USimpleError::new(0, "Permission Denied"))
}

#[cfg(unix)]
pub(crate) fn kill_process(pid: u32, sig: usize) -> uucore::error::UResult<()> {
    use nix::sys::signal;
    use nix::sys::signal::Signal;
    use nix::unistd::Pid;
    use uucore::error::USimpleError;

    signal::kill(Pid::from_raw(pid as i32), Signal::try_from(sig as i32)?)
        .map_err(|_| USimpleError::new(0, "Permission Denied"))
}
