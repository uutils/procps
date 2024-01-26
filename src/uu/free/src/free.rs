// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::Arg;
use clap::ArgAction;
use clap::{crate_version, Command};
use std::env;
use std::fs;
use std::process;
use uucore::{error::UResult, format_usage, help_about, help_usage};

const ABOUT: &str = help_about!("free.md");
const USAGE: &str = help_usage!("free.md");

fn parse_meminfo() -> Result<(u64, u64, u64, u64, u64, u64, u64, u64, u64, u64), std::io::Error> {
    let contents = fs::read_to_string("/proc/meminfo")?;
    let mut total = 0;
    let mut free = 0;
    let mut available = 0;
    let mut shared = 0;
    let mut buffers = 0;
    let mut cached = 0;
    let mut swap_total = 0;
    let mut swap_free = 0;
    let mut reclaimable = 0;

    for line in contents.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let parsed_value = parse_meminfo_value(value)?;
            match key.trim() {
                "MemTotal" => total = parsed_value,
                "MemFree" => free = parsed_value,
                "MemAvailable" => available = parsed_value,
                "Shmem" => shared = parsed_value,
                "Buffers" => buffers = parsed_value,
                "Cached" => cached = parsed_value,
                "SwapTotal" => swap_total = parsed_value,
                "SwapFree" => swap_free = parsed_value,
                "SReclaimable" => reclaimable = parsed_value,
                _ => {}
            }
        }
    }

    Ok((
        total,
        free,
        available,
        shared,
        buffers,
        cached,
        swap_total,
        swap_free,
        swap_total - swap_free,
        reclaimable,
    ))
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let wide = matches.get_flag("wide");

    match parse_meminfo() {
        Ok((
            total,
            free,
            available,
            shared,
            buffers,
            cached,
            swap_total,
            swap_free,
            swap_used,
            reclaimable,
        )) => {
            let buff_cache = if wide { buffers } else { buffers + cached };
            let cache = if wide { cached } else { 0 };
            let used = total - free;

            if wide {
                println!("              total        used        free      shared     buffers       cache   available");
                println!(
                    "Mem:   {:12} {:12} {:12} {:12} {:12} {:12} {:12}",
                    total,
                    used,
                    free,
                    shared,
                    buff_cache,
                    cache + reclaimable,
                    available
                );
            } else {
                println!("              total        used        free      shared  buff/cache   available");
                println!(
                    "Mem:   {:12} {:12} {:12} {:12} {:12} {:12}",
                    total,
                    used,
                    free,
                    shared,
                    buff_cache + reclaimable,
                    available
                );
            }
            println!("Swap:  {:12} {:12} {:12}", swap_total, swap_used, swap_free);
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
        .arg(
            Arg::new("wide")
                .short('w')
                .long("wide")
                .help("wide output")
                .action(ArgAction::SetTrue),
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
