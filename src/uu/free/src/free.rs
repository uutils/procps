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
#[derive(Default)]
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
}

#[cfg(target_os = "linux")]
fn parse_meminfo() -> Result<MemInfo, Error> {
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
                _ => {}
            }
        }
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
    };

    Ok(mem_info)
}

// TODO: implement function
#[cfg(target_os = "windows")]
fn parse_meminfo() -> Result<MemInfo, Box<dyn std::error::Error>> {
    Ok(MemInfo::default())
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;

    let wide = matches.get_flag("wide");
    let human = matches.get_flag("human");
    let si = matches.get_flag("si");
    let total = matches.get_flag("total");
    let mut count: u64 = matches.get_one("count").unwrap_or(&1_u64).to_owned();
    let seconds: f64 = matches.get_one("seconds").unwrap_or(&1.0_f64).to_owned();

    let dur = Duration::from_nanos(seconds.mul(1_000_000_000.0).round() as u64);
    let convert = detect_unit(&matches);

    while count > 0 {
        count -= 1;
        match parse_meminfo() {
            Ok(mem_info) => {
                let buff_cache = if wide {
                    mem_info.buffers
                } else {
                    mem_info.buffers + mem_info.cached
                };
                let cache = if wide { mem_info.cached } else { 0 };
                let used = mem_info.total - mem_info.available;

                if wide {
                    wide_header();
                    if human {
                        println!(
                            "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}",
                            "Mem:",
                            humanized(mem_info.total, si),
                            humanized(used, si),
                            humanized(mem_info.free, si),
                            humanized(mem_info.shared, si),
                            humanized(buff_cache, si),
                            humanized(cache + mem_info.reclaimable, si),
                            humanized(mem_info.available, si),
                        )
                    } else {
                        println!(
                            "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}",
                            "Mem:",
                            convert(mem_info.total),
                            convert(used),
                            convert(mem_info.free),
                            convert(mem_info.shared),
                            convert(buff_cache),
                            convert(cache + mem_info.reclaimable),
                            convert(mem_info.available),
                        )
                    }
                } else {
                    header();
                    if human {
                        println!(
                            "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}",
                            "Mem:",
                            humanized(mem_info.total, si),
                            humanized(used, si),
                            humanized(mem_info.free, si),
                            humanized(mem_info.shared, si),
                            humanized(buff_cache + mem_info.reclaimable, si),
                            humanized(mem_info.available, si),
                        )
                    } else {
                        println!(
                            "{:8}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}",
                            "Mem:",
                            convert(mem_info.total),
                            convert(used),
                            convert(mem_info.free),
                            convert(mem_info.shared),
                            convert(buff_cache + mem_info.reclaimable),
                            convert(mem_info.available),
                        )
                    }
                }
                if human {
                    println!(
                        "{:8}{:>12}{:>12}{:>12}",
                        "Swap:",
                        humanized(mem_info.swap_total, si),
                        humanized(mem_info.swap_used, si),
                        humanized(mem_info.swap_free, si)
                    );
                } else {
                    println!(
                        "{:8}{:>12}{:>12}{:>12}",
                        "Swap:",
                        convert(mem_info.swap_total),
                        convert(mem_info.swap_used),
                        convert(mem_info.swap_free)
                    );
                }
                if total {
                    if human {
                        println!(
                            "{:8}{:>12}{:>12}{:>12}",
                            "Total:",
                            humanized(mem_info.total + mem_info.swap_total, si),
                            humanized(used + mem_info.swap_used, si),
                            humanized(mem_info.free + mem_info.swap_free, si)
                        );
                    } else {
                        println!(
                            "{:8}{:>12}{:>12}{:>12}",
                            "Total:",
                            convert(mem_info.total + mem_info.swap_total),
                            convert(used + mem_info.swap_used),
                            convert(mem_info.free + mem_info.swap_free)
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("free: failed to read memory info: {}", e);
                process::exit(1);
            }
        }
        if count > 0 {
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
            arg!(-b --bytes "show output in bytes").action(ArgAction::SetTrue),
            arg!(   --kilo  "show output in kilobytes").action(ArgAction::SetFalse),
            arg!(   --mega  "show output in megabytes").action(ArgAction::SetTrue),
            arg!(   --giga  "show output in gigabytes").action(ArgAction::SetTrue),
            arg!(   --tera  "show output in terabytes").action(ArgAction::SetTrue),
            arg!(   --peta  "show output in petabytes").action(ArgAction::SetTrue),
            arg!(-k --kibi  "show output in kibibytes").action(ArgAction::SetTrue),
            arg!(-m --mebi  "show output in mebibytes").action(ArgAction::SetTrue),
            arg!(-g --gibi  "show output in gibibytes").action(ArgAction::SetTrue),
            arg!(   --tebi  "show output in tebibytes").action(ArgAction::SetTrue),
            arg!(   --pebi  "show output in pebibytes").action(ArgAction::SetTrue),
            arg!(-h --human "show human-readable output").action(ArgAction::SetTrue),
            arg!(   --si    "use powers of 1000 not 1024").action(ArgAction::SetFalse),
            // TODO: implement those
            // arg!(-l --lohi "show detailed low and high memory statistics").action(),
            // arg!(-L --line "show output on a single line").action(),
            arg!(-t --total "show total for RAM + swap").action(ArgAction::SetTrue),
            // arg!(-v --committed "show committed memory and commit limit").action(),
            // accept 1 as well as 0.5, 0.55, ...
            arg!(-s --seconds "repeat printing every N seconds")
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(f64)),
            // big int because predecesor accepts them as well (some scripts might have huge values as some sort of infinite)
            arg!(-c --count "repeat printing N times, then exit")
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(u64)),
            // arg!(-L --line "show output on a single line").action(),
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
