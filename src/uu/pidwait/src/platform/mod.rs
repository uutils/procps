#[cfg(target_os = "linux")]
pub use self::linux::wait;

#[cfg(windows)]
pub use self::windows::wait;

#[cfg(any(
    target_os = "freebsd",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
pub use self::bsd::wait;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(windows)]
mod windows;

#[cfg(any(
    target_os = "freebsd",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
mod bsd;