// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use bytesize::ByteSize;
use bytesize::GB;
use bytesize::GIB;
use bytesize::KIB;
use bytesize::MB;
use bytesize::MIB;
use bytesize::PB;
use bytesize::PIB;
use bytesize::TB;
use bytesize::TIB;
use clap::arg;
use clap::Arg;
use clap::ArgAction;
use clap::ArgGroup;
use clap::ArgMatches;
use clap::{crate_version, Command};
use std::env;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::io::Error;
use std::ops::Mul;
use std::process;
use std::thread::sleep;
use std::time::Duration;
use std::u64;
use uucore::{error::UResult, format_usage, help_about, help_usage};

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

// TODO: implement function
#[cfg(target_os = "windows")]
fn parse_meminfo() -> Result<MemInfo, Box<dyn std::error::Error>> {
    Ok(MemInfo::default())
}

// print total - used - free combo that is used for everything except memory for now
// free can be negative if the memory is overcommitted so it has to be signed
fn tuf_combo<F>(name: &str, total: u64, used: u64, free: i128, f: F)
where
    F: Fn(u64) -> String,
{
    // ugly hack to convert negative values
    let free_str: String = if free < 0 {
        "-".to_owned() + &f((-free) as u64)
    } else {
        f(free as u64)
    };

    println!("{:8}{:>12}{:>12}{:>12}", name, f(total), f(used), free_str);
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let wide = matches.get_flag("wide");
    let human = matches.get_flag("human");
    let si = matches.get_flag("si");
    let total = matches.get_flag("total");
    let lohi = matches.get_flag("lohi");
    let count_flag = matches.get_one("count");
    let mut count: u64 = count_flag.unwrap_or(&1_u64).to_owned();
    let seconds_flag = matches.get_one("seconds");
    let seconds: f64 = seconds_flag.unwrap_or(&1.0_f64).to_owned();
    let committed = matches.get_flag("committed");
    let one_line = matches.get_flag("line");

    let dur = Duration::from_nanos(seconds.mul(1_000_000_000.0).round() as u64);
    let convert = detect_unit(&matches);

    let infinite: bool = count_flag.is_none() && seconds_flag.is_some();

    while count > 0 || infinite {
        // prevent underflow
        if !infinite {
            count -= 1;
        }
        match parse_meminfo() {
            Ok(mem_info) => {
                let buff_cache = if wide {
                    mem_info.buffers
                } else {
                    mem_info.buffers + mem_info.cached
                };
                let cache = if wide { mem_info.cached } else { 0 };
                let used = mem_info.total - mem_info.available;

                // function that converts the number to the correct string
                let n2s = |x| match human {
                    true => humanized(x, si),
                    false => convert(x).to_string(),
                };
                if one_line {
                    println!(
                        "{:8}{:>12} {:8}{:>12}  {:8}{:>12} {:8}{:>12}",
                        "SwapUse",
                        n2s(mem_info.swap_used),
                        "CacheUse",
                        n2s(buff_cache + mem_info.reclaimable),
                        "MemUse",
                        n2s(used),
                        "MemFree",
                        n2s(mem_info.free)
                    );
                } else {
                    if wide {
                        wide_header();
                        println!(
                            "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}",
                            "Mem:",
                            n2s(mem_info.total),
                            n2s(used),
                            n2s(mem_info.free),
                            n2s(mem_info.shared),
                            n2s(buff_cache),
                            n2s(cache + mem_info.reclaimable),
                            n2s(mem_info.available),
                        );
                    } else {
                        header();
                        println!(
                            "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}",
                            "Mem:",
                            n2s(mem_info.total),
                            n2s(used),
                            n2s(mem_info.free),
                            n2s(mem_info.shared),
                            n2s(buff_cache + mem_info.reclaimable),
                            n2s(mem_info.available),
                        )
                    }

                    if lohi {
                        tuf_combo(
                            "Low:",
                            mem_info.low_total,
                            mem_info.low_total - mem_info.low_free,
                            mem_info.low_free.into(),
                            n2s,
                        );
                        tuf_combo(
                            "High:",
                            mem_info.high_total,
                            mem_info.high_total - mem_info.high_free,
                            mem_info.high_free.into(),
                            n2s,
                        );
                    }

                    tuf_combo(
                        "Swap:",
                        mem_info.swap_total,
                        mem_info.swap_used,
                        mem_info.swap_free.into(),
                        n2s,
                    );
                    if total {
                        tuf_combo(
                            "Total:",
                            mem_info.total + mem_info.swap_total,
                            used + mem_info.swap_used,
                            (mem_info.free + mem_info.swap_free).into(),
                            n2s,
                        );
                    }

                    if committed {
                        tuf_combo(
                            "Comm:",
                            mem_info.commit_limit,
                            mem_info.committed,
                            (mem_info.commit_limit as i128) - (mem_info.committed as i128),
                            n2s,
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("free: failed to read memory info: {}", e);
                process::exit(1);
            }
        }
        if count > 0 || infinite {
            // the original free prints a newline everytime before waiting for the next round
            println!();
            sleep(dur);
        }
    }

    Ok(())
}

pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .disable_help_flag(true)
        .group(ArgGroup::new("unit").args([
            "bytes", "kilo", "mega", "giga", "tera", "peta", "kibi", "mebi", "gibi", "tebi", "pebi",
        ]))
        .args([
            arg!(-b --bytes  "show output in bytes").action(ArgAction::SetTrue),
            arg!(   --kilo   "show output in kilobytes").action(ArgAction::SetFalse),
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
            arg!(   --si     "use powers of 1000 not 1024").action(ArgAction::SetFalse),
            arg!(-l --lohi   "show detailed low and high memory statistics")
                .action(ArgAction::SetTrue),
            arg!(-t --total "show total for RAM + swap").action(ArgAction::SetTrue),
            arg!(-v --committed "show committed memory and commit limit")
                .action(ArgAction::SetTrue),
            // accept 1 as well as 0.5, 0.55, ...
            arg!(-s --seconds "repeat printing every N seconds")
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(f64)),
            // big int because predecesor accepts them as well (some scripts might have huge values as some sort of infinite)
            arg!(-c --count "repeat printing N times, then exit")
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(u64)),
            arg!(-L --line "show output on a single line").action(ArgAction::SetTrue),
        ])
        .arg(
            Arg::new("wide")
                .short('w')
                .long("wide")
                .help("wide output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("help")
                .long("help")
                .action(ArgAction::Help)
                .help("display this help and exit"),
        )
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

// Here's the `-h` `--human` flag processing logic
fn humanized(kib: u64, si: bool) -> String {
    let binding = ByteSize::kib(kib).to_string_as(si);
    let split: Vec<&str> = binding.split(' ').collect();

    // TODO: finish the logic of automatic scale.
    let num_string = String::from(split[0]);

    let unit_string = {
        let mut tmp = String::from(split[1]);
        tmp.pop();
        tmp
    };
    format!("{}{}", num_string, unit_string)
}

fn detect_unit(arg: &ArgMatches) -> fn(u64) -> u64 {
    match arg {
        _ if arg.get_flag("bytes") => |kib: u64| ByteSize::kib(kib).0,
        _ if arg.get_flag("mega") => |kib: u64| ByteSize::kib(kib).0 / MB,
        _ if arg.get_flag("giga") => |kib: u64| ByteSize::kib(kib).0 / GB,
        _ if arg.get_flag("tera") => |kib: u64| ByteSize::kib(kib).0 / TB,
        _ if arg.get_flag("peta") => |kib: u64| ByteSize::kib(kib).0 / PB,
        _ if arg.get_flag("kibi") => |kib: u64| ByteSize::kib(kib).0 / KIB,
        _ if arg.get_flag("mebi") => |kib: u64| ByteSize::kib(kib).0 / MIB,
        _ if arg.get_flag("gibi") => |kib: u64| ByteSize::kib(kib).0 / GIB,
        _ if arg.get_flag("tebi") => |kib: u64| ByteSize::kib(kib).0 / TIB,
        _ if arg.get_flag("pebi") => |kib: u64| ByteSize::kib(kib).0 / PIB,
        _ => |kib: u64| kib,
    }
}

fn wide_header() {
    println!(
        "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}",
        " ", "total", "used", "free", "shared", "buffers", "cache", "available",
    );
}

fn header() {
    println!(
        "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}",
        " ", "total", "used", "free", "shared", "buff/cache", "available",
    )
}
