// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::picker::sysinfo;
use crate::platform::*;
use crate::tui::stat::{CpuValueMode, TuiStat};
use bytesize::ByteSize;
use uu_vmstat::{CpuLoad, CpuLoadRaw};
use uu_w::get_formatted_uptime_procps;
use uucore::uptime::{get_formatted_loadavg, get_formatted_nusers, get_formatted_time};

pub(crate) struct Header {
    pub uptime: Uptime,
    pub task: Task,
    pub cpu: Vec<(String, CpuLoad)>,
    pub memory: Memory,
}

impl Header {
    pub fn new(stat: &TuiStat) -> Header {
        Header {
            uptime: Uptime::new(),
            task: Task::new(),
            cpu: cpu(stat),
            memory: Memory::new(),
        }
    }

    pub fn update_cpu(&mut self, stat: &TuiStat) {
        self.cpu = cpu(stat);
    }
}

pub(crate) struct Uptime {
    pub time: String,
    pub uptime: String,
    pub user: String,
    pub load_average: String,
}

impl Uptime {
    pub fn new() -> Uptime {
        Uptime {
            time: get_formatted_time(),
            uptime: get_formatted_uptime_procps().unwrap_or_default(),
            user: user(),
            load_average: get_formatted_loadavg().unwrap_or_default(),
        }
    }
}

pub(crate) struct Task {
    pub total: usize,
    pub running: usize,
    pub sleeping: usize,
    pub stopped: usize,
    pub zombie: usize,
}
impl Task {
    pub fn new() -> Task {
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

        Task {
            total: process.len(),
            running: running_process,
            sleeping: sleeping_process,
            stopped: stopped_process,
            zombie: zombie_process,
        }
    }
}

pub(crate) struct Memory {
    pub total: u64,
    pub free: u64,
    pub used: u64,
    pub buff_cache: u64,
    pub available: u64,
    pub total_swap: u64,
    pub free_swap: u64,
    pub used_swap: u64,
}

impl Memory {
    pub fn new() -> Memory {
        get_memory()
    }
}

pub(crate) fn format_memory(memory_b: u64, unit: u64) -> f64 {
    ByteSize::b(memory_b).0 as f64 / unit as f64
}

// see: https://gitlab.com/procps-ng/procps/-/blob/4740a0efa79cade867cfc7b32955fe0f75bf5173/library/uptime.c#L63-L115
fn user() -> String {
    #[cfg(target_os = "linux")]
    if let Ok(nusers) = get_nusers_systemd() {
        return uucore::uptime::format_nusers(nusers);
    }

    get_formatted_nusers()
}

fn sum_cpu_loads(cpu_loads: Vec<&CpuLoadRaw>) -> CpuLoadRaw {
    let mut total = CpuLoadRaw {
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

fn cpu(stat: &TuiStat) -> Vec<(String, CpuLoad)> {
    let cpu_loads = get_cpu_loads();

    match stat.cpu_value_mode {
        CpuValueMode::PerCore => cpu_loads
            .iter()
            .enumerate()
            .map(|(nth, cpu_load_raw)| {
                let cpu_load = CpuLoad::from_raw(cpu_load_raw);
                (format!("Cpu{nth}"), cpu_load)
            })
            .collect::<Vec<(String, CpuLoad)>>(),
        CpuValueMode::Sum => {
            let total = sum_cpu_loads(cpu_loads.iter().collect());
            let cpu_load = CpuLoad::from_raw(&total);
            vec![(String::from("Cpu(s)"), cpu_load)]
        }
        CpuValueMode::Numa => {
            let numa_nodes = get_numa_nodes();
            let total = sum_cpu_loads(cpu_loads.iter().collect());
            let cpu_load = CpuLoad::from_raw(&total);
            let mut data = vec![(String::from("Cpu(s)"), cpu_load)];
            for (id, cores) in numa_nodes {
                let loads = cores.iter().map(|id| &cpu_loads[*id]).collect();
                let total = sum_cpu_loads(loads);
                let cpu_load = CpuLoad::from_raw(&total);
                data.push((format!("Node{id}"), cpu_load));
            }
            data
        }
        CpuValueMode::NumaNode(id) => {
            let numa_nodes = get_numa_nodes();
            if let Some(cores) = numa_nodes.get(&id) {
                let loads = cores.iter().map(|id| &cpu_loads[*id]).collect();
                let total = sum_cpu_loads(loads);
                let cpu_load = CpuLoad::from_raw(&total);
                let mut data = vec![(format!("Node{id}"), cpu_load)];
                data.extend(cores.iter().map(|core_id| {
                    let cpu_load = CpuLoad::from_raw(&cpu_loads[*core_id]);
                    (format!("Cpu{core_id}"), cpu_load)
                }));
                data
            } else {
                vec![]
            }
        }
    }
}
