// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

mod parser;

use clap::{crate_version, Command};
#[allow(unused_imports)]
pub use parser::*;
#[cfg(target_os = "linux")]
use procfs::{Current, CurrentSI};
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
    let mut data_len_excess = 0;

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
        .for_each(|f| f(&mut section, &mut title, &mut data, &mut data_len_excess));

    println!("{}", section.join(" "));
    println!("{}", title.join(" "));
    println!("{}", data.join(" "));

    Ok(())
}

type ConcatFunc = Box<dyn Fn(&mut Vec<String>, &mut Vec<String>, &mut Vec<String>, &mut usize)>;

#[allow(dead_code)]
fn concat_helper(
    func: impl Fn() -> (String, String, Vec<(usize, String)>) + 'static,
) -> ConcatFunc {
    Box::from(
        move |section: &mut Vec<String>,
              title: &mut Vec<String>,
              data: &mut Vec<String>,
              data_len_excess: &mut usize| {
            let output = func();
            section.push(output.0);
            title.push(output.1);
            output.2.iter().for_each(|(len, value)| {
                let len = len - *data_len_excess;
                let formatted_value = format!("{:>width$}", value, width = len);
                *data_len_excess = formatted_value.len() - len;
                data.push(formatted_value);
            });
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
fn get_process_info() -> (String, String, Vec<(usize, String)>) {
    let stat = procfs::KernelStats::current().unwrap();
    let runnable = stat.procs_running.unwrap_or_default();
    let blocked = stat.procs_blocked.unwrap_or_default();
    (
        "procs".into(),
        " r  b".into(),
        vec![(2, format!("{}", runnable)), (2, format!("{}", blocked))],
    )
}

#[cfg(target_os = "linux")]
fn get_memory_info() -> (String, String, Vec<(usize, String)>) {
    let memory_info = procfs::Meminfo::current().unwrap();
    let swap_used = (memory_info.swap_total - memory_info.swap_free) / 1024;
    let free = memory_info.mem_free / 1024;
    let buffer = memory_info.buffers / 1024;
    let cache = memory_info.cached / 1024;

    (
        "-----------memory----------".into(),
        "  swpd   free   buff  cache".into(),
        vec![
            (6, format!("{}", swap_used)),
            (6, format!("{}", free)),
            (6, format!("{}", buffer)),
            (6, format!("{}", cache)),
        ],
    )
}

#[cfg(target_os = "linux")]
fn get_swap_info() -> (String, String, Vec<(usize, String)>) {
    let uptime = up_secs();
    let vmstat = procfs::vmstat().unwrap();
    let swap_in = vmstat.get("pswpin").unwrap();
    let swap_out = vmstat.get("pswpout").unwrap();
    (
        "---swap--".into(),
        "  si   so".into(),
        vec![
            (4, format!("{:.0}", *swap_in as f64 / uptime)),
            (4, format!("{:.0}", *swap_out as f64 / uptime)),
        ],
    )
}

#[cfg(target_os = "linux")]
fn get_io_info() -> (String, String, Vec<(usize, String)>) {
    let uptime = up_secs();
    let vmstat = procfs::vmstat().unwrap();
    let read_bytes = vmstat.get("pgpgin").unwrap();
    let write_bytes = vmstat.get("pgpgout").unwrap();
    (
        "-----io----".into(),
        "   bi    bo".into(),
        vec![
            (5, format!("{:.0}", *read_bytes as f64 / uptime)),
            (5, format!("{:.0}", *write_bytes as f64 / uptime)),
        ],
    )
}

#[cfg(target_os = "linux")]
fn get_system_info() -> (String, String, Vec<(usize, String)>) {
    let uptime = up_secs();
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
        vec![
            (4, format!("{:.0}", interrupts as f64 / uptime)),
            (4, format!("{:.0}", context_switches as f64 / uptime)),
        ],
    )
}

#[cfg(target_os = "linux")]
fn get_cpu_info() -> (String, String, Vec<(usize, String)>) {
    let cpu_load = CpuLoad::current();

    (
        "-------cpu-------".into(),
        "us sy id wa st gu".into(),
        vec![
            (2, format!("{:.0}", cpu_load.user)),
            (2, format!("{:.0}", cpu_load.system)),
            (2, format!("{:.0}", cpu_load.idle)),
            (2, format!("{:.0}", cpu_load.io_wait)),
            (2, format!("{:.0}", cpu_load.steal_time)),
            (2, format!("{:.0}", cpu_load.guest)),
        ],
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
