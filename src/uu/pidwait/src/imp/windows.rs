// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Reference: pidwait-any crate.
// Thanks to @oxalica's implementation.

use std::time::Duration;
use uu_pgrep::process::ProcessInformation;

use std::ffi::c_void;
use std::io::{Error, Result};
use std::ptr::NonNull;

use windows_sys::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Threading::{
    OpenProcess, WaitForMultipleObjects, INFINITE, PROCESS_SYNCHRONIZE,
};

struct HandleWrapper(NonNull<c_void>);
unsafe impl Send for HandleWrapper {}
impl Drop for HandleWrapper {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0.as_ptr());
        };
    }
}

pub fn wait(procs: &[ProcessInformation], timeout: Option<Duration>) -> Result<Option<()>> {
    let hprocess = unsafe {
        let mut result = Vec::with_capacity(procs.len());
        for proc in procs {
            let handle = OpenProcess(PROCESS_SYNCHRONIZE, 0, proc.pid as u32);
            result.push(HandleWrapper(
                NonNull::new(handle).ok_or_else(Error::last_os_error)?,
            ));
        }
        result
    };
    const _: [(); 1] = [(); (INFINITE == u32::MAX) as usize];
    let timeout = match timeout {
        Some(timeout) => timeout
            .as_millis()
            .try_into()
            .unwrap_or(INFINITE - 1)
            .min(INFINITE - 1),
        None => INFINITE,
    };
    let ret = unsafe {
        WaitForMultipleObjects(
            hprocess.len() as u32,
            hprocess
                .into_iter()
                .map(|proc| proc.0.as_ptr())
                .collect::<Vec<_>>()
                .as_ptr(),
            1,
            timeout,
        )
    };
    match ret {
        WAIT_OBJECT_0 => Ok(Some(())),
        WAIT_TIMEOUT => Ok(None),
        _ => Err(Error::last_os_error()),
    }
}
