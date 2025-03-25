// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{crate_version, Command};
#[cfg(target_os = "linux")]
use procfs::{Current, CurrentSI};
#[cfg(target_os = "linux")]
use std::collections::HashMap;
use uucore::error::UResult;
use uucore::{format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("vmstat.md");
const USAGE: &str = help_usage!("vmstat.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let _matches = uu_app().try_get_matches_from(args)?;

    let mut section: Vec<String> = vec![];
    let mut title: Vec<String> = vec![];
    let mut data: Vec<String> = vec![];

    #[cfg(target_os = "linux")]
    let func = [
        concat_helper(get_process_info),
        concat_helper(get_memory_info),
        concat_helper(get_swap_info),
        concat_helper(get_io_info),
        concat_helper(get_system_info),
        concat_helper(get_cpu_info),
    ];

    #[cfg(not(target_os = "linux"))]
    let func: [ConcatFunc; 0] = [];

    func.iter()
        .for_each(|f| f(&mut section, &mut title, &mut data));

    println!("{}", section.join(" "));
    println!("{}", title.join(" "));
    println!("{}", data.join(" "));

    Ok(())
}

type ConcatFunc = Box<dyn Fn(&mut Vec<String>, &mut Vec<String>, &mut Vec<String>)>;

#[allow(dead_code)]
fn concat_helper(func: impl Fn() -> (String, String, String) + 'static) -> ConcatFunc {
    Box::from(
        move |section: &mut Vec<String>, title: &mut Vec<String>, data: &mut Vec<String>| {
            let output = func();
            section.push(output.0);
            title.push(output.1);
            data.push(output.2);
        },
    )
}

#[cfg(target_os = "linux")]
fn up_secs() -> f64 {
    let file = std::fs::File::open(std::path::Path::new("/proc/uptime")).unwrap();
    let content = std::io::read_to_string(file).unwrap();
    let mut parts = content.split_whitespace();
    parts.next().unwrap().parse::<f64>().unwrap()
}

#[cfg(target_os = "linux")]
fn up_secs_proc() -> f64 {
    let stat = parse_proc_file("/proc/stat");
    let n_proc = stat.keys().filter(|k| k.starts_with("cpu")).count() - 1; // exclude the line `cpu`

    n_proc as f64 * up_secs()
}

#[cfg(target_os = "linux")]
fn parse_proc_file(path: &str) -> HashMap<String, String> {
    let file = std::fs::File::open(std::path::Path::new(path)).unwrap();
    let content = std::io::read_to_string(file).unwrap();
    let mut map: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let parts = line.split_once(char::is_whitespace);
        if let Some(parts) = parts {
            map.insert(parts.0.to_string(), parts.1.trim_start().to_string());
        }
    }

    map
}

#[cfg(target_os = "linux")]
fn get_process_info() -> (String, String, String) {
    let stat = procfs::KernelStats::current().unwrap();
    let runnable = stat.procs_running.unwrap_or_default();
    let blocked = stat.procs_blocked.unwrap_or_default();
    (
        "procs".into(),
        " r  b".into(),
        format!("{:>2} {:>2}", runnable, blocked),
    )
}

#[cfg(target_os = "linux")]
fn get_memory_info() -> (String, String, String) {
    let memory_info = procfs::Meminfo::current().unwrap();
    let swap_used = (memory_info.swap_total - memory_info.swap_free) / 1024;
    let free = memory_info.mem_free / 1024;
    let buffer = memory_info.buffers / 1024;
    let cache = memory_info.cached / 1024;

    (
        "-----------memory----------".into(),
        "  swpd   free   buff  cache".into(),
        format!("{:>6} {:>6} {:>6} {:>6}", swap_used, free, buffer, cache),
    )
}

#[cfg(target_os = "linux")]
fn get_swap_info() -> (String, String, String) {
    let uptime = up_secs_proc();
    let vmstat = procfs::vmstat().unwrap();
    let swap_in = vmstat.get("pswpin").unwrap();
    let swap_out = vmstat.get("pswpout").unwrap();
    (
        "---swap--".into(),
        "  si   so".into(),
        format!(
            "{:>4} {:>4}",
            *swap_in as f64 / uptime,
            *swap_out as f64 / uptime
        ),
    )
}

#[cfg(target_os = "linux")]
fn get_io_info() -> (String, String, String) {
    let uptime = up_secs_proc();
    let vmstat = procfs::vmstat().unwrap();
    let read_bytes = vmstat.get("pgpgin").unwrap();
    let write_bytes = vmstat.get("pgpgout").unwrap();
    (
        "----io----".into(),
        "  bi    bo".into(),
        format!(
            "{:>4.0} {:>4.0}",
            *read_bytes as f64 / uptime,
            *write_bytes as f64 / uptime
        ),
    )
}

#[cfg(target_os = "linux")]
fn get_system_info() -> (String, String, String) {
    let uptime = up_secs_proc();
    let stat = parse_proc_file("/proc/stat");

    let interrupts = stat
        .get("intr")
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse::<i64>()
        .unwrap();
    let context_switches = stat.get("ctxt").unwrap().parse::<i64>().unwrap();

    (
        "-system--".into(),
        "  in   cs".into(),
        format!(
            "{:>4.0} {:>4.0}",
            interrupts as f64 / uptime,
            context_switches as f64 / uptime
        ),
    )
}

#[cfg(target_os = "linux")]
fn get_cpu_info() -> (String, String, String) {
    let stat = parse_proc_file("/proc/stat");
    let load = stat
        .get("cpu")
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

    (
        "------cpu-----".into(),
        "us sy id wa st".into(),
        format!(
            "{:>2.0} {:>2.0} {:>2.0} {:>2.0} {:>2.0}",
            user / total * 100.0,
            system / total * 100.0,
            idle / total * 100.0,
            io_wait / total * 100.0,
            steal_time / total * 100.0,
        ),
    )
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
    // .args([
    // arg!(<delay> "The delay between updates in seconds").required(false),
    // arg!(<count> "Number of updates").required(false),
    // arg!(-a --active "Display active and inactive memory"),
    // arg!(-f --forks "switch displays the number of forks since boot"),
    // arg!(-m --slabs "Display slabinfo"),
    // arg!(-n --one-header "Display the header only once rather than periodically"),
    // arg!(-s --stats "Displays a table of various event counters and memory statistics"),
    // arg!(-d --disk "Report disk statistics"),
    // arg!(-D --disk-sum "Report some summary statistics about disk activity"),
    // arg!(-p --partition <device> "Detailed statistics about partition"),
    // arg!(-S --unit <character> "Switches outputs between 1000 (k), 1024 (K), 1000000 (m), or 1048576 (M) bytes"),
    // arg!(-t --timestamp "Append timestamp to each line"),
    // arg!(-w --wide "Wide output mode"),
    // arg!(-y --no-first "Omits first report with statistics since system boot"),
    // arg!(-V --version "Display version information and exit"),
    // arg!(-h --help "Display help and exit"),
    // ])
}
