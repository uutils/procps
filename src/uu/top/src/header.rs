// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::picker::sysinfo;
use bytesize::ByteSize;
use uucore::uptime::{
    get_formated_uptime, get_formatted_loadavg, get_formatted_nusers, get_formatted_time,
};

pub(crate) fn header(scale_summary_mem: Option<&String>) -> String {
    format!(
        "top - {time} {uptime}, {user}, {load_average}\n\
        {task}\n\
        {cpu}\n\
        {memory}",
        time = get_formatted_time(),
        uptime = uptime(),
        user = user(),
        load_average = load_average(),
        task = task(),
        cpu = cpu(),
        memory = memory(scale_summary_mem),
    )
}

#[cfg(target_os = "linux")]
extern "C" {
    pub fn sd_booted() -> libc::c_int;
    pub fn sd_get_sessions(sessions: *mut *mut *mut libc::c_char) -> libc::c_int;
    pub fn sd_session_get_class(
        session: *const libc::c_char,
        class: *mut *mut libc::c_char,
    ) -> libc::c_int;
}

fn format_memory(memory_b: u64, unit: u64) -> f64 {
    ByteSize::b(memory_b).0 as f64 / unit as f64
}

fn uptime() -> String {
    get_formated_uptime(None).unwrap_or_default()
}

#[cfg(target_os = "linux")]
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

// see: https://gitlab.com/procps-ng/procps/-/blob/4740a0efa79cade867cfc7b32955fe0f75bf5173/library/uptime.c#L63-L115
fn user() -> String {
    #[cfg(target_os = "linux")]
    if let Ok(nusers) = get_nusers_systemd() {
        return uucore::uptime::format_nusers(nusers);
    }

    get_formatted_nusers()
}

fn load_average() -> String {
    get_formatted_loadavg().unwrap_or_default()
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
    let cpu_load = uu_vmstat::CpuLoad::current();

    format!(
        "%Cpu(s):  {:.1} us, {:.1} sy, {:.1} ni, {:.1} id, {:.1} wa, {:.1} hi, {:.1} si, {:.1} st",
        cpu_load.user,
        cpu_load.system,
        cpu_load.nice,
        cpu_load.idle,
        cpu_load.io_wait,
        cpu_load.hardware_interrupt,
        cpu_load.software_interrupt,
        cpu_load.steal_time,
    )
}

#[cfg(target_os = "windows")]
fn cpu() -> String {
    use libc::malloc;
    use windows_sys::Wdk::System::SystemInformation::NtQuerySystemInformation;

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
    // SAFETY: malloc is safe to use here. We free the memory after we are done with it. If action fails, all "time" will be 0.
    unsafe {
        let len = n_cpu * size_of::<SystemProcessorPerformanceInformation>();
        let data = malloc(len);
        let status = NtQuerySystemInformation(
            windows_sys::Wdk::System::SystemInformation::SystemProcessorPerformanceInformation,
            data,
            (n_cpu * size_of::<SystemProcessorPerformanceInformation>()) as u32,
            std::ptr::null_mut(),
        );
        if status == 0 {
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
    "TODO".into()
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
