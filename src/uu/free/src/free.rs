// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

#[cfg(target_os = "windows")]
mod windows_util;

use bytesize::{ByteSize, GB, GIB, KB, KIB, MB, MIB, PB, PIB, TB, TIB};
use clap::{arg, crate_version, ArgAction, ArgGroup, ArgMatches, Command};
use std::env;

#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::io::Error;
use std::ops::Mul;
use std::process;
use std::thread::sleep;
use std::time::Duration;
use uucore::{
    error::{UResult, USimpleError},
    format_usage, help_about, help_usage,
};

const ABOUT: &str = help_about!("free.md");
const USAGE: &str = help_usage!("free.md");

/// The unit of number is [UnitMultiplier::Bytes]
#[derive(Default, Clone)]
struct MemInfo {
    total: u64,
    free: u64,
    available: u64,
    shared: u64,
    buffers: u64,
    cached: u64,
    swap_total: u64,
    swap_free: u64,
    swap_used: u64,
    reclaimable: u64,
    low_total: u64,
    low_free: u64,
    high_total: u64,
    high_free: u64,
    commit_limit: u64,
    committed: u64,
}

#[cfg(target_os = "linux")]
fn parse_meminfo() -> Result<MemInfo, Error> {
    // kernel docs: https://www.kernel.org/doc/html/latest/filesystems/proc.html#meminfo
    let contents = fs::read_to_string("/proc/meminfo")?;
    let mut mem_info = MemInfo::default();

    for line in contents.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let parsed_value = parse_meminfo_value(value)?;
            match key.trim() {
                "MemTotal" => mem_info.total = parsed_value,
                "MemFree" => mem_info.free = parsed_value,
                "MemAvailable" => mem_info.available = parsed_value,
                "Shmem" => mem_info.shared = parsed_value,
                "Buffers" => mem_info.buffers = parsed_value,
                "Cached" => mem_info.cached = parsed_value,
                "SwapTotal" => mem_info.swap_total = parsed_value,
                "SwapFree" => mem_info.swap_free = parsed_value,
                "SReclaimable" => mem_info.reclaimable = parsed_value,
                "LowTotal" => mem_info.low_total = parsed_value,
                "LowFree" => mem_info.low_free = parsed_value,
                "HighTotal" => mem_info.high_total = parsed_value,
                "HighFree" => mem_info.high_free = parsed_value,
                "CommitLimit" => mem_info.commit_limit = parsed_value,
                "Committed_AS" => mem_info.committed = parsed_value,
                _ => {}
            }
        }
    }
    // as far as i understand the kernel doc everything that is not highmem (out of all the memory) is lowmem
    // from kernel doc: "Highmem is all memory above ~860MB of physical memory."
    // it would be better to implement this via optionals, etc. but that would require a refactor so lets not do that right now

    if mem_info.low_total == u64::default() {
        mem_info.low_total = mem_info.total - mem_info.high_total;
    }

    if mem_info.low_free == u64::default() {
        mem_info.low_free = mem_info.free - mem_info.high_free;
    }

    mem_info.swap_used = mem_info.swap_total - mem_info.swap_free;

    Ok(mem_info)
}

#[cfg(target_os = "macos")]
fn parse_meminfo() -> Result<MemInfo, Box<dyn std::error::Error>> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_memory();

    let mem_info = MemInfo {
        total: sys.total_memory(),
        free: sys.free_memory(),
        // `available` memory is not directly provided by sysinfo, so you might use `free` or calculate an approximation.
        available: sys.free_memory(),
        shared: 0,
        buffers: 0,
        cached: sys.used_memory().saturating_sub(sys.free_memory()),
        swap_total: sys.total_swap(),
        swap_free: sys.free_swap(),
        swap_used: sys.total_swap().saturating_sub(sys.free_swap()),
        reclaimable: 0,
        low_total: 0,
        low_free: 0,
        high_total: 0,
        high_free: 0,
        commit_limit: 0,
        committed: 0,
    };

    Ok(mem_info)
}

#[cfg(target_os = "windows")]
fn parse_meminfo() -> Result<MemInfo, Box<dyn std::error::Error>> {
    use std::mem::size_of;
    use windows::Win32::System::{
        ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION},
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
    };

    let (pagefile_used, pagefile_total) = windows_util::get_pagefile_usage()?;

    let mut status = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe { GlobalMemoryStatusEx(&mut status)? }

    let mut perf_info = PERFORMANCE_INFORMATION {
        cb: size_of::<PERFORMANCE_INFORMATION>() as u32,
        ..Default::default()
    };
    unsafe { GetPerformanceInfo(&mut perf_info, perf_info.cb)? }

    let mem_info = MemInfo {
        total: status.ullTotalPhys / 1024,
        free: (status.ullAvailPhys - (perf_info.SystemCache * perf_info.PageSize) as u64) / 1024,
        available: status.ullAvailPhys / 1024,
        cached: (perf_info.SystemCache * perf_info.PageSize) as u64 / 1024,
        swap_total: (pagefile_total as u64 * perf_info.PageSize as u64) / 1024,
        swap_free: ((pagefile_total - pagefile_used) as u64 * perf_info.PageSize as u64) / 1024,
        swap_used: (pagefile_used as u64 * perf_info.PageSize as u64) / 1024,
        commit_limit: (perf_info.CommitLimit * perf_info.PageSize) as u64 / 1024,
        committed: (perf_info.CommitTotal * perf_info.PageSize) as u64 / 1024,
        ..Default::default()
    };

    Ok(mem_info)
}

// print total - used - free combo that is used for everything except memory for now
// free can be negative if the memory is overcommitted so it has to be signed
fn construct_tuf_combo_str<F>(name: &str, total: u64, used: u64, free: i128, f: F) -> String
where
    F: Fn(u64) -> String,
{
    // ugly hack to convert negative values
    let free_str: String = if free < 0 {
        "-".to_owned() + &f((-free) as u64)
    } else {
        f(free as u64)
    };

    format!(
        "{:8}{:>12}{:>12}{:>12}\n",
        name,
        f(total),
        f(used),
        free_str
    )
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let count: Option<u64> = matches.get_one("count").copied();
    let seconds: Option<f64> = matches.get_one("seconds").copied();

    if count == Some(0) {
        return Err(USimpleError::new(
            1,
            "count argument must be greater than 0",
        ));
    }

    if seconds == Some(0.0) {
        return Err(USimpleError::new(
            1,
            "seconds argument must be greater than 0",
        ));
    }

    let (count, seconds) = match (count, seconds) {
        (None, None) => (Some(1), 1.0),
        (None, Some(s)) => (None, s),
        (Some(c), None) => (Some(c), 1.0),
        (Some(c), Some(s)) => (Some(c), s),
    };

    let duration = Duration::from_nanos(seconds.mul(1_000_000_000.0).round() as u64);
    let construct_str = parse_output_format(&matches);

    let output_meminfo = || match parse_meminfo() {
        Ok(mem_info) => {
            print!("{}", construct_str(&mem_info));
        }
        Err(e) => {
            eprintln!("free: failed to read memory info: {e}");
            process::exit(1);
        }
    };

    let do_sleep = || {
        if !matches.get_flag("line") {
            println!();
        }
        sleep(duration);
    };

    if let Some(c) = count {
        for _ in 0..c - 1 {
            output_meminfo();
            do_sleep();
        }
        output_meminfo();
    } else {
        loop {
            output_meminfo();
            do_sleep();
        }
    }

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .args_override_self(true)
        .infer_long_args(true)
        .disable_help_flag(true)
        .group(ArgGroup::new("unit").args([
            "bytes", "kilo", "mega", "giga", "tera", "peta", "kibi", "mebi", "gibi", "tebi", "pebi",
        ]))
        .args([
            arg!(-b --bytes  "show output in bytes").action(ArgAction::SetTrue),
            arg!(   --kilo   "show output in kilobytes").action(ArgAction::SetTrue),
            arg!(   --mega   "show output in megabytes").action(ArgAction::SetTrue),
            arg!(   --giga   "show output in gigabytes").action(ArgAction::SetTrue),
            arg!(   --tera   "show output in terabytes").action(ArgAction::SetTrue),
            arg!(   --peta   "show output in petabytes").action(ArgAction::SetTrue),
            arg!(-k --kibi   "show output in kibibytes").action(ArgAction::SetTrue),
            arg!(-m --mebi   "show output in mebibytes").action(ArgAction::SetTrue),
            arg!(-g --gibi   "show output in gibibytes").action(ArgAction::SetTrue),
            arg!(   --tebi   "show output in tebibytes").action(ArgAction::SetTrue),
            arg!(   --pebi   "show output in pebibytes").action(ArgAction::SetTrue),
            arg!(-h --human  "show human-readable output").action(ArgAction::SetTrue),
            arg!(   --si     "use powers of 1000 not 1024").action(ArgAction::SetTrue),
            arg!(-l --lohi   "show detailed low and high memory statistics")
                .action(ArgAction::SetTrue),
            arg!(-t --total "show total for RAM + swap").action(ArgAction::SetTrue),
            arg!(-v --committed "show committed memory and commit limit")
                .action(ArgAction::SetTrue),
            // accept 1 as well as 0.5, 0.55, ...
            arg!(-s --seconds "repeat printing every N seconds")
                .action(ArgAction::Set)
                .value_name("N")
                .value_parser(clap::value_parser!(f64)),
            // big int because predecesor accepts them as well (some scripts might have huge values as some sort of infinite)
            arg!(-c --count "repeat printing N times, then exit")
                .action(ArgAction::Set)
                .value_name("N")
                .value_parser(clap::value_parser!(u64)),
            arg!(-L --line "show output on a single line").action(ArgAction::SetTrue),
            arg!(-w --wide "wide output").action(ArgAction::SetTrue),
            arg!(   --help "display this help and exit").action(ArgAction::Help),
        ])
}

#[cfg(target_os = "linux")]
fn parse_meminfo_value(value: &str) -> Result<u64, std::io::Error> {
    value
        .split_whitespace()
        .next()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid memory info format",
            )
        })
        .and_then(|v| {
            v.parse::<u64>().map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid memory info format",
                )
            })
        })
}

fn parse_output_format(matches: &ArgMatches) -> impl Fn(&MemInfo) -> String {
    let wide = matches.get_flag("wide");
    let human = matches.get_flag("human");
    let si = matches.get_flag("si");
    let total = matches.get_flag("total");
    let lohi = matches.get_flag("lohi");
    let committed = matches.get_flag("committed");
    let one_line = matches.get_flag("line");

    let convert = detect_unit(matches);

    // function that converts the number to the correct string
    let n2s = move |x| {
        if human {
            humanized(x, si)
        } else {
            convert(x).to_string()
        }
    };

    move |mem_info: &MemInfo| {
        if one_line {
            construct_one_line_str(mem_info, &n2s)
        } else {
            let mut str = String::new();
            if wide {
                str += &construct_wide_str(mem_info, &n2s);
            } else {
                str += &construct_str(mem_info, &n2s);
            }

            if lohi {
                str += &construct_lohi_str(mem_info, &n2s);
            }

            str += &construct_swap_str(mem_info, &n2s);

            if total {
                str += &construct_total_str(mem_info, &n2s);
            }

            if committed {
                str += &construct_committed_str(mem_info, &n2s);
            }

            str
        }
    }
}

fn construct_one_line_str(mem_info: &MemInfo, n2s: &dyn Fn(u64) -> String) -> String {
    format!(
        "{:8}{:>11} {:8}{:>11}  {:8}{:>10} {:8}{:>11}\n",
        "SwapUse",
        n2s(mem_info.swap_used),
        "CachUse",
        n2s(mem_info.buffers + mem_info.cached + mem_info.reclaimable),
        "MemUse",
        n2s(mem_info.total - mem_info.available),
        "MemFree",
        n2s(mem_info.free)
    )
}

fn construct_wide_str(mem_info: &MemInfo, n2s: &dyn Fn(u64) -> String) -> String {
    format!(
        "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}\n",
        " ", "total", "used", "free", "shared", "buffers", "cache", "available",
    ) + &format!(
        "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}\n",
        "Mem:",
        n2s(mem_info.total),
        n2s(mem_info.total - mem_info.available),
        n2s(mem_info.free),
        n2s(mem_info.shared),
        n2s(mem_info.buffers),
        n2s(mem_info.cached + mem_info.reclaimable),
        n2s(mem_info.available),
    )
}

fn construct_str(mem_info: &MemInfo, n2s: &dyn Fn(u64) -> String) -> String {
    format!(
        "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}\n",
        " ", "total", "used", "free", "shared", "buff/cache", "available",
    ) + &format!(
        "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}\n",
        "Mem:",
        n2s(mem_info.total),
        n2s(mem_info.total - mem_info.available),
        n2s(mem_info.free),
        n2s(mem_info.shared),
        n2s(mem_info.buffers + mem_info.cached + mem_info.reclaimable),
        n2s(mem_info.available),
    )
}

fn construct_lohi_str(mem_info: &MemInfo, n2s: &dyn Fn(u64) -> String) -> String {
    construct_tuf_combo_str(
        "Low:",
        mem_info.low_total,
        mem_info.low_total - mem_info.low_free,
        mem_info.low_free.into(),
        n2s,
    ) + &construct_tuf_combo_str(
        "High:",
        mem_info.high_total,
        mem_info.high_total - mem_info.high_free,
        mem_info.high_free.into(),
        n2s,
    )
}

fn construct_swap_str(mem_info: &MemInfo, n2s: &dyn Fn(u64) -> String) -> String {
    construct_tuf_combo_str(
        "Swap:",
        mem_info.swap_total,
        mem_info.swap_used,
        mem_info.swap_free.into(),
        n2s,
    )
}

fn construct_total_str(mem_info: &MemInfo, n2s: &dyn Fn(u64) -> String) -> String {
    construct_tuf_combo_str(
        "Total:",
        mem_info.total + mem_info.swap_total,
        mem_info.total - mem_info.available + mem_info.swap_used,
        (mem_info.free + mem_info.swap_free).into(),
        n2s,
    )
}

fn construct_committed_str(mem_info: &MemInfo, n2s: &dyn Fn(u64) -> String) -> String {
    construct_tuf_combo_str(
        "Comm:",
        mem_info.commit_limit,
        mem_info.committed,
        (mem_info.commit_limit as i128) - (mem_info.committed as i128),
        n2s,
    )
}

// Here's the `-h` `--human` flag processing logic
// See: https://github.com/uutils/procps/pull/431
fn humanized(kib: u64, si: bool) -> String {
    let b = ByteSize::kib(kib).0;
    let units = ['B', 'K', 'M', 'G', 'T', 'P'];
    let mut level = 0;
    let mut divisor = 1;
    while level < units.len() - 1 && divisor * 100 <= b {
        divisor *= if si { 1000 } else { 1024 };
        level += 1;
    }
    if level == 0 {
        return format!("{}{}", b, units[level]);
    }

    let value = (b as f64) / (divisor as f64);
    let formatted_value = if (value * 10.0).round() < 100.0 {
        format!("{:.1}", (value * 10.0).round() / 10.0)
    } else {
        (value as u64).to_string()
    };

    format!(
        "{}{}{}",
        formatted_value,
        units[level].to_owned(),
        if si { "" } else { "i" }
    )
}

fn detect_unit(arg: &ArgMatches) -> fn(u64) -> u64 {
    let si = arg.get_flag("si");
    match arg {
        _ if arg.get_flag("bytes") => |kib: u64| ByteSize::kib(kib).0,
        _ if arg.get_flag("kilo") || (si && arg.get_flag("kibi")) => {
            |kib: u64| ByteSize::kib(kib).0 / KB
        }
        _ if arg.get_flag("mega") || (si && arg.get_flag("mebi")) => {
            |kib: u64| ByteSize::kib(kib).0 / MB
        }
        _ if arg.get_flag("giga") || (si && arg.get_flag("gibi")) => {
            |kib: u64| ByteSize::kib(kib).0 / GB
        }
        _ if arg.get_flag("tera") || (si && arg.get_flag("tebi")) => {
            |kib: u64| ByteSize::kib(kib).0 / TB
        }
        _ if arg.get_flag("peta") || (si && arg.get_flag("pebi")) => {
            |kib: u64| ByteSize::kib(kib).0 / PB
        }
        _ if arg.get_flag("kibi") => |kib: u64| ByteSize::kib(kib).0 / KIB,
        _ if arg.get_flag("mebi") => |kib: u64| ByteSize::kib(kib).0 / MIB,
        _ if arg.get_flag("gibi") => |kib: u64| ByteSize::kib(kib).0 / GIB,
        _ if arg.get_flag("tebi") => |kib: u64| ByteSize::kib(kib).0 / TIB,
        _ if arg.get_flag("pebi") => |kib: u64| ByteSize::kib(kib).0 / PIB,
        _ if si => |kib: u64| ByteSize::kib(kib).0 / KB,
        _ => |kib: u64| kib,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_line_wide() {
        let matches_with_line = uu_app()
            .try_get_matches_from(vec!["free", "--line"])
            .unwrap();
        let matches_with_line_wide = uu_app()
            .try_get_matches_from(vec!["free", "--line", "--wide"])
            .unwrap();
        let construct_line_str = parse_output_format(&matches_with_line);
        let construct_line_wide_str = parse_output_format(&matches_with_line_wide);
        match parse_meminfo() {
            Ok(mem_info) => {
                assert_eq!(
                    construct_line_str(&mem_info),
                    construct_line_wide_str(&mem_info)
                );
            }
            Err(e) => {
                eprintln!("free: failed to read memory info: {e}");
            }
        }
    }

    #[test]
    fn test_humanized_unit() {
        let test_cases = [
            (0, false, "0B"),
            (0, true, "0B"),
            (1023, false, "1.0Mi"),
            (1024, true, "1.0M"),
            (1024, false, "1.0Mi"),
            (1536, true, "1.6M"),
            (1536, false, "1.5Mi"),
            (8500, true, "8.7M"),
            (8500, false, "8.3Mi"),
            (10138, false, "9.9Mi"),
            (10230, false, "9Mi"),
        ];
        for &(kib, si, expected) in &test_cases {
            assert_eq!(humanized(kib, si), expected);
        }
    }
}
