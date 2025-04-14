// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use crate::{CpuLoad, Meminfo, ProcData};
#[cfg(target_os = "linux")]
use clap::ArgMatches;

#[cfg(target_os = "linux")]
pub type Picker = (
    (String, String),
    Box<dyn Fn(&ProcData, Option<&ProcData>, &ArgMatches, &mut Vec<String>, &mut usize)>,
);

#[cfg(target_os = "linux")]
pub fn get_pickers(matches: &ArgMatches) -> Vec<Picker> {
    let wide = matches.get_flag("wide");
    vec![
        concat_helper(
            if wide {
                ("--procs--".into(), "   r    b".into())
            } else {
                ("procs".into(), " r  b".into())
            },
            get_process_info,
        ),
        concat_helper(
            if wide {
                (
                    "-----------------------memory----------------------".into(),
                    if matches.get_flag("active") {
                        "        swpd         free        inact       active".into()
                    } else {
                        "        swpd         free         buff        cache".into()
                    },
                )
            } else {
                (
                    "-----------memory----------".into(),
                    if matches.get_flag("active") {
                        "  swpd   free  inact active".into()
                    } else {
                        "  swpd   free   buff  cache".into()
                    },
                )
            },
            get_memory_info,
        ),
        concat_helper(("---swap--".into(), "  si   so".into()), get_swap_info),
        concat_helper(("-----io----".into(), "   bi    bo".into()), get_io_info),
        concat_helper(("-system--".into(), "  in   cs".into()), get_system_info),
        concat_helper(
            if wide {
                (
                    "----------cpu----------".into(),
                    " us  sy  id  wa  st  gu".into(),
                )
            } else {
                ("-------cpu-------".into(), "us sy id wa st gu".into())
            },
            get_cpu_info,
        ),
    ]
}

#[cfg(target_os = "linux")]
fn with_unit(x: u64, arg: &ArgMatches) -> u64 {
    if let Some(unit) = arg.get_one::<String>("unit") {
        return match unit.as_str() {
            "k" => x / bytesize::KB,
            "K" => x / bytesize::KIB,
            "m" => x / bytesize::MB,
            "M" => x / bytesize::MIB,
            _ => x, // impossible
        };
    }
    x / bytesize::KIB
}

#[cfg(target_os = "linux")]
fn concat_helper(
    title: (String, String),
    func: impl Fn(&ProcData, Option<&ProcData>, &ArgMatches) -> Vec<(usize, String)> + 'static,
) -> Picker {
    (
        title,
        Box::from(
            move |proc_data: &ProcData,
                  proc_data_before: Option<&ProcData>,
                  matches: &ArgMatches,
                  data: &mut Vec<String>,
                  data_len_excess: &mut usize| {
                let output = func(proc_data, proc_data_before, matches);
                output.iter().for_each(|(len, value)| {
                    let len = if *data_len_excess > *len {
                        0
                    } else {
                        len - *data_len_excess
                    };
                    let formatted_value = format!("{:>width$}", value, width = len);
                    *data_len_excess = formatted_value.len() - len;
                    data.push(formatted_value);
                });
            },
        ),
    )
}

#[cfg(target_os = "linux")]
macro_rules! diff {
    ($now:expr, $before:expr, $($property:tt)*) => {
        if let Some(before) = &$before {
            $now.$($property)* - before.$($property)*
        } else {
            $now.$($property)*
        }
    };
}

#[cfg(target_os = "linux")]
fn get_process_info(
    proc_data: &ProcData,
    _proc_data_before: Option<&ProcData>,
    matches: &ArgMatches,
) -> Vec<(usize, String)> {
    let runnable = proc_data.stat.get("procs_running").unwrap();
    let blocked = proc_data.stat.get("procs_blocked").unwrap();
    let len = if matches.get_flag("wide") { 4 } else { 2 };

    vec![(len, runnable.to_string()), (len, blocked.to_string())]
}

#[cfg(target_os = "linux")]
fn get_memory_info(
    proc_data: &ProcData,
    _proc_data_before: Option<&ProcData>,
    matches: &ArgMatches,
) -> Vec<(usize, String)> {
    let len = if matches.get_flag("wide") { 12 } else { 6 };
    let memory_info = Meminfo::from_proc_map(&proc_data.meminfo);

    let swap_used = with_unit(
        (memory_info.swap_total - memory_info.swap_free).as_u64(),
        matches,
    );
    let free = with_unit(memory_info.mem_free.as_u64(), matches);

    if matches.get_flag("active") {
        let inactive = with_unit(memory_info.inactive.as_u64(), matches);
        let active = with_unit(memory_info.active.as_u64(), matches);
        return vec![
            (len, format!("{}", swap_used)),
            (len, format!("{}", free)),
            (len, format!("{}", inactive)),
            (len, format!("{}", active)),
        ];
    }

    let buffer = with_unit(memory_info.buffers.as_u64(), matches);
    let cache = with_unit(memory_info.cached.as_u64(), matches);

    vec![
        (len, format!("{}", swap_used)),
        (len, format!("{}", free)),
        (len, format!("{}", buffer)),
        (len, format!("{}", cache)),
    ]
}

#[cfg(target_os = "linux")]
fn get_swap_info(
    proc_data: &ProcData,
    proc_data_before: Option<&ProcData>,
    _matches: &ArgMatches,
) -> Vec<(usize, String)> {
    let period = diff!(proc_data, proc_data_before, uptime.0);
    let swap_in = diff!(
        proc_data,
        proc_data_before,
        vmstat.get("pswpin").unwrap().parse::<u64>().unwrap()
    );
    let swap_out = diff!(
        proc_data,
        proc_data_before,
        vmstat.get("pswpout").unwrap().parse::<u64>().unwrap()
    );

    vec![
        (4, format!("{:.0}", swap_in as f64 / period)),
        (4, format!("{:.0}", swap_out as f64 / period)),
    ]
}

#[cfg(target_os = "linux")]
fn get_io_info(
    proc_data: &ProcData,
    proc_data_before: Option<&ProcData>,
    _matches: &ArgMatches,
) -> Vec<(usize, String)> {
    let period = diff!(proc_data, proc_data_before, uptime.0);
    let read_bytes = diff!(
        proc_data,
        proc_data_before,
        vmstat.get("pgpgin").unwrap().parse::<u64>().unwrap()
    );
    let write_bytes = diff!(
        proc_data,
        proc_data_before,
        vmstat.get("pgpgout").unwrap().parse::<u64>().unwrap()
    );

    vec![
        (5, format!("{:.0}", read_bytes as f64 / period)),
        (5, format!("{:.0}", write_bytes as f64 / period)),
    ]
}

#[cfg(target_os = "linux")]
fn get_system_info(
    proc_data: &ProcData,
    proc_data_before: Option<&ProcData>,
    _matches: &ArgMatches,
) -> Vec<(usize, String)> {
    let period = diff!(proc_data, proc_data_before, uptime.0);

    let interrupts = diff!(
        proc_data,
        proc_data_before,
        stat.get("intr")
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<i64>()
            .unwrap()
    );
    let context_switches = diff!(
        proc_data,
        proc_data_before,
        stat.get("ctxt").unwrap().parse::<i64>().unwrap()
    );

    vec![
        (4, format!("{:.0}", interrupts as f64 / period)),
        (4, format!("{:.0}", context_switches as f64 / period)),
    ]
}

#[cfg(target_os = "linux")]
fn get_cpu_info(
    proc_data: &ProcData,
    _proc_data_before: Option<&ProcData>,
    matches: &ArgMatches,
) -> Vec<(usize, String)> {
    let len = if matches.get_flag("wide") { 3 } else { 2 };

    let cpu_load = CpuLoad::from_proc_map(&proc_data.stat);

    vec![
        (len, format!("{:.0}", cpu_load.user)),
        (len, format!("{:.0}", cpu_load.system)),
        (len, format!("{:.0}", cpu_load.idle)),
        (len, format!("{:.0}", cpu_load.io_wait)),
        (len, format!("{:.0}", cpu_load.steal_time)),
        (len, format!("{:.0}", cpu_load.guest)),
    ]
}
