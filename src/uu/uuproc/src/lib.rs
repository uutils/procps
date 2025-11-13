// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

//! Cross-platform process information abstraction
//!
//! This crate provides a unified interface for accessing process information
//! across different platforms (Linux, FreeBSD, macOS, Windows).
//!
//! # Example
//!
//! ```ignore
//! use uu_uuproc::walk_process;
//!
//! for process in walk_process() {
//!     println!("PID: {}, Command: {}", process.pid, process.cmdline);
//! }
//! ```

pub mod common;
pub mod platform;

// Re-export commonly used types and functions
pub use common::{CgroupMembership, Namespace, RunState, Teletype};
pub use platform::{walk_process, walk_threads, ProcessInformation};
