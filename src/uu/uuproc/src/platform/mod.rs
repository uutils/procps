// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "windows"
)))]
mod fallback;
#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "windows"
)))]
pub use fallback::{walk_process, walk_threads, ProcessInformation};
#[cfg(target_os = "freebsd")]
pub use freebsd::{walk_process, walk_threads, ProcessInformation};
#[cfg(target_os = "linux")]
pub use linux::{walk_process, walk_threads, ProcessInformation};
#[cfg(target_os = "macos")]
pub use macos::{walk_process, walk_threads, ProcessInformation};
#[cfg(target_os = "windows")]
pub use windows::{walk_process, walk_threads, ProcessInformation};
