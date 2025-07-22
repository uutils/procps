// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use std::collections::HashMap;

#[cfg(target_os = "linux")]
pub fn parse_proc_file(path: &str) -> HashMap<String, String> {
    let file = std::fs::File::open(std::path::Path::new(path)).unwrap();
    let content = std::io::read_to_string(file).unwrap();
    let mut map: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let parts = line.split_once(char::is_whitespace);
        if let Some(parts) = parts {
            map.insert(
                parts.0.strip_suffix(":").unwrap_or(parts.0).to_string(),
                parts.1.trim_start().to_string(),
            );
        }
    }

    map
}

#[cfg(target_os = "linux")]
pub struct ProcData {
    pub uptime: (f64, f64),
    pub stat: HashMap<String, String>,
    pub meminfo: HashMap<String, String>,
    pub vmstat: HashMap<String, String>,
}
#[cfg(target_os = "linux")]
impl Default for ProcData {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(target_os = "linux")]
impl ProcData {
    pub fn new() -> Self {
        let uptime = Self::get_uptime();
        let stat = parse_proc_file("/proc/stat");
        let meminfo = parse_proc_file("/proc/meminfo");
        let vmstat = parse_proc_file("/proc/vmstat");
        Self {
            uptime,
            stat,
            meminfo,
            vmstat,
        }
    }

    fn get_uptime() -> (f64, f64) {
        let file = std::fs::File::open(std::path::Path::new("/proc/uptime")).unwrap();
        let content = std::io::read_to_string(file).unwrap();
        let mut parts = content.split_whitespace();
        let uptime = parts.next().unwrap().parse::<f64>().unwrap();
        let idle_time = parts.next().unwrap().parse::<f64>().unwrap();
        (uptime, idle_time)
    }

    pub fn get_one<T>(table: &HashMap<String, String>, name: &str) -> T
    where
        T: Default + std::str::FromStr,
    {
        table
            .get(name)
            .and_then(|v| v.parse().ok())
            .unwrap_or_default()
    }
}

#[cfg(target_os = "linux")]
pub struct CpuLoadRaw {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub io_wait: u64,
    pub hardware_interrupt: u64,
    pub software_interrupt: u64,
    pub steal_time: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

#[cfg(target_os = "linux")]
pub struct CpuLoad {
    pub user: f64,
    pub nice: f64,
    pub system: f64,
    pub idle: f64,
    pub io_wait: f64,
    pub hardware_interrupt: f64,
    pub software_interrupt: f64,
    pub steal_time: f64,
    pub guest: f64,
    pub guest_nice: f64,
}

#[cfg(target_os = "linux")]
impl CpuLoadRaw {
    pub fn current() -> Self {
        let file = std::fs::File::open(std::path::Path::new("/proc/stat")).unwrap(); // do not use `parse_proc_file` here because only one line is used
        let content = std::io::read_to_string(file).unwrap();
        let load_str = content.lines().next().unwrap().strip_prefix("cpu").unwrap();
        Self::from_str(load_str)
    }

    pub fn from_proc_map(proc_map: &HashMap<String, String>) -> Self {
        let load_str = proc_map.get("cpu").unwrap();
        Self::from_str(load_str)
    }

    fn from_str(s: &str) -> Self {
        let load = s.split(' ').filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let user = load[0].parse::<u64>().unwrap();
        let nice = load[1].parse::<u64>().unwrap();
        let system = load[2].parse::<u64>().unwrap();
        let idle = load[3].parse::<u64>().unwrap_or_default(); // since 2.5.41
        let io_wait = load[4].parse::<u64>().unwrap_or_default(); // since 2.5.41
        let hardware_interrupt = load[5].parse::<u64>().unwrap_or_default(); // since 2.6.0
        let software_interrupt = load[6].parse::<u64>().unwrap_or_default(); // since 2.6.0
        let steal_time = load[7].parse::<u64>().unwrap_or_default(); // since 2.6.11
        let guest = load[8].parse::<u64>().unwrap_or_default(); // since 2.6.24
        let guest_nice = load[9].parse::<u64>().unwrap_or_default(); // since 2.6.33

        Self {
            user,
            system,
            nice,
            idle,
            io_wait,
            hardware_interrupt,
            software_interrupt,
            steal_time,
            guest,
            guest_nice,
        }
    }
}

#[cfg(target_os = "linux")]
impl CpuLoad {
    pub fn current() -> Self {
        Self::from_raw(CpuLoadRaw::current())
    }

    pub fn from_proc_map(proc_map: &HashMap<String, String>) -> Self {
        Self::from_raw(CpuLoadRaw::from_proc_map(proc_map))
    }

    pub fn from_raw(raw_data: CpuLoadRaw) -> Self {
        let total = (raw_data.user
            + raw_data.nice
            + raw_data.system
            + raw_data.idle
            + raw_data.io_wait
            + raw_data.hardware_interrupt
            + raw_data.software_interrupt
            + raw_data.steal_time
            + raw_data.guest
            + raw_data.guest_nice) as f64;
        Self {
            user: raw_data.user as f64 / total * 100.0,
            system: raw_data.system as f64 / total * 100.0,
            nice: raw_data.nice as f64 / total * 100.0,
            idle: raw_data.idle as f64 / total * 100.0,
            io_wait: raw_data.io_wait as f64 / total * 100.0,
            hardware_interrupt: raw_data.hardware_interrupt as f64 / total * 100.0,
            software_interrupt: raw_data.software_interrupt as f64 / total * 100.0,
            steal_time: raw_data.steal_time as f64 / total * 100.0,
            guest: raw_data.guest as f64 / total * 100.0,
            guest_nice: raw_data.guest_nice as f64 / total * 100.0,
        }
    }
}

#[cfg(target_os = "linux")]
pub struct Meminfo {
    pub mem_total: bytesize::ByteSize,
    pub mem_free: bytesize::ByteSize,
    pub mem_available: bytesize::ByteSize,
    pub buffers: bytesize::ByteSize,
    pub cached: bytesize::ByteSize,
    pub swap_cached: bytesize::ByteSize,
    pub active: bytesize::ByteSize,
    pub inactive: bytesize::ByteSize,
    pub swap_total: bytesize::ByteSize,
    pub swap_free: bytesize::ByteSize,
}
#[cfg(target_os = "linux")]
impl Meminfo {
    pub fn current() -> Self {
        let meminfo = parse_proc_file("/proc/meminfo");
        Self::from_proc_map(&meminfo)
    }

    pub fn from_proc_map(proc_map: &HashMap<String, String>) -> Self {
        use std::str::FromStr;

        let mem_total = bytesize::ByteSize::from_str(proc_map.get("MemTotal").unwrap()).unwrap();
        let mem_free = bytesize::ByteSize::from_str(proc_map.get("MemFree").unwrap()).unwrap();
        let mem_available =
            bytesize::ByteSize::from_str(proc_map.get("MemAvailable").unwrap()).unwrap();
        let buffers = bytesize::ByteSize::from_str(proc_map.get("Buffers").unwrap()).unwrap();
        let cached = bytesize::ByteSize::from_str(proc_map.get("Cached").unwrap()).unwrap();
        let swap_cached =
            bytesize::ByteSize::from_str(proc_map.get("SwapCached").unwrap()).unwrap();
        let active = bytesize::ByteSize::from_str(proc_map.get("Active").unwrap()).unwrap();
        let inactive = bytesize::ByteSize::from_str(proc_map.get("Inactive").unwrap()).unwrap();
        let swap_total = bytesize::ByteSize::from_str(proc_map.get("SwapTotal").unwrap()).unwrap();
        let swap_free = bytesize::ByteSize::from_str(proc_map.get("SwapFree").unwrap()).unwrap();
        Self {
            mem_total,
            mem_free,
            mem_available,
            buffers,
            cached,
            swap_cached,
            active,
            inactive,
            swap_total,
            swap_free,
        }
    }
}
