// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#![allow(unused)]

use crate::header::Memory;
use crate::picker::sysinfo;

pub fn get_cpu_loads() -> Vec<uu_vmstat::CpuLoadRaw> {
    vec![]
}

pub fn get_memory() -> Memory {
    let binding = sysinfo().read().unwrap();

    Memory {
        total: binding.total_memory(),
        free: binding.free_memory(),
        used: binding.used_memory(),
        buff_cache: binding.available_memory() - binding.free_memory(), // TODO: use proper buff/cache instead of available - free
        available: binding.available_memory(),
        total_swap: binding.total_swap(),
        free_swap: binding.free_swap(),
        used_swap: binding.used_swap(),
    }
}
