// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod windows;

pub mod fallback;

#[cfg(target_os = "linux")]
pub use linux::{get_cpu_loads, get_memory, get_nusers_systemd};
#[cfg(target_os = "windows")]
pub use windows::get_cpu_loads;

#[allow(unused)]
pub use fallback::*;
