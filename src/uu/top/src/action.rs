// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
pub(crate) fn renice(pid: u32, nice_value: i32) -> uucore::error::UResult<()> {
    use libc::{setpriority, PRIO_PROCESS};
    use uucore::error::USimpleError;
    let result = unsafe { setpriority(PRIO_PROCESS, pid, nice_value) };
    if result == -1 {
        Err(USimpleError::new(0, "Permission Denied"))
    } else {
        Ok(())
    }
}
