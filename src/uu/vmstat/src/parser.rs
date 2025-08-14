// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::fmt::{Debug, Display, Formatter};

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
    pub diskstat: Vec<String>,
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
        let diskstat = std::fs::read_to_string("/proc/diskstats")
            .unwrap()
            .lines()
            .map(|line| line.to_string())
            .collect();
        Self {
            uptime,
            stat,
            meminfo,
            vmstat,
            diskstat,
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

#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct DiskStatParseError;

#[cfg(target_os = "linux")]
impl Display for DiskStatParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt("Failed to parse diskstat line", f)
    }
}

#[cfg(target_os = "linux")]
impl std::error::Error for DiskStatParseError {}

#[cfg(target_os = "linux")]
pub struct DiskStat {
    // Name from https://www.kernel.org/doc/html/latest/admin-guide/iostats.html
    pub major: u64,
    pub minor: u64,
    pub device: String,
    pub reads_completed: u64,
    pub reads_merged: u64,
    pub sectors_read: u64,
    pub milliseconds_spent_reading: u64,
    pub writes_completed: u64,
    pub writes_merged: u64,
    pub sectors_written: u64,
    pub milliseconds_spent_writing: u64,
    pub ios_currently_in_progress: u64,
    pub milliseconds_spent_doing_ios: u64,
    pub weighted_milliseconds_spent_doing_ios: u64,
    pub discards_completed: u64,
    pub discards_merged: u64,
    pub sectors_discarded: u64,
    pub milliseconds_spent_discarding: u64,
    pub flush_requests_completed: u64,
    pub milliseconds_spent_flushing: u64,
}

#[cfg(target_os = "linux")]
impl std::str::FromStr for DiskStat {
    type Err = DiskStatParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 14 {
            Err(DiskStatParseError)?;
        }

        let parse_value = |s: &str| s.parse::<u64>().map_err(|_| DiskStatParseError);
        let parse_optional_value = |s: Option<&&str>| match s {
            None => Ok(0),
            Some(value) => value.parse::<u64>().map_err(|_| DiskStatParseError),
        };

        Ok(Self {
            major: parse_value(parts[0])?,
            minor: parse_value(parts[1])?,
            device: parts[2].to_string(),
            reads_completed: parse_value(parts[3])?,
            reads_merged: parse_value(parts[4])?,
            sectors_read: parse_value(parts[5])?,
            milliseconds_spent_reading: parse_value(parts[6])?,
            writes_completed: parse_value(parts[7])?,
            writes_merged: parse_value(parts[8])?,
            sectors_written: parse_value(parts[9])?,
            milliseconds_spent_writing: parse_value(parts[10])?,
            ios_currently_in_progress: parse_value(parts[11])?,
            milliseconds_spent_doing_ios: parse_value(parts[12])?,
            weighted_milliseconds_spent_doing_ios: parse_optional_value(parts.get(13))?,
            discards_completed: parse_optional_value(parts.get(14))?,
            discards_merged: parse_optional_value(parts.get(15))?,
            sectors_discarded: parse_optional_value(parts.get(16))?,
            milliseconds_spent_discarding: parse_optional_value(parts.get(17))?,
            flush_requests_completed: parse_optional_value(parts.get(18))?,
            milliseconds_spent_flushing: parse_optional_value(parts.get(19))?,
        })
    }
}

#[cfg(target_os = "linux")]
impl DiskStat {
    pub fn is_disk(&self) -> bool {
        std::path::Path::new(&format!("/sys/block/{}", self.device)).exists()
    }

    pub fn current() -> Result<Vec<Self>, DiskStatParseError> {
        let diskstats =
            std::fs::read_to_string("/proc/diskstats").map_err(|_| DiskStatParseError)?;
        let lines = diskstats.lines();
        Self::from_proc_vec(&lines.map(|line| line.to_string()).collect::<Vec<_>>())
    }

    pub fn from_proc_vec(proc_vec: &[String]) -> Result<Vec<Self>, DiskStatParseError> {
        proc_vec
            .iter()
            .map(|line| line.parse::<DiskStat>())
            .collect()
    }
}
