// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::arg;
use clap::Arg;
use clap::ArgAction;
use clap::ArgGroup;
use clap::{crate_version, Command};
use std::env;
use std::fs;
use std::io::Error;
use std::process;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("free.md");
const USAGE: &str = help_usage!("free.md");

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

fn parse_meminfo() -> Result<MemInfo, Error> {
    let contents = fs::read_to_string("/proc/meminfo")?;
    let mut mem_info = MemInfo {
        total: 0,
        free: 0,
        available: 0,
        shared: 0,
        buffers: 0,
        cached: 0,
        swap_total: 0,
        swap_free: 0,
        swap_used: 0,
        reclaimable: 0,
    };

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

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let wide = matches.get_flag("wide");

    match parse_meminfo() {
        Ok(mem_info) => {
            let buff_cache = match wide {
                true => mem_info.buffers,
                false => mem_info.buffers + mem_info.cached,
            };
            let cache = match wide {
                true => mem_info.cached,
                false => 0,
            };
            let used = mem_info.total - mem_info.available;

            if wide {
                println!("               total        used        free      shared     buffers       cache   available");
                println!(
                    "Mem:     {:11} {:11} {:11} {:11} {:11} {:11} {:11}",
                    mem_info.total,
                    used,
                    mem_info.free,
                    mem_info.shared,
                    buff_cache,
                    cache + mem_info.reclaimable,
                    mem_info.available
                );
            } else {
                println!("               total        used        free      shared  buff/cache   available");
                println!(
                    "Mem:     {:11} {:11} {:11} {:11} {:11} {:11}",
                    mem_info.total,
                    used,
                    mem_info.free,
                    mem_info.shared,
                    buff_cache + mem_info.reclaimable,
                    mem_info.available
                );
            }
            println!(
                "Swap:    {:11} {:11} {:11}",
                mem_info.swap_total, mem_info.swap_used, mem_info.swap_free
            );
        }
        Err(e) => {
            eprintln!("free: failed to read memory info: {}", e);
            process::exit(1);
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
            // arg!(-L --line  "show output on a single line"),
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
