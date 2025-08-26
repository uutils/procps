// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::picker::sysinfo;
use crate::platform::*;
use crate::{CpuGraphMode, CpuValueMode, Settings};
use bytesize::ByteSize;
use uu_vmstat::CpuLoad;
use uu_w::get_formatted_uptime_procps;
use uucore::uptime::{get_formatted_loadavg, get_formatted_nusers, get_formatted_time};

pub(crate) fn header(settings: &Settings) -> String {
    let uptime_line = format!(
        "top - {time} {uptime}, {user}, {load_average}\n",
        time = get_formatted_time(),
        uptime = uptime(),
        user = user(),
        load_average = load_average(),
    );

    let task_and_cpu = if settings.cpu_graph_mode == CpuGraphMode::Hide {
        String::new()
    } else {
        format!(
            "{task}\n\
            {cpu}\n",
            task = task(),
            cpu = cpu(settings),
        )
    };

    let memory_line = memory(settings.scale_summary_mem.as_ref());

    format!("{uptime_line}{task_and_cpu}{memory_line}")
}

fn format_memory(memory_b: u64, unit: u64) -> f64 {
    ByteSize::b(memory_b).0 as f64 / unit as f64
}

#[inline]
fn uptime() -> String {
    get_formatted_uptime_procps().unwrap_or_default()
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

fn sum_cpu_loads(cpu_loads: &[uu_vmstat::CpuLoadRaw]) -> uu_vmstat::CpuLoadRaw {
    let mut total = uu_vmstat::CpuLoadRaw {
        user: 0,
        nice: 0,
        system: 0,
        idle: 0,
        io_wait: 0,
        hardware_interrupt: 0,
        software_interrupt: 0,
        steal_time: 0,
        guest: 0,
        guest_nice: 0,
    };

    for load in cpu_loads {
        total.user += load.user;
        total.nice += load.nice;
        total.system += load.system;
        total.idle += load.idle;
        total.io_wait += load.io_wait;
        total.hardware_interrupt += load.hardware_interrupt;
        total.software_interrupt += load.software_interrupt;
        total.steal_time += load.steal_time;
        total.guest += load.guest;
        total.guest_nice += load.guest_nice;
    }

    total
}

fn cpu(settings: &Settings) -> String {
    if settings.cpu_graph_mode == CpuGraphMode::Hide {
        return String::new();
    }

    let cpu_loads = get_cpu_loads();

    match settings.cpu_value_mode {
        CpuValueMode::PerCore => {
            let lines = cpu_loads
                .iter()
                .enumerate()
                .map(|(nth, cpu_load_raw)| {
                    let cpu_load = CpuLoad::from_raw(cpu_load_raw);
                    cpu_line(format!("Cpu{nth}").as_str(), &cpu_load, settings)
                })
                .collect::<Vec<String>>();
            lines.join("\n")
        }
        CpuValueMode::Sum => {
            let total = sum_cpu_loads(&cpu_loads);
            let cpu_load = CpuLoad::from_raw(&total);
            cpu_line("Cpu", &cpu_load, settings)
        }
    }
}

fn cpu_line(tag: &str, cpu_load: &CpuLoad, settings: &Settings) -> String {
    if settings.cpu_graph_mode == CpuGraphMode::Hide {
        return String::new();
    }

    if settings.cpu_graph_mode == CpuGraphMode::Sum {
        return format!(
            "%{tag:<6}:  {:.1} us, {:.1} sy, {:.1} ni, {:.1} id, {:.1} wa, {:.1} hi, {:.1} si, {:.1} st",
            cpu_load.user,
            cpu_load.system,
            cpu_load.nice,
            cpu_load.idle,
            cpu_load.io_wait,
            cpu_load.hardware_interrupt,
            cpu_load.software_interrupt,
            cpu_load.steal_time,
        );
    }

    // TODO: render colored bar chart or block chart
    format!("%{tag:<6}: {:>5.1}/{:<5.1}", cpu_load.user, cpu_load.system)
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
