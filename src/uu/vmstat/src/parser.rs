#[cfg(target_os = "linux")]
pub fn parse_proc_file(path: &str) -> std::collections::HashMap<String, String> {
    let file = std::fs::File::open(std::path::Path::new(path)).unwrap();
    let content = std::io::read_to_string(file).unwrap();
    let mut map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for line in content.lines() {
        let parts = line.split_once(char::is_whitespace);
        if let Some(parts) = parts {
            map.insert(parts.0.to_string(), parts.1.trim_start().to_string());
        }
    }

    map
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
impl CpuLoad {
    pub fn current() -> CpuLoad {
        let file = std::fs::File::open(std::path::Path::new("/proc/stat")).unwrap(); // do not use `parse_proc_file` here because only one line is used
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
        Self {
            user: user / total * 100.0,
            system: system / total * 100.0,
            nice: nice / total * 100.0,
            idle: idle / total * 100.0,
            io_wait: io_wait / total * 100.0,
            hardware_interrupt: hardware_interrupt / total * 100.0,
            software_interrupt: software_interrupt / total * 100.0,
            steal_time: steal_time / total * 100.0,
            guest: guest / total * 100.0,
            guest_nice: guest_nice / total * 100.0,
        }
    }
}
