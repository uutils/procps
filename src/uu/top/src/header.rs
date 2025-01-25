// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::picker::{sysinfo, systemstat};
use bytesize::ByteSize;
use systemstat::Platform;

pub(crate) fn header(scale_summary_mem: Option<&String>) -> String {
    format!(
        "top - {time} {uptime}, {user}, {load_average}\n\
        {task}\n\
        {cpu}\n\
        {memory}",
        time = chrono::Local::now().format("%H:%M:%S"),
        uptime = uptime(),
        user = user(),
        load_average = load_average(),
        task = task(),
        cpu = cpu(),
        memory = memory(scale_summary_mem),
    )
}

#[cfg(not(target_os = "linux"))]
fn todo() -> String {
    "TODO".into()
}

fn format_memory(memory_b: u64, unit: u64) -> f64 {
    ByteSize::b(memory_b).0 as f64 / unit as f64
}

fn uptime() -> String {
    let binding = systemstat().read().unwrap();

    let up_seconds = binding.uptime().unwrap().as_secs();
    let up_minutes = (up_seconds % (60 * 60)) / 60;
    let up_hours = (up_seconds % (24 * 60 * 60)) / (60 * 60);
    let up_days = up_seconds / (24 * 60 * 60);

    let mut res = String::from("up ");

    if up_days > 0 {
        res.push_str(&format!(
            "{} day{}, ",
            up_days,
            if up_days > 1 { "s" } else { "" }
        ));
    }
    if up_hours > 0 {
        res.push_str(&format!("{}:{:0>2}", up_hours, up_minutes));
    } else {
        res.push_str(&format!("{} min", up_minutes));
    }

    res
}

#[inline]
fn format_user(user: u64) -> String {
    match user {
        0 => "0 user".to_string(),
        1 => "1 user".to_string(),
        _ => format!("{} users", user),
    }
}

#[cfg(target_os = "windows")]
fn user() -> String {
    use windows::{core::*, Win32::System::RemoteDesktop::*};

    let mut num_user = 0;

    unsafe {
        let mut session_info_ptr = std::ptr::null_mut();
        let mut session_count = 0;

        WTSEnumerateSessionsW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            0,
            1,
            &mut session_info_ptr,
            &mut session_count,
        )
        .unwrap();

        let sessions = std::slice::from_raw_parts(session_info_ptr, session_count as usize);

        for session in sessions {
            let mut buffer = PWSTR::null();
            let mut bytes_returned = 0;

            WTSQuerySessionInformationW(
                Some(WTS_CURRENT_SERVER_HANDLE),
                session.SessionId,
                WTS_INFO_CLASS(5),
                &mut buffer,
                &mut bytes_returned,
            )
            .unwrap();

            let username = PWSTR(buffer.0).to_string().unwrap_or_default();
            if !username.is_empty() {
                num_user += 1;
            }

            WTSFreeMemory(buffer.0 as _);
        }

        WTSFreeMemory(session_info_ptr as _);
    }

    format_user(num_user)
}

#[cfg(unix)]
// see: https://gitlab.com/procps-ng/procps/-/blob/4740a0efa79cade867cfc7b32955fe0f75bf5173/library/uptime.c#L63-L115
fn user() -> String {
    use uucore::utmpx::Utmpx;

    #[cfg(target_os = "linux")]
    unsafe {
        use libc::free;
        use libsystemd_sys::daemon::sd_booted;
        use libsystemd_sys::login::{sd_get_sessions, sd_session_get_class};
        use std::ffi::{c_char, c_void, CStr};
        use std::ptr;
        // systemd
        if sd_booted() > 0 {
            let mut sessions_list: *mut *mut c_char = ptr::null_mut();
            let mut num_user = 0;
            let sessions = sd_get_sessions(&mut sessions_list); // rust-systemd does not implement this

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

            return format_user(num_user);
        }
    }

    // utmpx
    let mut num_user = 0;
    Utmpx::iter_all_records().for_each(|ut| {
        if ut.record_type() == 7 && !ut.user().is_empty() {
            num_user += 1;
        }
    });
    format_user(num_user)
}

#[cfg(not(target_os = "windows"))]
fn load_average() -> String {
    let binding = systemstat().read().unwrap();

    let load_average = binding.load_average().unwrap();
    format!(
        "load average: {:.2}, {:.2}, {:.2}",
        load_average.one, load_average.five, load_average.fifteen
    )
}

#[cfg(target_os = "windows")]
fn load_average() -> String {
    todo()
}

fn task() -> String {
    let binding = sysinfo().read().unwrap();

    let process = binding.processes();
    let mut running_process = 0;
    let mut sleeping_process = 0;
    let mut stopped_process = 0;
    let mut zombie_process = 0;

    for (_, process) in process.iter() {
        match process.status() {
            sysinfo::ProcessStatus::Run => running_process += 1,
            sysinfo::ProcessStatus::Sleep => sleeping_process += 1,
            sysinfo::ProcessStatus::Stop => stopped_process += 1,
            sysinfo::ProcessStatus::Zombie => zombie_process += 1,
            _ => {}
        };
    }

    format!(
        "Tasks: {} total, {} running, {} sleeping, {} stopped, {} zombie",
        process.len(),
        running_process,
        sleeping_process,
        stopped_process,
        zombie_process,
    )
}

#[cfg(target_os = "linux")]
fn cpu() -> String {
    let file = std::fs::File::open(std::path::Path::new("/proc/stat")).unwrap();
    let content = std::io::read_to_string(file).unwrap();
    let load = content
        .lines()
        .next()
        .unwrap()
        .strip_prefix("cpu")
        .unwrap()
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let user = load[0].parse::<f64>().unwrap();
    let nice = load[1].parse::<f64>().unwrap();
    let system = load[2].parse::<f64>().unwrap();
    let idle = load[3].parse::<f64>().unwrap_or_default(); // since 2.5.41
    let io_wait = load[4].parse::<f64>().unwrap_or_default(); // since 2.5.41
    let hardware_interrupt = load[5].parse::<f64>().unwrap_or_default(); // since 2.6.0
    let software_interrupt = load[6].parse::<f64>().unwrap_or_default(); // since 2.6.0
    let steal_time = load[7].parse::<f64>().unwrap_or_default(); // since 2.6.11
                                                                 // GNU do not show guest and guest_nice
    let guest = load[8].parse::<f64>().unwrap_or_default(); // since 2.6.24
    let guest_nice = load[9].parse::<f64>().unwrap_or_default(); // since 2.6.33
    let total = user
        + nice
        + system
        + idle
        + io_wait
        + hardware_interrupt
        + software_interrupt
        + steal_time
        + guest
        + guest_nice;

    format!(
        "%Cpu(s):  {:.1} us, {:.1} sy, {:.1} ni, {:.1} id, {:.1} wa, {:.1} hi, {:.1} si, {:.1} st",
        user / total * 100.0,
        system / total * 100.0,
        nice / total * 100.0,
        idle / total * 100.0,
        io_wait / total * 100.0,
        hardware_interrupt / total * 100.0,
        software_interrupt / total * 100.0,
        steal_time / total * 100.0,
    )
}

#[cfg(target_os = "windows")]
fn cpu() -> String {
    use libc::malloc;
    use windows::Wdk::System::SystemInformation::NtQuerySystemInformation;

    #[repr(C)]
    #[derive(Debug)]
    struct SystemProcessorPerformanceInformation {
        idle_time: i64,       // LARGE_INTEGER
        kernel_time: i64,     // LARGE_INTEGER
        user_time: i64,       // LARGE_INTEGER
        dpc_time: i64,        // LARGE_INTEGER
        interrupt_time: i64,  // LARGE_INTEGER
        interrupt_count: u32, // ULONG
    }

    let n_cpu = sysinfo().read().unwrap().cpus().len();
    let mut cpu_load = SystemProcessorPerformanceInformation {
        idle_time: 0,
        kernel_time: 0,
        user_time: 0,
        dpc_time: 0,
        interrupt_time: 0,
        interrupt_count: 0,
    };
    unsafe {
        let len = n_cpu * size_of::<SystemProcessorPerformanceInformation>();
        let data = malloc(len);
        let _ = NtQuerySystemInformation(
            windows::Wdk::System::SystemInformation::SystemProcessorPerformanceInformation,
            data,
            (n_cpu * size_of::<SystemProcessorPerformanceInformation>()) as u32,
            std::ptr::null_mut(),
        );
        for i in 0..n_cpu {
            let cpu = data.add(i * size_of::<SystemProcessorPerformanceInformation>())
                as *const SystemProcessorPerformanceInformation;
            let cpu = cpu.as_ref().unwrap();
            cpu_load.idle_time += cpu.idle_time;
            cpu_load.kernel_time += cpu.kernel_time;
            cpu_load.user_time += cpu.user_time;
            cpu_load.dpc_time += cpu.dpc_time;
            cpu_load.interrupt_time += cpu.interrupt_time;
            cpu_load.interrupt_count += cpu.interrupt_count;
        }
    }
    let total = cpu_load.idle_time
        + cpu_load.kernel_time
        + cpu_load.user_time
        + cpu_load.dpc_time
        + cpu_load.interrupt_time;
    format!(
        "%Cpu(s):  {:.1} us,  {:.1} sy,  {:.1} id,  {:.1} hi,  {:.1} si",
        cpu_load.user_time as f64 / total as f64 * 100.0,
        cpu_load.kernel_time as f64 / total as f64 * 100.0,
        cpu_load.idle_time as f64 / total as f64 * 100.0,
        cpu_load.interrupt_time as f64 / total as f64 * 100.0,
        cpu_load.dpc_time as f64 / total as f64 * 100.0,
    )
}

//TODO: Implement for macos
#[cfg(target_os = "macos")]
fn cpu() -> String {
    todo()
}

fn memory(scale_summary_mem: Option<&String>) -> String {
    let binding = sysinfo().read().unwrap();
    let (unit, unit_name) = match scale_summary_mem {
        Some(scale) => match scale.as_str() {
            "k" => (bytesize::KIB, "KiB"),
            "m" => (bytesize::MIB, "MiB"),
            "g" => (bytesize::GIB, "GiB"),
            "t" => (bytesize::TIB, "TiB"),
            "p" => (bytesize::PIB, "PiB"),
            "e" => (1_152_921_504_606_846_976, "EiB"),
            _ => (bytesize::MIB, "MiB"),
        },
        None => (bytesize::MIB, "MiB"),
    };

    format!(
        "{unit_name} Mem : {:8.1} total, {:8.1} free, {:8.1} used, {:8.1} buff/cache\n\
        {unit_name} Swap: {:8.1} total, {:8.1} free, {:8.1} used, {:8.1} avail Mem",
        format_memory(binding.total_memory(), unit),
        format_memory(binding.free_memory(), unit),
        format_memory(binding.used_memory(), unit),
        format_memory(binding.available_memory() - binding.free_memory(), unit),
        format_memory(binding.total_swap(), unit),
        format_memory(binding.free_swap(), unit),
        format_memory(binding.used_swap(), unit),
        format_memory(binding.available_memory(), unit),
        unit_name = unit_name
    )
}
