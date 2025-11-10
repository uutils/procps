// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::picker::sysinfo;
use windows_sys::Wdk::System::SystemInformation::NtQuerySystemInformation;

pub fn get_cpu_loads() -> Vec<uu_vmstat::CpuLoadRaw> {
    let mut cpu_loads = Vec::new();

    #[repr(C)]
    #[derive(Debug, Clone)]
    struct SystemProcessorPerformanceInformation {
        idle_time: i64,       // LARGE_INTEGER
        kernel_time: i64,     // LARGE_INTEGER
        user_time: i64,       // LARGE_INTEGER
        dpc_time: i64,        // LARGE_INTEGER
        interrupt_time: i64,  // LARGE_INTEGER
        interrupt_count: u32, // ULONG
    }

    let n_cpu = sysinfo().read().unwrap().cpus().len();

    let mut data = vec![
        SystemProcessorPerformanceInformation {
            idle_time: 0,
            kernel_time: 0,
            user_time: 0,
            dpc_time: 0,
            interrupt_time: 0,
            interrupt_count: 0,
        };
        n_cpu
    ];
    let status = unsafe {
        NtQuerySystemInformation(
            windows_sys::Wdk::System::SystemInformation::SystemProcessorPerformanceInformation,
            data.as_mut_ptr() as *mut uucore::libc::c_void,
            (n_cpu * size_of::<SystemProcessorPerformanceInformation>()) as u32,
            std::ptr::null_mut(),
        )
    };

    if status == 0 {
        data.iter().for_each(|load| {
            let raw = uu_vmstat::CpuLoadRaw {
                user: load.user_time as u64,
                nice: 0,
                system: load.kernel_time as u64,
                idle: load.idle_time as u64,
                io_wait: 0,
                hardware_interrupt: load.interrupt_time as u64,
                software_interrupt: load.dpc_time as u64,
                steal_time: 0,
                guest: 0,
                guest_nice: 0,
            };
            cpu_loads.push(raw);
        });
    }

    cpu_loads
}
