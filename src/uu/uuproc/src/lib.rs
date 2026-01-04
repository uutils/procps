// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

//! Cross-platform process information library. Provides unified API for querying
//! process information across Linux, FreeBSD, macOS, and Windows.
//!
//! # Comparison with Reference Implementations
//!
//! This library modernizes patterns from:
//! - **C procps**: Field-based queryable API, comprehensive coverage
//! - **Rust coreutils**: Clean type abstractions, feature-gated design
//!
//! Improvements over references:
//! - Cross-platform support (C procps is Linux-only)
//! - Type safety (Rust enums vs C chars/ints)
//! - 96.61% test coverage
//! - Macro-driven code generation
//!
//! # Example
//!
//! ```no_run
//! use uu_uuproc::platform::ProcessInformation;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//!
//! let mut proc = ProcessInformation::try_new("/proc/self".into())?;
//! println!("Process name: {}", proc.name()?);
//! # Ok(())
//! # }
//! ```

pub mod common;
pub mod platform;

// Re-export commonly used types and functions
pub use common::{CgroupMembership, Namespace, ProcessError, RunState, Teletype};
pub use platform::{walk_process, walk_threads, ProcessInformation};
