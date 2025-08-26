// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use std::str::FromStr;

extern "C" {
    pub fn sd_booted() -> libc::c_int;
    pub fn sd_get_sessions(sessions: *mut *mut *mut libc::c_char) -> libc::c_int;
    pub fn sd_session_get_class(
        session: *const libc::c_char,
        class: *mut *mut libc::c_char,
    ) -> libc::c_int;
}

pub fn get_nusers_systemd() -> uucore::error::UResult<usize> {
    use std::ffi::CStr;
    use std::ptr;
    use uucore::error::USimpleError;
    use uucore::libc::*;

    // SAFETY: sd_booted to check if system is booted with systemd.
    unsafe {
        // systemd
        if sd_booted() > 0 {
            let mut sessions_list: *mut *mut c_char = ptr::null_mut();
            let mut num_user = 0;
            let sessions = sd_get_sessions(&mut sessions_list);

            if sessions > 0 {
                for i in 0..sessions {
                    let mut class: *mut c_char = ptr::null_mut();

                    if sd_session_get_class(
                        *sessions_list.add(i as usize) as *const c_char,
                        &mut class,
                    ) < 0
                    {
                        continue;
                    }
                    if CStr::from_ptr(class).to_str().unwrap().starts_with("user") {
                        num_user += 1;
                    }
                    free(class as *mut c_void);
                }
            }

            for i in 0..sessions {
                free(*sessions_list.add(i as usize) as *mut c_void);
            }
            free(sessions_list as *mut c_void);

            return Ok(num_user);
        }
    }
    Err(USimpleError::new(
        1,
        "could not retrieve number of logged users",
    ))
}

pub fn get_cpu_loads() -> Vec<uu_vmstat::CpuLoadRaw> {
    let mut cpu_loads = Vec::new();

    let file = std::fs::File::open(std::path::Path::new("/proc/stat")).unwrap(); // do not use `parse_proc_file` here because only one line is used
    let content = std::io::read_to_string(file).unwrap();

    for line in content.lines() {
        let tag = line.split_whitespace().next().unwrap();
        if tag != "cpu" && tag.starts_with("cpu") {
            let load = uu_vmstat::CpuLoadRaw::from_str(line.strip_prefix(tag).unwrap()).unwrap();
            cpu_loads.push(load);
        }
    }

    cpu_loads
}
