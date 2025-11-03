// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

/// Helper macro to extract a field from platform-specific data
/// Returns an error if the data is not available
#[macro_export]
macro_rules! extract_field {
    ($data:expr, $field:expr, $error_msg:expr) => {
        $data
            .as_ref()
            .map(|d| $field(d))
            .ok_or_else(|| io::Error::other($error_msg))
    };
}

/// Helper macro to extract a field with a default value
#[macro_export]
macro_rules! extract_field_or_default {
    ($data:expr, $field:expr, $default:expr) => {
        $data.as_ref().map(|d| $field(d)).unwrap_or($default)
    };
}

/// Helper function to convert C string to Rust String
#[cfg(any(target_os = "freebsd", target_os = "macos"))]
pub fn c_string_to_rust(c_str: &[u8]) -> String {
    use std::ffi::CStr;
    unsafe {
        CStr::from_ptr(c_str.as_ptr() as *const i8)
            .to_string_lossy()
            .to_string()
    }
}

/// Helper function to convert Windows string to Rust String
#[cfg(target_os = "windows")]
pub fn windows_string_to_rust(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .to_string()
}

/// Common error message for missing platform data
pub const MISSING_DATA_ERROR: &str = "Platform data not available";
