// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "linux")]
use crate::{CpuLoad, Meminfo, ProcData};

#[cfg(target_os = "linux")]
pub type Picker = (
    (String, String),
    Box<dyn Fn(&ProcData, Option<&ProcData>, &mut Vec<String>, &mut usize)>,
);

#[cfg(target_os = "linux")]
pub fn get_pickers() -> Vec<Picker> {
    vec![
        concat_helper(("procs".into(), " r  b".into()), get_process_info),
        concat_helper(
            (
                "-----------memory----------".into(),
                "  swpd   free   buff  cache".into(),
            ),
            get_memory_info,
        ),
        concat_helper(("---swap--".into(), "  si   so".into()), get_swap_info),
        concat_helper(("-----io----".into(), "   bi    bo".into()), get_io_info),
        concat_helper(("-system--".into(), "  in   cs".into()), get_system_info),
        concat_helper(
            ("-------cpu-------".into(), "us sy id wa st gu".into()),
            get_cpu_info,
        ),
    ]
}

#[cfg(target_os = "linux")]
fn concat_helper(
    title: (String, String),
    func: impl Fn(&ProcData, Option<&ProcData>) -> Vec<(usize, String)> + 'static,
) -> Picker {
    (
        title,
        Box::from(
            move |proc_data: &ProcData,
                  proc_data_before: Option<&ProcData>,
                  data: &mut Vec<String>,
                  data_len_excess: &mut usize| {
                let output = func(proc_data, proc_data_before);
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
) -> Vec<(usize, String)> {
    let runnable = proc_data.stat.get("procs_running").unwrap();
    let blocked = proc_data.stat.get("procs_blocked").unwrap();

    vec![(2, runnable.to_string()), (2, blocked.to_string())]
}

#[cfg(target_os = "linux")]
fn get_memory_info(
    proc_data: &ProcData,
    _proc_data_before: Option<&ProcData>,
) -> Vec<(usize, String)> {
    use bytesize::*;

    let memory_info = Meminfo::from_proc_map(&proc_data.meminfo);

    let swap_used = (memory_info.swap_total - memory_info.swap_free).as_u64() / KB;
    let free = memory_info.mem_free.as_u64() / KB;
    let buffer = memory_info.buffers.as_u64() / KB;
    let cache = memory_info.cached.as_u64() / KB;

    vec![
        (6, format!("{}", swap_used)),
        (6, format!("{}", free)),
        (6, format!("{}", buffer)),
        (6, format!("{}", cache)),
    ]
}

#[cfg(target_os = "linux")]
fn get_swap_info(
    proc_data: &ProcData,
    proc_data_before: Option<&ProcData>,
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
fn get_io_info(proc_data: &ProcData, proc_data_before: Option<&ProcData>) -> Vec<(usize, String)> {
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
) -> Vec<(usize, String)> {
    let cpu_load = CpuLoad::from_proc_map(&proc_data.stat);

    vec![
        (2, format!("{:.0}", cpu_load.user)),
        (2, format!("{:.0}", cpu_load.system)),
        (2, format!("{:.0}", cpu_load.idle)),
        (2, format!("{:.0}", cpu_load.io_wait)),
        (2, format!("{:.0}", cpu_load.steal_time)),
        (2, format!("{:.0}", cpu_load.guest)),
    ]
}
