// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// Reference: pidwait-any crate.
// Thanks to @oxalica's implementation.

// FIXME: Test this implementation

use rustix::event::kqueue::{kevent, kqueue, Event, EventFilter, EventFlags, ProcessEvents};
use rustix::process::Pid;
use std::io::{Error, ErrorKind, Result};
use std::mem::MaybeUninit;
use std::time::Duration;
use uu_pgrep::process::ProcessInformation;

pub fn wait(procs: &[ProcessInformation], timeout: Option<Duration>) -> Result<Option<()>> {
    let mut events = Vec::with_capacity(procs.len());
    let kqueue = kqueue()?;
    for proc in procs {
        let pid = Pid::from_raw(proc.pid as i32).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidInput,
                format!("Invalid PID: {}", proc.pid),
            )
        })?;
        let event = Event::new(
            EventFilter::Proc {
                pid,
                flags: ProcessEvents::EXIT,
            },
            EventFlags::ADD,
            std::ptr::null_mut(),
        );
        events.push(event);
    }
    let ret = unsafe { kevent::<_, &mut [Event; 0]>(&kqueue, &events, &mut [], None)? };
    debug_assert_eq!(ret, 0);
    let mut buf = [MaybeUninit::uninit()];
    let (events, _rest_buf) = unsafe { kevent(&kqueue, &[], &mut buf, timeout)? };
    if events.is_empty() {
        return Ok(None);
    };
    debug_assert!(matches!(
        events[0].filter(),
        EventFilter::Proc { flags, .. }
        if flags.contains(ProcessEvents::EXIT)
    ));
    Ok(Some(()))
}
