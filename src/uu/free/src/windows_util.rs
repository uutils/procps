use std::ffi::c_void;

use windows::{
    core::Result,
    Wdk::System::SystemInformation::{NtQuerySystemInformation, SYSTEM_INFORMATION_CLASS},
    Win32::Foundation::{STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS, UNICODE_STRING},
};

#[repr(C)]
#[derive(Default, Debug)]
#[allow(non_snake_case)]
struct SYSTEM_PAGEFILE_INFORMATION {
    NextEntryOffset: u32,
    TotalSize: u32,
    TotalInUse: u32,
    PeakUsage: u32,
    PageFileName: UNICODE_STRING,
}

/// Get the usage and total size of all page files.
pub(crate) fn get_pagefile_usage() -> Result<(u32, u32)> {
    let mut buf: Vec<u8> = Vec::new();

    let mut return_len: u32 = 0;

    loop {
        let status = unsafe {
            NtQuerySystemInformation(
                // SystemPageFileInformation
                SYSTEM_INFORMATION_CLASS(0x12),
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u32,
                &mut return_len,
            )
        };

        debug_assert!(buf.len() <= return_len as usize);

        if status == STATUS_INFO_LENGTH_MISMATCH {
            debug_assert!(return_len > buf.len() as u32);
            buf.resize(return_len as usize, 0);
            continue;
        } else if status == STATUS_SUCCESS {
            debug_assert!(return_len == buf.len() as u32);
            break;
        } else {
            return Err(status.into());
        }
    }

    let mut usage = 0;
    let mut total = 0;

    if return_len > 0 {
        let ptr = buf.as_ptr();
        let mut offset = 0;
        loop {
            let ptr_offset =
                unsafe { ptr.byte_offset(offset) } as *const SYSTEM_PAGEFILE_INFORMATION;
            let record = unsafe { std::ptr::read(ptr_offset) };

            usage += record.TotalInUse;
            total += record.TotalSize;

            offset = record.NextEntryOffset as isize;

            if offset == 0 {
                break;
            }
        }
    }

    Ok((usage, total))
}
