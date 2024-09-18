// This file is part of the uutils procps package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use clap::{crate_version, Arg, ArgAction, Command};
use maps_format_parser::parse_map_line;
use std::env;
use std::fs;
use std::io::Error;
use uucore::error::{set_exit_code, UResult};
use uucore::{format_usage, help_about, help_usage};

mod maps_format_parser;

const ABOUT: &str = help_about!("pmap.md");
const USAGE: &str = help_usage!("pmap.md");

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let matches = uu_app().try_get_matches_from(args)?;
    let pids = matches.get_many::<String>("pid").expect("PID required");

    for pid in pids {
        match parse_cmdline(pid) {
            Ok(cmdline) => {
                println!("{pid}:   {cmdline}");
            }
            Err(_) => {
                set_exit_code(42);
                continue;
            }
        }

        match parse_maps(pid) {
            Ok(total) => println!(" total {total:>16}K"),
            Err(_) => {
                set_exit_code(1);
            }
        }
    }

    Ok(())
}

fn parse_cmdline(pid: &str) -> Result<String, Error> {
    let path = format!("/proc/{pid}/cmdline");
    let contents = fs::read(path)?;
    // Command line arguments are separated by null bytes.
    // Replace them with spaces for display.
    let cmdline = contents
        .split(|&c| c == 0)
        .filter_map(|c| std::str::from_utf8(c).ok())
        .collect::<Vec<&str>>()
        .join(" ");
    let cmdline = cmdline.trim_end();
    Ok(cmdline.into())
}

fn parse_maps(pid: &str) -> Result<u64, Error> {
    let path = format!("/proc/{pid}/maps");
    let contents = fs::read_to_string(path)?;
    let mut total = 0;

    for line in contents.lines() {
        let map_line = parse_map_line(line);
        println!(
            "{} {:>6}K {} {}",
            map_line.address, map_line.size_in_kb, map_line.perms, map_line.mapping
        );
        total += map_line.size_in_kb;
    }

    Ok(total)
}

pub fn uu_app() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg(
            Arg::new("pid")
                .help("Process ID")
                .required_unless_present_any(["create-rc", "create-rc-to"]) // Adjusted for -n, -N note
                .action(ArgAction::Append)
                .conflicts_with_all(["create-rc", "create-rc-to"]),
        ) // Ensure pid is not used with -n, -N
        .arg(
            Arg::new("extended")
                .short('x')
                .long("extended")
                .help("show details"),
        )
        .arg(
            Arg::new("very-extended")
                .short('X')
                .help("show even more details"),
        )
        .arg(
            Arg::new("all-details")
                .long("XX")
                .help("show everything the kernel provides"),
        )
        .arg(
            Arg::new("read-rc")
                .short('c')
                .long("read-rc")
                .help("read the default rc"),
        )
        .arg(
            Arg::new("read-rc-from")
                .short('C')
                .long("read-rc-from")
                .num_args(1)
                .help("read the rc from file"),
        )
        .arg(
            Arg::new("create-rc")
                .short('n')
                .long("create-rc")
                .help("create new default rc"),
        )
        .arg(
            Arg::new("create-rc-to")
                .short('N')
                .long("create-rc-to")
                .num_args(1)
                .help("create new rc to file"),
        )
        .arg(
            Arg::new("device")
                .short('d')
                .long("device")
                .help("show the device format"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("do not display header and footer"),
        )
        .arg(
            Arg::new("show-path")
                .short('p')
                .long("show-path")
                .help("show path in the mapping"),
        )
        .arg(
            Arg::new("range")
                .short('A')
                .long("range")
                .num_args(1..=2)
                .help("limit results to the given range"),
        )
}
